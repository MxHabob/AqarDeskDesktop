use crate::{
    application::{
        ApplicationError, DashboardRepository, DashboardSummary, MonthlyPayment, StatusCount,
    },
    infrastructure::{
        currency_repository::Entity as CurrencyEntity,
        models::{payment, tenant, unit},
        property_repository::{Column as PropertyColumn, Entity as PropertyEntity},
    },
};
use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::Expr,
};
use std::collections::BTreeMap;

pub struct SeaOrmDashboardRepository {
    database: DatabaseConnection,
}

impl SeaOrmDashboardRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl DashboardRepository for SeaOrmDashboardRepository {
    async fn summary(
        &self,
        organization_id: &str,
        months: u8,
    ) -> Result<DashboardSummary, ApplicationError> {
        let timezone = crate::infrastructure::models::organization::Entity::find_by_id(
            organization_id,
        )
        .one(&self.database)
        .await
        .map_err(persistence_error)?
        .ok_or(ApplicationError::NotFound)?
        .timezone
        .parse::<Tz>()
        .map_err(|error| {
            tracing::error!(%error, organization_id, "Stored organization time zone is invalid");
            ApplicationError::Persistence
        })?;

        let month_ranges = reporting_months(months, timezone)?;
        let first_month = month_ranges.first().ok_or(ApplicationError::Persistence)?;
        let final_month = month_ranges.last().ok_or(ApplicationError::Persistence)?;

        let active_properties_count = PropertyEntity::find()
            .filter(PropertyColumn::OrganizationId.eq(organization_id))
            .filter(PropertyColumn::DeletedAt.is_null())
            .filter(PropertyColumn::Status.eq("active"))
            .count(&self.database)
            .await
            .map_err(persistence_error)?;

        let units_count = unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .count(&self.database)
            .await
            .map_err(persistence_error)?;
        let available_units_count = unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .filter(unit::Column::Status.eq("available"))
            .count(&self.database)
            .await
            .map_err(persistence_error)?;
        let tenants_count = tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .count(&self.database)
            .await
            .map_err(persistence_error)?;
        let posted_payments_count = payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .filter(payment::Column::Status.eq("posted"))
            .count(&self.database)
            .await
            .map_err(persistence_error)?;

        let statuses = unit::Entity::find()
            .select_only()
            .column(unit::Column::Status)
            .column_as(Expr::col(unit::Column::Id).count(), "count")
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .group_by(unit::Column::Status)
            .order_by_asc(unit::Column::Status)
            .into_tuple::<(String, i64)>()
            .all(&self.database)
            .await
            .map_err(persistence_error)?
            .into_iter()
            .map(|(status, count)| {
                u64::try_from(count)
                    .map(|count| StatusCount { status, count })
                    .map_err(|error| {
                        tracing::error!(%error, "Dashboard status count is negative");
                        ApplicationError::Persistence
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let payment_rows = payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .filter(payment::Column::Status.eq("posted"))
            .filter(payment::Column::ReceivedAt.gte(first_month.start))
            .filter(payment::Column::ReceivedAt.lt(final_month.end))
            .select_only()
            .column(payment::Column::ReceivedAt)
            .column(payment::Column::AmountMinor)
            .column(payment::Column::CurrencyCode)
            .into_tuple::<(DateTime<Utc>, i64, String)>()
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        let currencies = CurrencyEntity::find()
            .all(&self.database)
            .await
            .map_err(persistence_error)?
            .into_iter()
            .map(|currency| (currency.code, currency.decimal_digits))
            .collect::<std::collections::HashMap<_, _>>();
        let mut totals = BTreeMap::<(String, String), (i64, u64)>::new();
        for (received_at, amount_minor, currency_code) in payment_rows {
            let month = received_at
                .with_timezone(&timezone)
                .format("%Y-%m")
                .to_string();
            let total = totals.entry((month, currency_code)).or_default();
            total.0 = total.0.checked_add(amount_minor).ok_or_else(|| {
                tracing::error!("Dashboard payment sum exceeds the supported integer range");
                ApplicationError::Persistence
            })?;
            total.1 = total
                .1
                .checked_add(1)
                .ok_or(ApplicationError::Persistence)?;
        }
        let monthly_payments = totals
            .into_iter()
            .map(|((month, currency_code), (amount_minor, payment_count))| {
                let currency_decimal_digits =
                    currencies.get(&currency_code).copied().ok_or_else(|| {
                        tracing::error!(%currency_code, "Payment references an unknown currency");
                        ApplicationError::Persistence
                    })?;
                Ok(MonthlyPayment {
                    month,
                    currency_code,
                    currency_decimal_digits,
                    amount_minor,
                    payment_count,
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;

        Ok(DashboardSummary {
            active_properties_count,
            units_count,
            available_units_count,
            tenants_count,
            posted_payments_count,
            payment_months: month_ranges.into_iter().map(|month| month.label).collect(),
            monthly_payments,
            unit_status_counts: statuses,
        })
    }
}

struct ReportingMonth {
    label: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

fn reporting_months(months: u8, timezone: Tz) -> Result<Vec<ReportingMonth>, ApplicationError> {
    reporting_months_from(
        Utc::now().with_timezone(&timezone).date_naive(),
        months,
        timezone,
    )
}

fn reporting_months_from(
    local_now: NaiveDate,
    months: u8,
    timezone: Tz,
) -> Result<Vec<ReportingMonth>, ApplicationError> {
    let current_month = NaiveDate::from_ymd_opt(local_now.year(), local_now.month(), 1)
        .ok_or(ApplicationError::Persistence)?;
    let mut result = Vec::with_capacity(usize::from(months));
    for offset in (0..months).rev() {
        let month = current_month
            .checked_sub_months(Months::new(u32::from(offset)))
            .ok_or(ApplicationError::Persistence)?;
        let next_month = month
            .checked_add_months(Months::new(1))
            .ok_or(ApplicationError::Persistence)?;
        result.push(ReportingMonth {
            label: month.format("%Y-%m").to_string(),
            start: local_month_boundary(month, timezone)?,
            end: local_month_boundary(next_month, timezone)?,
        });
    }
    Ok(result)
}

fn local_month_boundary(date: NaiveDate, timezone: Tz) -> Result<DateTime<Utc>, ApplicationError> {
    let local = date
        .and_hms_opt(0, 0, 0)
        .ok_or(ApplicationError::Persistence)?;
    let boundary = match timezone.from_local_datetime(&local) {
        chrono::LocalResult::Single(value) => value,
        chrono::LocalResult::Ambiguous(first, second) => first.min(second),
        chrono::LocalResult::None => {
            tracing::error!(%date, %timezone, "Could not resolve reporting month boundary");
            return Err(ApplicationError::Persistence);
        }
    };
    Ok(boundary.with_timezone(&Utc))
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Dashboard query failed");
    ApplicationError::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn reporting_period_contains_exact_local_calendar_months() {
        let current = NaiveDate::from_ymd_opt(2026, 3, 15).expect("valid date");
        let months =
            reporting_months_from(current, 3, chrono_tz::Asia::Riyadh).expect("valid period");
        assert_eq!(months.len(), 3);
        assert_eq!(months[0].label, "2026-01");
        assert_eq!(months[1].label, "2026-02");
        assert_eq!(months[2].label, "2026-03");
        assert_eq!(
            months[0]
                .end
                .signed_duration_since(months[0].start)
                .num_hours(),
            24 * 31
        );
        assert_eq!(
            months[1]
                .end
                .signed_duration_since(months[1].start)
                .num_hours(),
            24 * 28
        );
        assert_eq!(
            months[2]
                .end
                .signed_duration_since(months[2].start)
                .num_hours(),
            24 * 31
        );
    }

    #[test]
    fn reporting_month_boundaries_respect_daylight_saving_offsets() {
        let month = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid month");
        let timezone = chrono_tz::America::New_York;
        let boundary = local_month_boundary(month, timezone).expect("valid boundary");
        assert_eq!(boundary.with_timezone(&timezone).hour(), 0);
        let next_month = NaiveDate::from_ymd_opt(2026, 4, 1).expect("valid next month");
        let next_boundary =
            local_month_boundary(next_month, timezone).expect("valid next boundary");
        assert_eq!(
            next_boundary.signed_duration_since(boundary).num_hours(),
            743
        );
    }
}
