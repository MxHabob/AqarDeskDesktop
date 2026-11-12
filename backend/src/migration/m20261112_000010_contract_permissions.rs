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
                r#"CREATE TABLE organization_role_permissions_next (
                    organization_id TEXT NOT NULL,
                    role_id TEXT NOT NULL,
                    permission_code TEXT NOT NULL,
                    is_enabled BOOLEAN NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_by TEXT,
                    PRIMARY KEY (organization_id, role_id, permission_code),
                    FOREIGN KEY (organization_id, role_id) REFERENCES organization_roles(organization_id, role_id) ON DELETE CASCADE,
                    CHECK (permission_code IN (
                        'view_properties',
                        'create_properties',
                        'update_properties',
                        'archive_properties',
                        'view_owners',
                        'create_owners',
                        'update_owners',
                        'view_units',
                        'create_units',
                        'update_units',
                        'archive_units',
                        'view_tenants',
                        'create_tenants',
                        'update_tenants',
                        'archive_tenants',
                        'view_payments',
                        'record_payments',
                        'void_payments',
                        'view_contracts',
                        'create_contracts'
                    ))
                )"#,
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, permission_code, is_enabled, updated_at, updated_by FROM organization_role_permissions",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'view_contracts', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'create_contracts', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'view_contracts', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'create_contracts', TRUE FROM organizations",
                "DROP TABLE organization_role_permissions",
                "ALTER TABLE organization_role_permissions_next RENAME TO organization_role_permissions",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract permission migration is irreversible to preserve role configuration."
                .to_string(),
        ))
    }
}
