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
                "ALTER TABLE contract_documents ADD COLUMN docforge_document_json TEXT",
                "ALTER TABLE contract_documents ADD COLUMN docforge_context_json TEXT",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract DocForge snapshots are irreversible to protect saved contracts.".to_string(),
        ))
    }
}
