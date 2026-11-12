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
                r#"CREATE TABLE IF NOT EXISTS organization_role_permissions (
                organization_id TEXT NOT NULL,
                access_role TEXT NOT NULL,
                permission_code TEXT NOT NULL,
                is_enabled BOOLEAN NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_by TEXT,
                PRIMARY KEY (organization_id, access_role, permission_code),
                FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
                CHECK (access_role IN ('staff', 'readonly')),
                CHECK (permission_code IN (
                    'manage_properties',
                    'manage_owners',
                    'manage_units',
                    'manage_tenants',
                    'manage_payments'
                ))
            )"#,
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Role permission migration is irreversible to protect access configuration."
                .to_string(),
        ))
    }
}
