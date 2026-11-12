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
                        "SELECT t.id, t.organization_id, v.version_number, \
                                v.content_body, v.placeholders_json \
                         FROM contract_templates t \
                         JOIN contract_template_versions v ON v.template_id = t.id \
                         WHERE t.template_key = '{}' \
                           AND v.version_number = (SELECT MAX(latest.version_number) \
                               FROM contract_template_versions latest \
                               WHERE latest.template_id = t.id)",
                        sql_literal(template.key)
                    ),
                ))
                .await?;

            for row in rows {
                let content: Option<String> = row.try_get_by("content_body")?;
                let placeholders: String = row.try_get_by("placeholders_json")?;
                if content.as_deref() != Some(template.content) || placeholders != "{}" {
                    continue;
                }

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
                          'Added DocForge to unchanged Yemen default template', NULL, CURRENT_TIMESTAMP)",
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
            "DocForge default templates are retained to protect customer data.".to_string(),
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
    async fn upgrades_only_unchanged_defaults_and_is_idempotent() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('docforge-org', 'DocForge organization', 'SAR', 'Asia/Riyadh')",
            )
            .await
            .expect("create test organization");

        for (index, template) in DEFAULT_CONTRACT_TEMPLATES.iter().take(3).enumerate() {
            let template_id = format!("docforge-template-{index}");
            let (name, body, placeholders) = match index {
                0 => ("Renamed default", template.content, "{}"),
                1 => ("Edited default", "Customer-edited content", "{}"),
                _ => (
                    "Customized design",
                    template.content,
                    r##"{"design":{"accent_color":"#FF0000"}}"##,
                ),
            };
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_templates \
                     (id, organization_id, template_key, name, lease_type, locale, is_active) \
                     VALUES ('{}', 'docforge-org', '{}', '{}', '{}', '{}', FALSE)",
                    template_id,
                    sql_literal(template.key),
                    sql_literal(name),
                    template.lease_type,
                    template.locale
                ))
                .await
                .expect("create default template");
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_template_versions \
                     (id, organization_id, template_id, version_number, content_format, content_body, \
                      placeholders_json, content_sha256, change_note) \
                     VALUES ('docforge-version-{index}', 'docforge-org', '{}', 1, 'markdown', '{}', '{}', \
                             'original-hash', 'Default template')",
                    template_id,
                    sql_literal(body),
                    sql_literal(placeholders)
                ))
                .await
                .expect("create default template version");
        }

        let manager = SchemaManager::new(&database);
        Migration.up(&manager).await.expect("upgrade defaults");
        Migration
            .up(&manager)
            .await
            .expect("keep migration idempotent");

        let rows = database
            .query_all(Statement::from_string(
                database.get_database_backend(),
                "SELECT t.name, v.version_number, v.placeholders_json \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'docforge-org' \
                   AND v.version_number = (SELECT MAX(latest.version_number) \
                     FROM contract_template_versions latest WHERE latest.template_id = t.id) \
                 ORDER BY t.template_key",
            ))
            .await
            .expect("read migrated templates");

        assert_eq!(rows.len(), 3);
        for row in &rows {
            let name: String = row.try_get_by("name").expect("read template name");
            let version: i32 = row.try_get_by("version_number").expect("read version");
            let placeholders: String = row
                .try_get_by("placeholders_json")
                .expect("read placeholders");
            if name == "Renamed default" {
                assert_eq!(name, "Renamed default");
                assert_eq!(version, 2);
                assert!(placeholders.contains("docforge_document"));
            } else {
                assert_eq!(version, 1);
                assert!(!placeholders.contains("docforge_document"));
            }
        }
    }
}
