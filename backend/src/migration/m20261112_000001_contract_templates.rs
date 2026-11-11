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
                r#"CREATE TABLE IF NOT EXISTS contract_templates (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    template_key TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    lease_type TEXT NOT NULL,
                    locale TEXT NOT NULL,
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    created_by TEXT,
                    updated_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    UNIQUE (organization_id, template_key, locale),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (updated_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT
                )"#,
                r#"CREATE TABLE IF NOT EXISTS contract_template_versions (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    template_id TEXT NOT NULL,
                    version_number INTEGER NOT NULL CHECK (version_number > 0),
                    content_format TEXT NOT NULL,
                    content_body TEXT,
                    placeholders_json TEXT NOT NULL DEFAULT '{}',
                    content_sha256 TEXT NOT NULL,
                    change_note TEXT,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (template_id, version_number),
                    FOREIGN KEY (template_id, organization_id) REFERENCES contract_templates(id, organization_id) ON DELETE CASCADE,
                    FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT,
                    CHECK (content_format IN ('text', 'markdown', 'html'))
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_contract_templates_org_active ON contract_templates (organization_id, is_active, name)",
                "CREATE INDEX IF NOT EXISTS idx_contract_template_versions_template ON contract_template_versions (template_id, version_number DESC)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract template migration is irreversible to protect business data.".to_string(),
        ))
    }
}
