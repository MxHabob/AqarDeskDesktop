use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::{MigrationTrait, MigratorTrait, SchemaManager, prelude::*};

mod m20261111_000001_platform_core;
mod m20261112_000001_contract_templates;
mod m20261112_000002_onboarding_defaults;
mod m20261112_000003_contract_template_starters;

pub struct PlatformMigrator;

#[async_trait::async_trait]
impl MigratorTrait for PlatformMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261111_000001_platform_core::Migration),
            Box::new(m20261112_000001_contract_templates::Migration),
            Box::new(m20261112_000002_onboarding_defaults::Migration),
            Box::new(m20261112_000003_contract_template_starters::Migration),
        ]
    }
}

pub async fn initialize(database: &DatabaseConnection) -> Result<(), DbErr> {
    PlatformMigrator::up(database, None).await?;
    verify_schema(database).await
}

pub async fn verify_schema(database: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(database);
    for table in [
        "organizations",
        "currencies",
        "users",
        "organization_currencies",
        "user_sessions",
        "properties",
        "property_units",
        "tenants",
        "payments",
        "audit_events",
        "contract_templates",
        "contract_template_versions",
    ] {
        if !manager.has_table(table).await? {
            return Err(DbErr::Custom(format!(
                "Required platform table is missing: {table}"
            )));
        }
    }
    Ok(())
}

async fn execute_statements(manager: &SchemaManager<'_>, statements: &[&str]) -> Result<(), DbErr> {
    for sql in statements {
        manager.get_connection().execute_unprepared(sql).await?;
    }
    Ok(())
}

pub async fn identify_legacy_tables(database: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
    let query = match database.get_database_backend() {
        DbBackend::Sqlite => {
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        }
        DbBackend::Postgres => {
            "SELECT table_name AS name FROM information_schema.tables WHERE table_schema = current_schema() ORDER BY table_name"
        }
        backend => {
            return Err(DbErr::Custom(format!(
                "Unsupported database backend for import: {backend:?}"
            )));
        }
    };
    let rows = database
        .query_all(Statement::from_string(
            database.get_database_backend(),
            query,
        ))
        .await?;
    rows.into_iter()
        .map(|row| row.try_get("", "name"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

    #[tokio::test]
    async fn fresh_database_migration_is_repeatable_and_keeps_existing_schema() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        initialize(&database).await.expect("initialize platform");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organizations (id, name, default_currency, timezone) VALUES ('preserved-org', 'Existing customer', 'SAR', 'Asia/Riyadh')",
            ))
            .await
            .expect("insert existing customer data");
        initialize(&database)
            .await
            .expect("reapply platform migrations");
        let preserved = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT name FROM organizations WHERE id = 'preserved-org'",
            ))
            .await
            .expect("read preserved organization")
            .expect("organization remains after migration")
            .try_get_by::<String, _>("name")
            .expect("read preserved name");
        assert_eq!(preserved, "Existing customer");

        let tables = identify_legacy_tables(&database)
            .await
            .expect("list existing tables");
        for expected in [
            "organizations",
            "properties",
            "property_units",
            "tenants",
            "payments",
            "audit_events",
        ] {
            assert!(tables.iter().any(|table| table == expected));
        }

        let currency_count = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS count FROM currencies",
            ))
            .await
            .expect("count default currencies")
            .expect("currency count exists")
            .try_get_by::<i64, _>("count")
            .expect("read currency count");
        assert_eq!(currency_count, 10);
    }
}
