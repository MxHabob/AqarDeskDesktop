use crate::models::{
    currency, lease, lease_charge, lease_contract_document, payment, payment_allocation, property,
    tenant, unit,
};
use chrono::{Datelike, Months, TimeZone, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct MonthlyPayment {
    pub month: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub payment_count: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct OpenChargesByCurrency {
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub charge_count: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct DashboardSummary {
    pub active_properties_count: u64,
    pub units_count: u64,
    pub available_units_count: u64,
    pub tenants_count: u64,
    pub active_leases_count: u64,
    pub open_charges_count: u64,
    pub posted_payments_count: u64,
    pub contract_documents_count: u64,
    pub expiring_leases_count: u64,
    pub payment_months: Vec<String>,
    pub monthly_payments: Vec<MonthlyPayment>,
    pub unit_status_counts: Vec<StatusCount>,
    pub lease_status_counts: Vec<StatusCount>,
    pub open_charges_by_currency: Vec<OpenChargesByCurrency>,
}

pub async fn get_summary(
    database: &DatabaseConnection,
    organization_id: &str,
    months: u32,
) -> Result<DashboardSummary, String> {
    if !matches!(months, 3 | 6 | 12) {
        return Err("Dashboard period must be 3, 6, or 12 months.".to_string());
    }

    let now = Utc::now();
    let first_of_current_month = now
        .date_naive()
        .with_day(1)
        .ok_or_else(|| "Could not determine the current dashboard period.".to_string())?;
    let first_month = first_of_current_month
        .checked_sub_months(Months::new(months - 1))
        .ok_or_else(|| "Could not determine the dashboard date range.".to_string())?;
    let start_at = Utc.from_utc_datetime(
        &first_month
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "Could not determine the dashboard start time.".to_string())?,
    );
    let payment_months = (0..months)
        .map(|offset| {
            first_month
                .checked_add_months(Months::new(offset))
                .map(|date| date.format("%Y-%m").to_string())
                .ok_or_else(|| "Could not determine a dashboard month.".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expires_before = now
        .date_naive()
        .checked_add_months(Months::new(months))
        .ok_or_else(|| "Could not determine the lease expiry range.".to_string())?;

    let (
        active_properties_count,
        units_count,
        available_units_count,
        tenants_count,
        active_leases_count,
        posted_payments_count,
        contract_documents_count,
        expiring_leases_count,
        unit_statuses,
        lease_statuses,
        payments,
        open_charges,
    ) = tokio::try_join!(
        property::Entity::find()
            .filter(property::Column::OrganizationId.eq(organization_id))
            .filter(property::Column::Status.eq("active"))
            .filter(property::Column::DeletedAt.is_null())
            .count(database),
        unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .count(database),
        unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::Status.eq("available"))
            .filter(unit::Column::DeletedAt.is_null())
            .count(database),
        tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .count(database),
        lease::Entity::find()
            .filter(lease::Column::OrganizationId.eq(organization_id))
            .filter(lease::Column::Status.eq("active"))
            .filter(lease::Column::DeletedAt.is_null())
            .count(database),
        payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .filter(payment::Column::Status.eq("posted"))
            .count(database),
        lease_contract_document::Entity::find()
            .filter(lease_contract_document::Column::OrganizationId.eq(organization_id))
            .count(database),
        lease::Entity::find()
            .filter(lease::Column::OrganizationId.eq(organization_id))
            .filter(lease::Column::Status.eq("active"))
            .filter(lease::Column::DeletedAt.is_null())
            .filter(lease::Column::EndOn.gte(now.date_naive()))
            .filter(lease::Column::EndOn.lte(expires_before))
            .count(database),
        unit::Entity::find()
            .select_only()
            .column(unit::Column::Status)
            .column_as(Expr::col(unit::Column::Id).count(), "count")
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .group_by(unit::Column::Status)
            .into_tuple::<(String, i64)>()
            .all(database),
        lease::Entity::find()
            .select_only()
            .column(lease::Column::Status)
            .column_as(Expr::col(lease::Column::Id).count(), "count")
            .filter(lease::Column::OrganizationId.eq(organization_id))
            .filter(lease::Column::DeletedAt.is_null())
            .group_by(lease::Column::Status)
            .into_tuple::<(String, i64)>()
            .all(database),
        payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .filter(payment::Column::Status.eq("posted"))
            .filter(payment::Column::ReceivedAt.gte(start_at))
            .filter(payment::Column::ReceivedAt.lte(now))
            .all(database),
        lease_charge::Entity::find()
            .filter(lease_charge::Column::OrganizationId.eq(organization_id))
            .filter(lease_charge::Column::Status.is_in(["scheduled", "due", "partially_paid"]))
            .all(database),
    )
    .map_err(|error| format!("Failed to load dashboard statistics: {error}"))?;

    let unit_status_counts = status_counts(unit_statuses)?;
    let lease_status_counts = status_counts(lease_statuses)?;

    let mut monthly_amounts = BTreeMap::<(String, String), (i64, u64)>::new();
    let mut currency_codes = BTreeSet::new();
    for model in payments {
        let month = model.received_at.format("%Y-%m").to_string();
        currency_codes.insert(model.currency_code.clone());
        let entry = monthly_amounts
            .entry((month, model.currency_code))
            .or_default();
        entry.0 = entry
            .0
            .checked_add(model.amount_minor)
            .ok_or_else(|| "Monthly payment total exceeds the supported range.".to_string())?;
        entry.1 += 1;
    }

    let charge_ids: Vec<String> = open_charges
        .iter()
        .map(|charge| charge.id.clone())
        .collect();
    let allocations = if charge_ids.is_empty() {
        Vec::new()
    } else {
        payment_allocation::Entity::find()
            .filter(payment_allocation::Column::OrganizationId.eq(organization_id))
            .filter(payment_allocation::Column::LeaseChargeId.is_in(charge_ids))
            .all(database)
            .await
            .map_err(|error| format!("Failed to load open charge allocations: {error}"))?
    };
    let payment_ids: Vec<String> = allocations
        .iter()
        .map(|allocation| allocation.payment_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let posted_payment_ids = if payment_ids.is_empty() {
        HashSet::new()
    } else {
        payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .filter(payment::Column::Status.eq("posted"))
            .filter(payment::Column::Id.is_in(payment_ids))
            .all(database)
            .await
            .map_err(|error| format!("Failed to verify open charge payments: {error}"))?
            .into_iter()
            .map(|model| model.id)
            .collect()
    };
    let mut paid_by_charge = HashMap::<String, i64>::new();
    for allocation in allocations {
        if !posted_payment_ids.contains(&allocation.payment_id) {
            continue;
        }
        let paid = paid_by_charge
            .entry(allocation.lease_charge_id)
            .or_default();
        *paid = paid
            .checked_add(allocation.amount_in_charge_minor)
            .ok_or_else(|| "Charge allocation total exceeds the supported range.".to_string())?;
    }

    let mut open_charges_by_currency = BTreeMap::<String, (i64, u64)>::new();
    let mut open_charges_count = 0;
    for charge in open_charges {
        let allocated_amount = paid_by_charge.get(&charge.id).copied().unwrap_or_default();
        let remaining = charge
            .amount_minor
            .checked_sub(allocated_amount)
            .ok_or_else(|| format!("Charge {} balance exceeds the supported range.", charge.id))?;
        if remaining < 0 {
            return Err(format!("Charge {} is over-allocated.", charge.id));
        }
        if remaining == 0 {
            continue;
        }
        open_charges_count += 1;
        currency_codes.insert(charge.currency_code.clone());
        let entry = open_charges_by_currency
            .entry(charge.currency_code)
            .or_default();
        entry.0 = entry
            .0
            .checked_add(remaining)
            .ok_or_else(|| "Open charge total exceeds the supported range.".to_string())?;
        entry.1 += 1;
    }

    let currencies = if currency_codes.is_empty() {
        Vec::new()
    } else {
        currency::Entity::find()
            .filter(currency::Column::Code.is_in(currency_codes))
            .all(database)
            .await
            .map_err(|error| format!("Failed to load dashboard currencies: {error}"))?
    };
    let decimal_digits: HashMap<String, i16> = currencies
        .into_iter()
        .map(|model| (model.code, model.decimal_digits))
        .collect();
    let monthly_payments = monthly_amounts
        .into_iter()
        .map(|((month, currency_code), (amount_minor, payment_count))| {
            Ok(MonthlyPayment {
                month,
                currency_decimal_digits: decimal_digits.get(&currency_code).copied().ok_or_else(
                    || format!("Currency {currency_code} is missing from the currency catalog."),
                )?,
                currency_code,
                amount_minor,
                payment_count,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let open_charges_by_currency = open_charges_by_currency
        .into_iter()
        .map(|(currency_code, (amount_minor, charge_count))| {
            Ok(OpenChargesByCurrency {
                currency_decimal_digits: decimal_digits.get(&currency_code).copied().ok_or_else(
                    || format!("Currency {currency_code} is missing from the currency catalog."),
                )?,
                currency_code,
                amount_minor,
                charge_count,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(DashboardSummary {
        active_properties_count,
        units_count,
        available_units_count,
        tenants_count,
        active_leases_count,
        open_charges_count,
        posted_payments_count,
        contract_documents_count,
        expiring_leases_count,
        payment_months,
        monthly_payments,
        unit_status_counts,
        lease_status_counts,
        open_charges_by_currency,
    })
}

fn status_counts(rows: Vec<(String, i64)>) -> Result<Vec<StatusCount>, String> {
    rows.into_iter()
        .map(|(status, count)| {
            Ok(StatusCount {
                status,
                count: u64::try_from(count)
                    .map_err(|error| format!("Invalid dashboard status count: {error}"))?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{run_migrations, users};
    use sea_orm::{ConnectionTrait, Database};

    #[tokio::test]
    async fn summary_counts_active_records_only_for_the_requested_organization() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory database");
        run_migrations(&database).await.expect("apply migrations");
        let admin = users::create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create administrator");
        let payment_time = Utc::now();

        let statements = [
            "INSERT INTO organizations (id, name, default_currency, timezone) VALUES ('other-org', 'Other', 'SAR', 'Asia/Riyadh')".to_string(),
            format!(
                "INSERT INTO properties (id, organization_id, code, name, property_type, address_line_1, city, country_code, status) VALUES
                    ('active-property', '{}', 'ACTIVE', 'Active', 'residential', 'Street 1', 'Riyadh', 'SA', 'active'),
                    ('inactive-property', '{}', 'INACTIVE', 'Inactive', 'residential', 'Street 2', 'Riyadh', 'SA', 'inactive'),
                    ('other-property', 'other-org', 'OTHER', 'Other', 'residential', 'Street 3', 'Riyadh', 'SA', 'active')",
                admin.organization_id, admin.organization_id
            ),
            format!(
                "INSERT INTO property_units (id, organization_id, property_id, unit_number, unit_type, status) VALUES
                    ('available-unit', '{}', 'active-property', 'A1', 'apartment', 'available'),
                    ('maintenance-unit', '{}', 'active-property', 'A2', 'apartment', 'maintenance'),
                    ('other-unit', 'other-org', 'other-property', 'B1', 'apartment', 'available')",
                admin.organization_id, admin.organization_id
            ),
            format!(
                "INSERT INTO tenants (id, organization_id, full_name, email) VALUES
                    ('active-tenant', '{}', 'Tenant', 'tenant@example.com'),
                    ('other-tenant', 'other-org', 'Other Tenant', 'other@example.com')",
                admin.organization_id
            ),
        ];
        for statement in statements {
            database
                .execute_unprepared(&statement)
                .await
                .expect("insert test records");
        }
        database
            .execute_unprepared(&format!(
                "INSERT INTO payments
                    (id, organization_id, tenant_id, received_at, amount_minor, currency_code, payment_method, status)
                    VALUES ('payment-1', '{}', 'active-tenant', '{}', 125000, 'SAR', 'cash', 'posted')",
                admin.organization_id,
                payment_time.to_rfc3339()
            ))
            .await
            .expect("insert posted payment");
        database
            .execute_unprepared(&format!(
                "INSERT INTO leases
                    (id, organization_id, lease_number, unit_id, tenant_id, start_on, end_on,
                     rent_amount_minor, deposit_amount_minor, currency_code, billing_period,
                     due_day, status)
                    VALUES ('draft-lease', '{}', 'L-1', 'available-unit', 'active-tenant',
                            '2026-01-01', '2027-01-01', 90000, 0, 'SAR', 'monthly', 1, 'draft')",
                admin.organization_id
            ))
            .await
            .expect("insert draft lease");
        database
            .execute_unprepared(&format!(
                "INSERT INTO lease_charges
                    (id, organization_id, lease_id, tenant_id, charge_type, period_start, period_end,
                     due_on, amount_minor, currency_code, status)
                    VALUES ('open-charge', '{}', 'draft-lease', 'active-tenant', 'rent',
                            '2026-01-01', '2026-01-31', '2026-02-01', 90000, 'SAR', 'due')",
                admin.organization_id
            ))
            .await
            .expect("insert open charge");

        let summary = get_summary(&database, &admin.organization_id, 6)
            .await
            .expect("load organization summary");

        assert_eq!(
            summary,
            DashboardSummary {
                active_properties_count: 1,
                units_count: 2,
                available_units_count: 1,
                tenants_count: 1,
                active_leases_count: 0,
                open_charges_count: 1,
                posted_payments_count: 1,
                contract_documents_count: 0,
                expiring_leases_count: 0,
                payment_months: payment_months_for_test(),
                monthly_payments: vec![MonthlyPayment {
                    month: payment_time.format("%Y-%m").to_string(),
                    currency_code: "SAR".to_string(),
                    currency_decimal_digits: 2,
                    amount_minor: 125000,
                    payment_count: 1,
                }],
                unit_status_counts: vec![
                    StatusCount {
                        status: "available".to_string(),
                        count: 1
                    },
                    StatusCount {
                        status: "maintenance".to_string(),
                        count: 1
                    },
                ],
                lease_status_counts: vec![StatusCount {
                    status: "draft".to_string(),
                    count: 1,
                }],
                open_charges_by_currency: vec![OpenChargesByCurrency {
                    currency_code: "SAR".to_string(),
                    currency_decimal_digits: 2,
                    amount_minor: 90000,
                    charge_count: 1,
                }],
            }
        );
    }

    fn payment_months_for_test() -> Vec<String> {
        let now = Utc::now();
        let first = now.date_naive().with_day(1).expect("first of month");
        (0..6)
            .rev()
            .map(|offset| {
                first
                    .checked_sub_months(Months::new(offset))
                    .expect("valid month")
                    .format("%Y-%m")
                    .to_string()
            })
            .collect()
    }
}
