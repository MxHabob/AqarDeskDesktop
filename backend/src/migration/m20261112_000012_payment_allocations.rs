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
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_id_org ON payments (id, organization_id)",
                r#"CREATE TABLE IF NOT EXISTS payment_allocations (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    payment_id TEXT NOT NULL,
                    contract_document_id TEXT NOT NULL,
                    contract_number TEXT NOT NULL,
                    property_name TEXT NOT NULL,
                    property_code TEXT NOT NULL,
                    unit_number TEXT NOT NULL,
                    amount_minor BIGINT NOT NULL CHECK (amount_minor > 0),
                    UNIQUE (payment_id, contract_document_id),
                    FOREIGN KEY (payment_id, organization_id) REFERENCES payments(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (contract_document_id, organization_id) REFERENCES contract_documents(id, organization_id) ON DELETE RESTRICT
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_payment_allocations_org_payment ON payment_allocations (organization_id, payment_id)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Payment allocation migration is irreversible to protect recorded allocations."
                .to_string(),
        ))
    }
}
