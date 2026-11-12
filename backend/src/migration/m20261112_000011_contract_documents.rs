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
                "CREATE UNIQUE INDEX IF NOT EXISTS uq_contract_template_versions_document_scope ON contract_template_versions (id, organization_id, template_id)",
                "CREATE UNIQUE INDEX IF NOT EXISTS uq_property_units_document_scope ON property_units (id, property_id, organization_id)",
                r#"CREATE TABLE IF NOT EXISTS contract_documents (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    contract_number TEXT NOT NULL,
                    template_id TEXT NOT NULL,
                    template_version_id TEXT NOT NULL,
                    template_name TEXT NOT NULL,
                    template_locale TEXT NOT NULL,
                    property_id TEXT NOT NULL,
                    property_name TEXT NOT NULL,
                    property_code TEXT NOT NULL,
                    unit_id TEXT NOT NULL,
                    unit_number TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    tenant_name TEXT NOT NULL,
                    start_date TEXT NOT NULL,
                    end_date TEXT NOT NULL,
                    currency_code VARCHAR(3) NOT NULL,
                    currency_decimal_digits SMALLINT NOT NULL,
                    rent_amount_minor BIGINT NOT NULL CHECK (rent_amount_minor > 0),
                    deposit_amount_minor BIGINT NOT NULL CHECK (deposit_amount_minor >= 0),
                    document_body TEXT NOT NULL CHECK (length(trim(document_body)) > 0),
                    created_by TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    UNIQUE (organization_id, contract_number),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    FOREIGN KEY (template_id, organization_id) REFERENCES contract_templates(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (template_version_id, organization_id, template_id) REFERENCES contract_template_versions(id, organization_id, template_id) ON DELETE RESTRICT,
                    FOREIGN KEY (property_id, organization_id) REFERENCES properties(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (unit_id, property_id, organization_id) REFERENCES property_units(id, property_id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (tenant_id, organization_id) REFERENCES tenants(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,
                    FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT,
                    CHECK (length(currency_code) = 3 AND currency_code = upper(currency_code)),
                    CHECK (currency_decimal_digits BETWEEN 0 AND 4),
                    CHECK (start_date <= end_date)
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_contract_documents_org_created ON contract_documents (organization_id, created_at DESC)",
                "CREATE INDEX IF NOT EXISTS idx_contract_documents_org_property ON contract_documents (organization_id, property_id, created_at DESC)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract document migration is irreversible to protect saved contracts.".to_string(),
        ))
    }
}
