use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::{MigrationTrait, MigratorTrait, SchemaManager, prelude::*};

mod m20261111_000001_platform_core;
mod m20261112_000001_contract_templates;
mod m20261112_000002_onboarding_defaults;
mod m20261112_000003_contract_template_starters;
mod m20261112_000004_payment_receipt_templates;
mod m20261112_000005_default_lease_templates;
mod m20261112_000006_property_details_and_owners;
mod m20261112_000007_role_permissions;
mod m20261112_000008_custom_roles;
mod m20261112_000009_operation_permissions;
mod m20261112_000010_contract_permissions;
mod m20261112_000011_contract_documents;
mod m20261112_000012_payment_allocations;
mod m20261112_000013_contract_document_design;
mod m20261112_000014_contract_lifecycle;
mod m20261112_000015_organization_logo;
mod m20261112_000016_docforge_default_templates;
mod m20261112_000017_yemen_legal_template_redesign;
mod m20261112_000018_payment_allocation_contract_index;

pub struct PlatformMigrator;

#[async_trait::async_trait]
impl MigratorTrait for PlatformMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261111_000001_platform_core::Migration),
            Box::new(m20261112_000001_contract_templates::Migration),
            Box::new(m20261112_000002_onboarding_defaults::Migration),
            Box::new(m20261112_000003_contract_template_starters::Migration),
            Box::new(m20261112_000004_payment_receipt_templates::Migration),
            Box::new(m20261112_000005_default_lease_templates::Migration),
            Box::new(m20261112_000006_property_details_and_owners::Migration),
            Box::new(m20261112_000007_role_permissions::Migration),
            Box::new(m20261112_000008_custom_roles::Migration),
            Box::new(m20261112_000009_operation_permissions::Migration),
            Box::new(m20261112_000010_contract_permissions::Migration),
            Box::new(m20261112_000011_contract_documents::Migration),
            Box::new(m20261112_000012_payment_allocations::Migration),
            Box::new(m20261112_000014_contract_lifecycle::Migration),
            Box::new(m20261112_000013_contract_document_design::Migration),
            Box::new(m20261112_000015_organization_logo::Migration),
            Box::new(m20261112_000016_docforge_default_templates::Migration),
            Box::new(m20261112_000017_yemen_legal_template_redesign::Migration),
            Box::new(m20261112_000018_payment_allocation_contract_index::Migration),
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
        "platform_installation",
        "organizations",
        "currencies",
        "users",
        "organization_role_permissions",
        "organization_roles",
        "organization_currencies",
        "user_sessions",
        "properties",
        "owners",
        "owner_phones",
        "property_owners",
        "property_units",
        "tenants",
        "payments",
        "audit_events",
        "contract_templates",
        "contract_template_versions",
        "contract_documents",
        "payment_allocations",
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
        let query_plan = database
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "EXPLAIN QUERY PLAN SELECT payment_id FROM payment_allocations WHERE organization_id = 'org' AND contract_document_id = 'contract'",
            ))
            .await
            .expect("explain property payment allocation lookup");
        let plan_details = query_plan
            .iter()
            .map(|row| {
                row.try_get_by::<String, _>("detail")
                    .expect("read query plan detail")
            })
            .collect::<Vec<_>>();
        assert!(
            plan_details
                .iter()
                .any(|detail| { detail.contains("idx_payment_allocations_org_contract_payment") }),
            "property payment lookup should use the contract allocation index: {plan_details:?}"
        );
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
    }

    #[tokio::test]
    async fn unified_role_migration_preserves_existing_users_and_permission_values() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        PlatformMigrator::up(&database, Some(8))
            .await
            .expect("apply migrations through legacy role permissions");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organizations (id, name, default_currency, timezone) VALUES ('legacy-org', 'Legacy organization', 'SAR', 'Asia/Riyadh')",
            ))
            .await
            .expect("insert legacy organization");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO users (id, organization_id, full_name, email, password_hash, access_role) VALUES ('legacy-user', 'legacy-org', 'Legacy user', 'legacy@example.com', 'unused', 'readonly')",
            ))
            .await
            .expect("insert legacy user");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organization_role_permissions (organization_id, access_role, permission_code, is_enabled) VALUES ('legacy-org', 'readonly', 'manage_properties', FALSE), ('legacy-org', 'staff', 'manage_units', TRUE), ('legacy-org', 'staff', 'manage_tenants', FALSE), ('legacy-org', 'staff', 'manage_payments', TRUE)",
            ))
            .await
            .expect("insert legacy role permission");

        initialize(&database)
            .await
            .expect("apply unified role migration");
        let migrated_user = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT role_id FROM users WHERE id = 'legacy-user'",
            ))
            .await
            .expect("query migrated user")
            .expect("legacy user remains")
            .try_get_by::<String, _>("role_id")
            .expect("read migrated user role");
        let migrated_permission = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT is_enabled FROM organization_role_permissions WHERE organization_id = 'legacy-org' AND role_id = 'readonly' AND permission_code = 'create_properties'",
            ))
            .await
            .expect("query migrated role permission")
            .expect("legacy permission remains")
            .try_get_by::<bool, _>("is_enabled")
            .expect("read migrated permission value");
        let migrated_staff_contract_permission = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT is_enabled FROM organization_role_permissions WHERE organization_id = 'legacy-org' AND role_id = 'staff' AND permission_code = 'create_contracts'",
            ))
            .await
            .expect("query migrated contract permission")
            .expect("contract permission exists")
            .try_get_by::<bool, _>("is_enabled")
            .expect("read migrated contract permission");
        let migrated_view_permission = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT is_enabled FROM organization_role_permissions WHERE organization_id = 'legacy-org' AND role_id = 'readonly' AND permission_code = 'view_properties'",
            ))
            .await
            .expect("query migrated view permission")
            .expect("legacy view permission remains")
            .try_get_by::<bool, _>("is_enabled")
            .expect("read migrated view permission value");
        let migrated_unit_view_permission = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT is_enabled FROM organization_role_permissions WHERE organization_id = 'legacy-org' AND role_id = 'staff' AND permission_code = 'view_properties'",
            ))
            .await
            .expect("query migrated unit property view permission")
            .expect("unit property view permission exists")
            .try_get_by::<bool, _>("is_enabled")
            .expect("read migrated unit property view permission");
        let migrated_payment_tenant_view_permission = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT is_enabled FROM organization_role_permissions WHERE organization_id = 'legacy-org' AND role_id = 'staff' AND permission_code = 'view_tenants'",
            ))
            .await
            .expect("query migrated payment tenant view permission")
            .expect("payment tenant view permission exists")
            .try_get_by::<bool, _>("is_enabled")
            .expect("read migrated payment tenant view permission");
        assert_eq!(migrated_user, "readonly");
        assert!(!migrated_permission);
        assert!(!migrated_view_permission);
        assert!(migrated_unit_view_permission);
        assert!(migrated_payment_tenant_view_permission);
        assert!(migrated_staff_contract_permission);

        let tables = identify_legacy_tables(&database)
            .await
            .expect("list existing tables");
        for expected in [
            "organizations",
            "properties",
            "owners",
            "owner_phones",
            "property_owners",
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

    #[tokio::test]
    async fn property_details_migration_preserves_existing_property_rows() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        PlatformMigrator::up(&database, Some(6))
            .await
            .expect("apply migrations before property details");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organizations (id, name, default_currency, timezone) VALUES ('legacy-org', 'Legacy office', 'SAR', 'Asia/Riyadh')",
            ))
            .await
            .expect("insert legacy organization");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO properties (id, organization_id, code, name, property_type, address_line_1, city, country_code) VALUES ('legacy-property', 'legacy-org', 'LEGACY-1', 'Existing property', 'residential', 'Old street', 'Riyadh', 'SA')",
            ))
            .await
            .expect("insert legacy property");

        initialize(&database)
            .await
            .expect("apply property details migration");
        let property = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT name, district, lot_area_square_meters FROM properties WHERE id = 'legacy-property'",
            ))
            .await
            .expect("read migrated property")
            .expect("legacy property remains");
        assert_eq!(
            property
                .try_get_by::<String, _>("name")
                .expect("read property name"),
            "Existing property"
        );
        assert_eq!(
            property
                .try_get_by::<Option<String>, _>("district")
                .expect("read new district field"),
            None
        );
        assert_eq!(
            property
                .try_get_by::<Option<f64>, _>("lot_area_square_meters")
                .expect("read new area field"),
            None
        );
    }
}
