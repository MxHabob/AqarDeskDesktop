use sea_orm::{ConnectionTrait, DatabaseConnection};
use sea_orm_migration::prelude::*;

mod m20261111_000001_core_identity;
mod m20261111_000002_property_management;
mod m20261111_000003_leasing_and_payments;
mod m20261111_000004_multi_currency;
mod m20261111_000005_contract_templates;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261111_000001_core_identity::Migration),
            Box::new(m20261111_000002_property_management::Migration),
            Box::new(m20261111_000003_leasing_and_payments::Migration),
            Box::new(m20261111_000004_multi_currency::Migration),
            Box::new(m20261111_000005_contract_templates::Migration),
        ]
    }
}

async fn execute_statements(manager: &SchemaManager<'_>, statements: &[&str]) -> Result<(), DbErr> {
    for statement in statements {
        manager
            .get_connection()
            .execute_unprepared(statement)
            .await?;
    }
    Ok(())
}

pub async fn verify_schema(database: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(database);
    for table in [
        "organizations",
        "currencies",
        "organization_currencies",
        "exchange_rates",
        "users",
        "user_sessions",
        "properties",
        "property_units",
        "tenants",
        "leases",
        "lease_charges",
        "payments",
        "payment_allocations",
        "audit_events",
        "contract_templates",
        "contract_template_versions",
        "lease_contract_documents",
    ] {
        if !manager.has_table(table).await? {
            return Err(DbErr::Custom(format!(
                "Required AqarDesk table is missing: {table}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DbBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    #[tokio::test]
    async fn sqlite_schema_is_repeatable_and_enforces_core_constraints() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        Migrator::up(&database, None)
            .await
            .expect("apply migrations");
        Migrator::up(&database, None)
            .await
            .expect("reapply migrations");
        verify_schema(&database).await.expect("verify tables");
        Migrator::down(&database, Some(2))
            .await
            .expect("roll back currency and contract migrations");
        Migrator::up(&database, Some(2))
            .await
            .expect("reapply currency and contract migrations");
        verify_schema(&database)
            .await
            .expect("verify tables after rollback and reapply");

        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organizations (id, name, default_currency, timezone) VALUES ('org-1', 'AqarDesk', 'SAR', 'Asia/Riyadh')",
            ))
            .await
            .expect("insert organization");

        let invalid_role = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO users (id, organization_id, full_name, email, password_hash, access_role) VALUES ('bad-user', 'org-1', 'User', 'bad@example.com', 'a sufficiently long password hash', 'owner')",
            ))
            .await;
        assert!(invalid_role.is_err(), "invalid user roles must be rejected");

        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO users (id, organization_id, full_name, email, password_hash, access_role) VALUES ('user-1', 'org-1', 'Admin', 'admin@example.com', 'a sufficiently long password hash', 'admin')",
            ))
            .await
            .expect("insert valid user");
        for currency in ["SAR", "USD"] {
            database
                .execute(Statement::from_string(
                    DbBackend::Sqlite,
                    format!(
                        "INSERT INTO organization_currencies (organization_id, currency_code) VALUES ('org-1', '{currency}')"
                    ),
                ))
                .await
                .expect("enable organization currency");
        }
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO exchange_rates (id, organization_id, base_currency_code, quote_currency_code, rate, effective_at) VALUES ('rate-1', 'org-1', 'USD', 'SAR', 3.75, '2026-01-01T00:00:00Z')",
            ))
            .await
            .expect("insert exchange rate");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO properties (id, organization_id, code, name, property_type, address_line_1, city, country_code) VALUES ('property-1', 'org-1', 'HQ', 'Building', 'residential', 'Street 1', 'Riyadh', 'SA')",
            ))
            .await
            .expect("insert property");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO property_units (id, organization_id, property_id, unit_number, unit_type) VALUES ('unit-1', 'org-1', 'property-1', 'A1', 'apartment')",
            ))
            .await
            .expect("insert property unit");
        for (tenant_id, email) in [
            ("tenant-1", "one@example.com"),
            ("tenant-2", "two@example.com"),
        ] {
            database
                .execute(Statement::from_string(
                    DbBackend::Sqlite,
                    format!(
                        "INSERT INTO tenants (id, organization_id, full_name, email) VALUES ('{tenant_id}', 'org-1', 'Tenant', '{email}')"
                    ),
                ))
                .await
                .expect("insert tenant");
        }

        let missing_tenant = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO leases (id, organization_id, lease_number, unit_id, tenant_id, start_on, end_on, rent_amount_minor, deposit_amount_minor, currency_code, billing_period, due_day, status) VALUES ('lease-bad', 'org-1', 'L-1', 'unit-1', 'missing', '2026-01-01', '2027-01-01', 100, 0, 'SAR', 'monthly', 1, 'draft')",
            ))
            .await;
        assert!(
            missing_tenant.is_err(),
            "leases must reject missing referenced tenants"
        );

        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO leases (id, organization_id, lease_number, unit_id, tenant_id, start_on, end_on, rent_amount_minor, deposit_amount_minor, currency_code, billing_period, due_day, status, signed_at) VALUES ('lease-1', 'org-1', 'L-2', 'unit-1', 'tenant-1', '2026-01-01', '2027-01-01', 10000, 0, 'SAR', 'monthly', 1, 'active', CURRENT_TIMESTAMP)",
            ))
            .await
            .expect("insert active lease");
        let overlapping_active_lease = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO leases (id, organization_id, lease_number, unit_id, tenant_id, start_on, end_on, rent_amount_minor, deposit_amount_minor, currency_code, billing_period, due_day, status, signed_at) VALUES ('lease-2', 'org-1', 'L-3', 'unit-1', 'tenant-2', '2026-06-01', '2027-06-01', 10000, 0, 'SAR', 'monthly', 1, 'active', CURRENT_TIMESTAMP)",
            ))
            .await;
        assert!(
            overlapping_active_lease.is_err(),
            "a unit cannot have two active leases"
        );

        let mismatched_currency_charge = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO lease_charges (id, organization_id, lease_id, tenant_id, charge_type, period_start, period_end, due_on, amount_minor, currency_code) VALUES ('charge-bad', 'org-1', 'lease-1', 'tenant-1', 'rent', '2026-01-01', '2026-01-31', '2026-01-01', 10000, 'USD')",
            ))
            .await;
        assert!(
            mismatched_currency_charge.is_err(),
            "a charge must use its lease currency"
        );

        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO lease_charges (id, organization_id, lease_id, tenant_id, charge_type, period_start, period_end, due_on, amount_minor, currency_code) VALUES ('charge-1', 'org-1', 'lease-1', 'tenant-1', 'rent', '2026-01-01', '2026-01-31', '2026-01-01', 18750, 'SAR')",
            ))
            .await
            .expect("insert lease charge");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO payments (id, organization_id, tenant_id, received_at, amount_minor, currency_code, payment_method) VALUES ('payment-1', 'org-1', 'tenant-1', '2026-01-02T00:00:00Z', 5000, 'USD', 'bank_transfer')",
            ))
            .await
            .expect("insert payment");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO payment_allocations (id, organization_id, payment_id, lease_charge_id, tenant_id, payment_currency_code, charge_currency_code, amount_in_payment_minor, amount_in_charge_minor, fx_rate, exchange_rate_id) VALUES ('allocation-1', 'org-1', 'payment-1', 'charge-1', 'tenant-1', 'USD', 'SAR', 5000, 18750, 3.75, 'rate-1')",
            ))
            .await
            .expect("allocate payment across currencies");

        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO contract_templates (id, organization_id, template_key, name, lease_type, locale) VALUES ('template-1', 'org-1', 'residential-default', 'Residential lease', 'residential', 'ar-SA')",
            ))
            .await
            .expect("insert contract template");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO contract_template_versions (id, organization_id, template_id, version_number, content_format, content_body, placeholders_json, content_sha256) VALUES ('template-version-1', 'org-1', 'template-1', 1, 'markdown', '# Lease', '{}', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa')",
            ))
            .await
            .expect("insert immutable contract template version");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO lease_contract_documents (id, organization_id, lease_id, template_version_id, document_type, storage_key, file_name, mime_type, content_sha256, size_bytes) VALUES ('document-1', 'org-1', 'lease-1', 'template-version-1', 'contract', 'org-1/lease-1/contract.pdf', 'lease.pdf', 'application/pdf', 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', 128)",
            ))
            .await
            .expect("store generated lease document");
    }
}
