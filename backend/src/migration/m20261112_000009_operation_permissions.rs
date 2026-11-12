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
                        'void_payments'
                    ))
                )"#,
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'view_properties', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_properties'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'create_properties', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_properties'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'update_properties', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_properties'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'archive_properties', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_properties'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'view_owners', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_owners'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'create_owners', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_owners'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'update_owners', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_owners'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'view_units', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_units'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'create_units', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_units'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'update_units', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_units'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'archive_units', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_units'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'view_tenants', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_tenants'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'create_tenants', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_tenants'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'update_tenants', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_tenants'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'archive_tenants', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_tenants'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'view_payments', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_payments'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'record_payments', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_payments'",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT organization_id, role_id, 'void_payments', is_enabled, updated_at, updated_by FROM organization_role_permissions WHERE permission_code = 'manage_payments'",
                "UPDATE organization_role_permissions_next SET is_enabled = TRUE WHERE permission_code = 'view_properties' AND EXISTS (SELECT 1 FROM organization_role_permissions_next operation WHERE operation.organization_id = organization_role_permissions_next.organization_id AND operation.role_id = organization_role_permissions_next.role_id AND operation.permission_code IN ('create_units', 'update_units') AND operation.is_enabled = TRUE)",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT DISTINCT operation.organization_id, operation.role_id, 'view_properties', TRUE, CURRENT_TIMESTAMP, NULL FROM organization_role_permissions_next operation WHERE operation.permission_code IN ('create_units', 'update_units') AND operation.is_enabled = TRUE AND NOT EXISTS (SELECT 1 FROM organization_role_permissions_next existing WHERE existing.organization_id = operation.organization_id AND existing.role_id = operation.role_id AND existing.permission_code = 'view_properties')",
                "UPDATE organization_role_permissions_next SET is_enabled = TRUE WHERE permission_code = 'view_tenants' AND EXISTS (SELECT 1 FROM organization_role_permissions_next operation WHERE operation.organization_id = organization_role_permissions_next.organization_id AND operation.role_id = organization_role_permissions_next.role_id AND operation.permission_code = 'record_payments' AND operation.is_enabled = TRUE)",
                "INSERT INTO organization_role_permissions_next (organization_id, role_id, permission_code, is_enabled, updated_at, updated_by) SELECT DISTINCT operation.organization_id, operation.role_id, 'view_tenants', TRUE, CURRENT_TIMESTAMP, NULL FROM organization_role_permissions_next operation WHERE operation.permission_code = 'record_payments' AND operation.is_enabled = TRUE AND NOT EXISTS (SELECT 1 FROM organization_role_permissions_next existing WHERE existing.organization_id = operation.organization_id AND existing.role_id = operation.role_id AND existing.permission_code = 'view_tenants')",
                "DROP TABLE organization_role_permissions",
                "ALTER TABLE organization_role_permissions_next RENAME TO organization_role_permissions",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Operation permission migration is irreversible to preserve role configuration."
                .to_string(),
        ))
    }
}
