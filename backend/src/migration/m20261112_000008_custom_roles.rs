use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        connection
            .execute_unprepared(
                "ALTER TABLE organization_role_permissions RENAME TO organization_role_permissions_legacy",
            )
            .await?;
        execute_statements(
            manager,
            &[
                r#"CREATE TABLE IF NOT EXISTS organization_roles (
                    organization_id TEXT NOT NULL,
                    role_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    is_system BOOLEAN NOT NULL DEFAULT FALSE,
                    system_role TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (organization_id, role_id),
                    UNIQUE (organization_id, normalized_name),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
                    CHECK (length(trim(name)) BETWEEN 1 AND 80),
                    CHECK (length(trim(normalized_name)) BETWEEN 1 AND 80),
                    CHECK (
                        (is_system = TRUE AND system_role IN ('admin', 'staff', 'readonly'))
                        OR (is_system = FALSE AND system_role IS NULL)
                    )
                )"#,
                r#"CREATE TABLE IF NOT EXISTS organization_role_permissions (
                    organization_id TEXT NOT NULL,
                    role_id TEXT NOT NULL,
                    permission_code TEXT NOT NULL,
                    is_enabled BOOLEAN NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_by TEXT,
                    PRIMARY KEY (organization_id, role_id, permission_code),
                    FOREIGN KEY (organization_id, role_id) REFERENCES organization_roles(organization_id, role_id) ON DELETE CASCADE,
                    CHECK (permission_code IN (
                        'manage_properties',
                        'manage_owners',
                        'manage_units',
                        'manage_tenants',
                        'manage_payments'
                    ))
                )"#,
                "ALTER TABLE users ADD COLUMN role_id TEXT NOT NULL DEFAULT 'staff'",
                "UPDATE users SET role_id = access_role",
                "CREATE INDEX IF NOT EXISTS idx_platform_users_role_id ON users (organization_id, role_id)",
                "INSERT INTO organization_roles (organization_id, role_id, name, normalized_name, is_system, system_role) SELECT id, 'admin', 'admin', 'admin', TRUE, 'admin' FROM organizations",
                "INSERT INTO organization_roles (organization_id, role_id, name, normalized_name, is_system, system_role) SELECT id, 'staff', 'staff', 'staff', TRUE, 'staff' FROM organizations",
                "INSERT INTO organization_roles (organization_id, role_id, name, normalized_name, is_system, system_role) SELECT id, 'readonly', 'readonly', 'readonly', TRUE, 'readonly' FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT organization_id, access_role, permission_code, is_enabled FROM organization_role_permissions_legacy",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'manage_properties', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'manage_owners', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'manage_units', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'manage_tenants', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'admin', 'manage_payments', TRUE FROM organizations",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'manage_properties', TRUE FROM organizations WHERE NOT EXISTS (SELECT 1 FROM organization_role_permissions p WHERE p.organization_id = organizations.id AND p.role_id = 'staff' AND p.permission_code = 'manage_properties')",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'manage_owners', TRUE FROM organizations WHERE NOT EXISTS (SELECT 1 FROM organization_role_permissions p WHERE p.organization_id = organizations.id AND p.role_id = 'staff' AND p.permission_code = 'manage_owners')",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'manage_units', TRUE FROM organizations WHERE NOT EXISTS (SELECT 1 FROM organization_role_permissions p WHERE p.organization_id = organizations.id AND p.role_id = 'staff' AND p.permission_code = 'manage_units')",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'manage_tenants', TRUE FROM organizations WHERE NOT EXISTS (SELECT 1 FROM organization_role_permissions p WHERE p.organization_id = organizations.id AND p.role_id = 'staff' AND p.permission_code = 'manage_tenants')",
                "INSERT INTO organization_role_permissions (organization_id, role_id, permission_code, is_enabled) SELECT id, 'staff', 'manage_payments', TRUE FROM organizations WHERE NOT EXISTS (SELECT 1 FROM organization_role_permissions p WHERE p.organization_id = organizations.id AND p.role_id = 'staff' AND p.permission_code = 'manage_payments')",
                "DROP TABLE organization_role_permissions_legacy",
            ],
        )
        .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Custom role migration is irreversible to protect access configuration.".to_string(),
        ))
    }
}
