use crate::{
    application::{
        ApplicationError, CurrencyAmountTotal, DashboardRepository, DashboardSummary,
        MonthlyPayment, PaymentCurrencyTotal, PropertyDashboardSummary, StatusCount,
    },
    infrastructure::{
        currency_repository::Entity as CurrencyEntity,
        models::{payment, payment_allocation, property_dashboard_contract, tenant, unit},
        property_repository::{Column as PropertyColumn, Entity as PropertyEntity},
    },
};
use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::Expr,
};
use std::collections::{BTreeMap, HashMap, HashSet};

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

        let tenants_count = tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .count(&self.database)
            .await
            .map_err(persistence_error)?;

        let unit_status_rows = unit::Entity::find()
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
            .map_err(persistence_error)?;
        let mut units_count = 0_u64;
        let mut available_units_count = 0_u64;
        let statuses = unit_status_rows
            .into_iter()
            .map(|(status, count)| {
                let count = u64::try_from(count).map_err(|error| {
                    tracing::error!(%error, "Dashboard status count is negative");
                    ApplicationError::Persistence
                })?;
                units_count = units_count.checked_add(count).ok_or_else(|| {
                    tracing::error!("Dashboard unit count exceeds the supported range");
                    ApplicationError::Persistence
                })?;
                if status == "available" {
                    available_units_count = count;
                }
                Ok(StatusCount { status, count })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;

        let payment_status_rows = payment::Entity::find()
            .select_only()
            .column(payment::Column::Status)
            .column_as(Expr::col(payment::Column::Id).count(), "count")
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .group_by(payment::Column::Status)
            .order_by_asc(payment::Column::Status)
            .into_tuple::<(String, i64)>()
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut posted_payments_count = 0_u64;
        let payment_statuses = payment_status_rows
            .into_iter()
            .map(|(status, count)| {
                let count = u64::try_from(count).map_err(|error| {
                    tracing::error!(%error, "Dashboard payment status count is negative");
                    ApplicationError::Persistence
                })?;
                if status == "posted" {
                    posted_payments_count = count;
                }
                Ok(StatusCount { status, count })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;

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
        let mut currency_totals = BTreeMap::<String, (i64, u64, i16)>::new();
        for payment in &monthly_payments {
            let total = currency_totals
                .entry(payment.currency_code.clone())
                .or_insert((0, 0, payment.currency_decimal_digits));
            total.0 = total.0.checked_add(payment.amount_minor).ok_or_else(|| {
                tracing::error!(
                    currency_code = %payment.currency_code,
                    "Dashboard currency total exceeds the supported integer range"
                );
                ApplicationError::Persistence
            })?;
            total.1 = total
                .1
                .checked_add(payment.payment_count)
                .ok_or(ApplicationError::Persistence)?;
        }
        let payments_by_currency = currency_totals
            .into_iter()
            .map(
                |(currency_code, (amount_minor, payment_count, currency_decimal_digits))| {
                    PaymentCurrencyTotal {
                        currency_code,
                        currency_decimal_digits,
                        amount_minor,
                        payment_count,
                    }
                },
            )
            .collect();

        Ok(DashboardSummary {
            active_properties_count,
            units_count,
            available_units_count,
            tenants_count,
            posted_payments_count,
            payment_months: month_ranges.into_iter().map(|month| month.label).collect(),
            monthly_payments,
            payments_by_currency,
            unit_status_counts: statuses,
            payment_status_counts: payment_statuses,
        })
    }

    async fn property_summary(
        &self,
        organization_id: &str,
        property_id: &str,
        include_payments: bool,
    ) -> Result<PropertyDashboardSummary, ApplicationError> {
        let property = PropertyEntity::find_by_id(property_id)
            .filter(PropertyColumn::OrganizationId.eq(organization_id))
            .filter(PropertyColumn::DeletedAt.is_null())
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;

        let contracts = property_dashboard_contract::Entity::find()
            .filter(property_dashboard_contract::Column::OrganizationId.eq(organization_id))
            .filter(property_dashboard_contract::Column::PropertyId.eq(property.id))
            .filter(property_dashboard_contract::Column::Status.eq("active"))
            .order_by_desc(property_dashboard_contract::Column::Id)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        let contract_count = u64::try_from(contracts.len()).map_err(|error| {
            tracing::error!(%error, "Dashboard contract count exceeds the supported range");
            ApplicationError::Persistence
        })?;
        let tenant_count = u64::try_from(
            contracts
                .iter()
                .map(|contract| contract.tenant_id.as_str())
                .collect::<HashSet<_>>()
                .len(),
        )
        .map_err(|error| {
            tracing::error!(%error, "Dashboard tenant count exceeds the supported range");
            ApplicationError::Persistence
        })?;

        let mut rent_totals = BTreeMap::<String, (i64, i16)>::new();
        for contract in &contracts {
            let total = rent_totals
                .entry(contract.currency_code.clone())
                .or_insert((0, contract.currency_decimal_digits));
            total.0 =
                add_dashboard_amount(total.0, contract.rent_amount_minor, &contract.currency_code)?;
        }
        let rent_by_currency = to_currency_amounts(rent_totals);

        let posted_allocations_by_currency = if include_payments {
            let contract_ids = contracts
                .iter()
                .map(|contract| contract.id.clone())
                .collect::<Vec<_>>();
            if contract_ids.is_empty() {
                Some(Vec::new())
            } else {
                let allocations = payment_allocation::Entity::find()
                    .filter(payment_allocation::Column::OrganizationId.eq(organization_id))
                    .filter(
                        payment_allocation::Column::ContractDocumentId.is_in(contract_ids.clone()),
                    )
                    .all(&self.database)
                    .await
                    .map_err(persistence_error)?;
                let payment_ids = allocations
                    .iter()
                    .map(|allocation| allocation.payment_id.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                if payment_ids.is_empty() {
                    Some(Vec::new())
                } else {
                    let posted_payments = payment::Entity::find()
                        .select_only()
                        .column(payment::Column::Id)
                        .filter(payment::Column::OrganizationId.eq(organization_id))
                        .filter(payment::Column::Status.eq("posted"))
                        .filter(payment::Column::Id.is_in(payment_ids))
                        .into_tuple::<String>()
                        .all(&self.database)
                        .await
                        .map_err(persistence_error)?
                        .into_iter()
                        .collect::<HashSet<_>>();
                    let contract_currencies = contracts
                        .iter()
                        .map(|contract| {
                            (
                                contract.id.as_str(),
                                (
                                    contract.currency_code.as_str(),
                                    contract.currency_decimal_digits,
                                ),
                            )
                        })
                        .collect::<HashMap<_, _>>();
                    let mut payment_totals = BTreeMap::<String, (i64, i16)>::new();
                    for allocation in allocations
                        .iter()
                        .filter(|allocation| posted_payments.contains(&allocation.payment_id))
                    {
                        let (currency_code, decimal_digits) = contract_currencies
                            .get(allocation.contract_document_id.as_str())
                            .copied()
                            .ok_or_else(|| {
                                tracing::error!(
                                    contract_id = %allocation.contract_document_id,
                                    "Payment allocation references a contract outside the property summary"
                                );
                                ApplicationError::Persistence
                            })?;
                        let total = payment_totals
                            .entry(currency_code.to_string())
                            .or_insert((0, decimal_digits));
                        total.0 =
                            add_dashboard_amount(total.0, allocation.amount_minor, currency_code)?;
                    }
                    Some(to_currency_amounts(payment_totals))
                }
            }
        } else {
            None
        };

        Ok(PropertyDashboardSummary {
            contract_count,
            tenant_count,
            rent_by_currency,
            posted_allocations_by_currency,
        })
    }
}

fn add_dashboard_amount(
    total: i64,
    amount: i64,
    currency_code: &str,
) -> Result<i64, ApplicationError> {
    total.checked_add(amount).ok_or_else(|| {
        tracing::error!(
            %currency_code,
            "Property dashboard amount exceeds the supported integer range"
        );
        ApplicationError::Persistence
    })
}

fn to_currency_amounts(totals: BTreeMap<String, (i64, i16)>) -> Vec<CurrencyAmountTotal> {
    totals
        .into_iter()
        .map(
            |(currency_code, (amount_minor, currency_decimal_digits))| CurrencyAmountTotal {
                currency_code,
                currency_decimal_digits,
                amount_minor,
            },
        )
        .collect()
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
