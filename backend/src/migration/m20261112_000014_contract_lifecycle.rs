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
                "ALTER TABLE contract_documents ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'amended', 'terminated', 'cancelled'))",
                "ALTER TABLE contract_documents ADD COLUMN status_reason TEXT",
                "ALTER TABLE contract_documents ADD COLUMN status_effective_date TEXT",
                "ALTER TABLE contract_documents ADD COLUMN parent_contract_id TEXT REFERENCES contract_documents(id) ON DELETE RESTRICT",
                "CREATE INDEX IF NOT EXISTS idx_contract_documents_org_status ON contract_documents (organization_id, status, created_at DESC)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract lifecycle migration is irreversible to preserve the contract history."
                .to_string(),
        ))
    }
}
