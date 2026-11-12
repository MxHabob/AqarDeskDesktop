use crate::contract_template_defaults::{
    DEFAULT_CONTRACT_TEMPLATES, default_template_content, default_template_placeholders,
};
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

        for template in &DEFAULT_CONTRACT_TEMPLATES {
            let rows = connection
                .query_all(Statement::from_string(
                    backend,
                    format!(
                        "SELECT t.id, t.organization_id, v.version_number \
                         FROM contract_templates t \
                         JOIN contract_template_versions v ON v.template_id = t.id \
                         WHERE t.template_key = '{}' \
                           AND v.version_number = (SELECT MAX(latest.version_number) \
                               FROM contract_template_versions latest \
                               WHERE latest.template_id = t.id) \
                           AND v.change_note = 'Redesigned default template with DocForge'",
                        sql_literal(template.key)
                    ),
                ))
                .await?;

            for row in rows {
                let template_id: String = row.try_get_by("id")?;
                let organization_id: String = row.try_get_by("organization_id")?;
                let version_number: i32 = row.try_get_by("version_number")?;
                let next_version = version_number.checked_add(1).ok_or_else(|| {
                    DbErr::Custom("Contract template version overflow.".to_string())
                })?;
                let content = default_template_content(template);
                let content_hash = Sha256::digest(content.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let placeholders_json = default_template_placeholders(template);

                connection
                    .execute_unprepared(&format!(
                        "INSERT INTO contract_template_versions \
                         (id, organization_id, template_id, version_number, content_format, content_body, \
                          placeholders_json, content_sha256, change_note, created_by, created_at) \
                         VALUES ('{}', '{}', '{}', {}, 'markdown', '{}', '{}', '{}', \
                          'Redesigned Yemen default template with legal clauses', NULL, CURRENT_TIMESTAMP)",
                        Uuid::new_v4(),
                        sql_literal(&organization_id),
                        sql_literal(&template_id),
                        next_version,
                        sql_literal(&content),
                        sql_literal(&placeholders_json),
                        content_hash
                    ))
                    .await?;
                connection
                    .execute_unprepared(&format!(
                        "UPDATE contract_templates SET updated_at = CURRENT_TIMESTAMP \
                         WHERE id = '{}' AND organization_id = '{}'",
                        sql_literal(&template_id),
                        sql_literal(&organization_id)
                    ))
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Yemen default template revisions are retained to protect customer data.".to_string(),
        ))
    }
}

fn sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn replaces_only_prior_unedited_docforge_defaults() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('yemen-template-org', 'Yemen template organization', 'YER', 'Asia/Aden')",
            )
            .await
            .expect("create test organization");

        let lease = DEFAULT_CONTRACT_TEMPLATES
            .iter()
            .find(|template| template.lease_type == "residential" && template.locale == "ar")
            .expect("find Arabic lease");
        let receipt = DEFAULT_CONTRACT_TEMPLATES
            .iter()
            .find(|template| template.lease_type == "receipt" && template.locale == "ar")
            .expect("find Arabic receipt");
        let custom = DEFAULT_CONTRACT_TEMPLATES
            .iter()
            .find(|template| template.lease_type == "commercial" && template.locale == "ar")
            .expect("find Arabic commercial lease");

        for (index, template) in [lease, receipt, custom].into_iter().enumerate() {
            let template_id = format!("yemen-template-{index}");
            let (content, change_note) = if index == 2 {
                ("Customer-edited contract body", "Customer edit")
            } else {
                (
                    template.content,
                    "Redesigned default template with DocForge",
                )
            };
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_templates \
                     (id, organization_id, template_key, name, lease_type, locale, is_active) \
                     VALUES ('{}', 'yemen-template-org', '{}', '{}', '{}', '{}', FALSE)",
                    template_id,
                    sql_literal(template.key),
                    sql_literal(template.name),
                    template.lease_type,
                    template.locale
                ))
                .await
                .expect("create prior default template");
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_template_versions \
                     (id, organization_id, template_id, version_number, content_format, content_body, \
                      placeholders_json, content_sha256, change_note) \
                     VALUES ('yemen-version-{index}', 'yemen-template-org', '{}', 1, 'markdown', '{}', \
                             '{{}}', 'old-hash', '{}')",
                    template_id,
                    sql_literal(content),
                    sql_literal(change_note)
                ))
                .await
                .expect("create prior DocForge version");
        }

        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("redesign default templates");
        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("keep migration idempotent");

        let rows = database
            .query_all(Statement::from_string(
                database.get_database_backend(),
                "SELECT t.lease_type, v.version_number, v.content_body, v.placeholders_json \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'yemen-template-org' \
                   AND v.version_number = (SELECT MAX(latest.version_number) \
                     FROM contract_template_versions latest WHERE latest.template_id = t.id) \
                 ORDER BY t.lease_type",
            ))
            .await
            .expect("read redesigned templates");

        assert_eq!(rows.len(), 3);
        for row in rows {
            let lease_type: String = row.try_get_by("lease_type").expect("read template type");
            let version: i32 = row.try_get_by("version_number").expect("read version");
            let content: String = row.try_get_by("content_body").expect("read content");
            let placeholders: String = row
                .try_get_by("placeholders_json")
                .expect("read placeholders");
            if lease_type == "commercial" {
                assert_eq!(version, 1);
                assert_eq!(content, "Customer-edited contract body");
                assert!(!placeholders.contains("docforge_document"));
                continue;
            }

            assert_eq!(version, 2);
            assert!(placeholders.contains("docforge_document"));
            if lease_type == "residential" {
                assert!(content.contains("[[PAGE_BREAK]]"));
                assert!(content.contains("المادة (7)"));
            } else {
                let placeholders: serde_json::Value =
                    serde_json::from_str(&placeholders).expect("parse receipt placeholders");
                let document: serde_json::Value = serde_json::from_str(
                    placeholders["docforge_document"]
                        .as_str()
                        .expect("read receipt DocForge document"),
                )
                .expect("parse receipt DocForge document");
                assert_eq!(document["meta"]["pageSetup"]["size"], "A5");
            }
        }
    }
}
