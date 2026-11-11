use sea_orm::{ConnectionTrait, DatabaseConnection};
use sea_orm_migration::prelude::*;

mod m20261111_000001_core_identity;
mod m20261111_000002_property_management;
mod m20261111_000003_leasing_and_payments;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261111_000001_core_identity::Migration),
            Box::new(m20261111_000002_property_management::Migration),
            Box::new(m20261111_000003_leasing_and_payments::Migration),
        ]
    }
}

async fn execute_statements(
    manager: &SchemaManager<'_>,
    statements: &[&str],
) -> Result<(), DbErr> {
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
    use sea_orm::{Database, Statement, DbBackend};
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

        let invalid_role = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO users (id, organization_id, full_name, email, password_hash, role) VALUES ('u', 'o', 'User', 'u@example.com', 'hash', 'owner')",
            ))
            .await;
        assert!(invalid_role.is_err(), "invalid user roles must be rejected");

        let missing_tenant = database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO leases (id, organization_id, lease_number, unit_id, tenant_id, start_on, end_on, rent_amount_minor, deposit_amount_minor, currency_code, billing_period, due_day, status) VALUES ('l', 'o', 'L-1', 'unit', 'missing', '2026-01-01', '2027-01-01', 100, 0, 'SAR', 'monthly', 1, 'draft')",
            ))
            .await;
        assert!(
            missing_tenant.is_err(),
            "leases must reject missing referenced tenants"
        );
    }
}
