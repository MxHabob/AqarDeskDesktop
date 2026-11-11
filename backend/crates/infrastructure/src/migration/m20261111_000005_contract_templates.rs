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
                r#"CREATE TABLE contract_templates (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    template_key TEXT NOT NULL
                        CHECK (length(trim(template_key)) > 0 AND template_key = lower(trim(template_key))),
                    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
                    description TEXT,
                    lease_type TEXT NOT NULL
                        CHECK (lease_type IN ('residential', 'commercial', 'short_term', 'other')),
                    locale TEXT NOT NULL DEFAULT 'ar'
                        CHECK (length(locale) BETWEEN 2 AND 35),
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    archived_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    UNIQUE (organization_id, template_key),
                    CONSTRAINT fk_contract_templates_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_contract_templates_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_contract_templates_org_type ON contract_templates (organization_id, lease_type, is_active) WHERE archived_at IS NULL",
                r#"CREATE TABLE contract_template_versions (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    template_id TEXT NOT NULL,
                    version_number INTEGER NOT NULL CHECK (version_number > 0),
                    content_format TEXT NOT NULL
                        CHECK (content_format IN ('text', 'markdown', 'html', 'docx')),
                    content_body TEXT,
                    source_storage_key TEXT,
                    placeholders_json TEXT NOT NULL DEFAULT '{}',
                    content_sha256 VARCHAR(64) NOT NULL
                        CHECK (length(content_sha256) = 64),
                    change_note TEXT,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    UNIQUE (template_id, version_number),
                    CONSTRAINT ck_contract_template_version_content CHECK (
                        (content_format = 'docx' AND source_storage_key IS NOT NULL AND content_body IS NULL) OR
                        (content_format <> 'docx' AND content_body IS NOT NULL AND source_storage_key IS NULL)
                    ),
                    CONSTRAINT fk_contract_template_versions_template
                        FOREIGN KEY (template_id, organization_id)
                        REFERENCES contract_templates(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_contract_template_versions_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_contract_template_versions_latest ON contract_template_versions (template_id, version_number)",
                r#"CREATE TABLE lease_contract_documents (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    lease_id TEXT NOT NULL,
                    template_version_id TEXT,
                    document_type TEXT NOT NULL
                        CHECK (document_type IN ('contract', 'amendment', 'notice', 'attachment')),
                    storage_key TEXT NOT NULL CHECK (length(trim(storage_key)) > 0),
                    file_name TEXT NOT NULL CHECK (length(trim(file_name)) > 0),
                    mime_type TEXT NOT NULL,
                    content_sha256 VARCHAR(64) NOT NULL
                        CHECK (length(content_sha256) = 64),
                    size_bytes BIGINT NOT NULL CHECK (size_bytes > 0),
                    is_signed BOOLEAN NOT NULL DEFAULT FALSE,
                    signed_at TIMESTAMPTZ,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (organization_id, storage_key),
                    CONSTRAINT ck_lease_document_signature
                        CHECK ((is_signed AND signed_at IS NOT NULL) OR (NOT is_signed AND signed_at IS NULL)),
                    CONSTRAINT fk_lease_contract_documents_lease
                        FOREIGN KEY (lease_id, organization_id) REFERENCES leases(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_lease_contract_documents_template_version
                        FOREIGN KEY (template_version_id, organization_id)
                        REFERENCES contract_template_versions(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_lease_contract_documents_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_lease_contract_documents_lease ON lease_contract_documents (lease_id, created_at)",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "DROP TABLE lease_contract_documents",
                "DROP TABLE contract_template_versions",
                "DROP TABLE contract_templates",
            ],
        )
        .await
    }
}
