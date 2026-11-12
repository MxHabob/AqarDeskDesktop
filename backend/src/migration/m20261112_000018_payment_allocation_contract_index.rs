use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &["CREATE INDEX IF NOT EXISTS idx_payment_allocations_org_contract_payment ON payment_allocations (organization_id, contract_document_id, payment_id)"],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &["DROP INDEX IF EXISTS idx_payment_allocations_org_contract_payment"],
        )
        .await
    }
}
