use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "CREATE INDEX IF NOT EXISTS idx_platform_properties_org_name ON properties (organization_id, name, id) WHERE deleted_at IS NULL",
                "CREATE INDEX IF NOT EXISTS idx_platform_units_org_property_number ON property_units (organization_id, property_id, unit_number, id) WHERE deleted_at IS NULL",
                "CREATE INDEX IF NOT EXISTS idx_platform_payments_org_status_date ON payments (organization_id, status, received_at)",
                "DROP INDEX IF EXISTS idx_platform_users_org_role",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "CREATE INDEX IF NOT EXISTS idx_platform_users_org_role ON users (organization_id, access_role, is_active)",
                "DROP INDEX IF EXISTS idx_platform_properties_org_name",
                "DROP INDEX IF EXISTS idx_platform_units_org_property_number",
                "DROP INDEX IF EXISTS idx_platform_payments_org_status_date",
            ],
        )
        .await
    }
}
