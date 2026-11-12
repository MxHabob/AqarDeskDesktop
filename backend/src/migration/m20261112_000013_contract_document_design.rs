use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &["ALTER TABLE contract_documents ADD COLUMN document_style_json TEXT NOT NULL DEFAULT '{}'"],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract document design migration is irreversible to protect saved contracts."
                .to_string(),
        ))
    }
}
