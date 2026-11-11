use crate::contract_template_defaults::DEFAULT_CONTRACT_TEMPLATES;
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        let backend = connection.get_database_backend();
        let organizations = connection
            .query_all(Statement::from_string(
                backend,
                "SELECT id FROM organizations".to_string(),
            ))
            .await?;

        for organization in organizations {
            let organization_id: String = organization.try_get_by("id")?;
            for template in DEFAULT_CONTRACT_TEMPLATES
                .iter()
                .filter(|template| template.lease_type == "receipt")
            {
                let existing = connection
                    .query_one(Statement::from_string(
                        backend,
                        format!(
                            "SELECT id FROM contract_templates \
                             WHERE organization_id = '{}' AND template_key = '{}' AND locale = '{}'",
                            sql_literal(&organization_id),
                            sql_literal(template.key),
                            sql_literal(template.locale)
                        ),
                    ))
                    .await?;
                if existing.is_some() {
                    continue;
                }

                let template_id = Uuid::new_v4().to_string();
                let version_id = Uuid::new_v4().to_string();
                let content_hash = Sha256::digest(template.content.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                connection
                    .execute_unprepared(&format!(
                        "INSERT INTO contract_templates \
                         (id, organization_id, template_key, name, lease_type, locale, is_active) \
                         VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {})",
                        template_id,
                        sql_literal(&organization_id),
                        sql_literal(template.key),
                        sql_literal(template.name),
                        sql_literal(template.lease_type),
                        sql_literal(template.locale),
                        template.is_active
                    ))
                    .await?;
                connection
                    .execute_unprepared(&format!(
                        "INSERT INTO contract_template_versions \
                         (id, organization_id, template_id, version_number, content_format, content_body, \
                          placeholders_json, content_sha256, change_note) \
                         VALUES ('{}', '{}', '{}', 1, 'markdown', '{}', '{{}}', '{}', \
                          'Default payment receipt template')",
                        version_id,
                        sql_literal(&organization_id),
                        template_id,
                        sql_literal(template.content),
                        content_hash
                    ))
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Default payment receipt templates are retained to protect customer data.".to_string(),
        ))
    }
}

fn sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DbBackend};

    #[tokio::test]
    async fn adds_active_localized_receipt_templates_once_for_existing_organizations() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('receipt-org', 'Receipt organization', 'SAR', 'Asia/Riyadh')",
            )
            .await
            .expect("create test organization");

        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("add receipt templates");
        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("keep migration idempotent");

        let rows = database
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT t.template_key, t.is_active, t.lease_type, v.content_body \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'receipt-org' ORDER BY t.locale",
            ))
            .await
            .expect("read receipt templates");
        assert_eq!(rows.len(), 2);
        for row in rows {
            let key: String = row.try_get_by("template_key").expect("read template key");
            let is_active: bool = row.try_get_by("is_active").expect("read active flag");
            let lease_type: String = row.try_get_by("lease_type").expect("read template type");
            let content: String = row
                .try_get_by("content_body")
                .expect("read template content");
            assert!(key.starts_with("payment-receipt-"));
            assert!(is_active);
            assert_eq!(lease_type, "receipt");
            assert!(content.contains("{{payment.receipt_number}}"));
        }
    }
}
