use crate::contract_template_defaults::{
    DEFAULT_CONTRACT_TEMPLATES, default_template_content, default_template_placeholders,
};
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_CHANGE_NOTES: &[&str] = &[
    "Blank editable onboarding template",
    "Default Yemen lease template",
    "Default DocForge payment receipt template",
    "DocForge starter template; review and customize before use.",
    "Added DocForge to unchanged Yemen default template",
    "Redesigned default template with DocForge",
    "Redesigned Yemen default template with legal clauses",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        let backend = connection.get_database_backend();
        let change_notes = DEFAULT_CHANGE_NOTES
            .iter()
            .map(|note| format!("'{}'", sql_literal(note)))
            .collect::<Vec<_>>()
            .join(", ");

        for template in &DEFAULT_CONTRACT_TEMPLATES {
            let rows = connection
                .query_all(Statement::from_string(
                    backend,
                    format!(
                        "SELECT t.id, t.organization_id, v.version_number \
                         FROM contract_templates t \
                         JOIN contract_template_versions v ON v.template_id = t.id \
                         WHERE t.template_key = '{}' AND t.locale = '{}' \
                           AND v.version_number = (SELECT MAX(latest.version_number) \
                               FROM contract_template_versions latest \
                               WHERE latest.template_id = t.id) \
                           AND COALESCE(v.change_note, '') IN ({change_notes})",
                        sql_literal(template.key),
                        sql_literal(template.locale),
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
                          'Refreshed default template design', NULL, CURRENT_TIMESTAMP)",
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
            "Refreshed default template designs are retained to protect customer data.".to_string(),
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
    use serde_json::Value;

    #[tokio::test]
    async fn refreshes_only_unedited_defaults_and_sets_receipts_to_a5_landscape() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('template-refresh-org', 'Template refresh organization', 'SAR', 'Asia/Riyadh')",
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

        for (index, (template, change_note, body)) in [
            (lease, "Default Yemen lease template", lease.content),
            (
                receipt,
                "Default DocForge payment receipt template",
                receipt.content,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let template_id = format!("template-refresh-{index}");
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_templates \
                     (id, organization_id, template_key, name, lease_type, locale, is_active) \
                     VALUES ('{}', 'template-refresh-org', '{}', '{}', '{}', '{}', FALSE)",
                    template_id,
                    sql_literal(template.key),
                    sql_literal(template.name),
                    template.lease_type,
                    template.locale
                ))
                .await
                .expect("create test template");
            database
                .execute_unprepared(&format!(
                    "INSERT INTO contract_template_versions \
                     (id, organization_id, template_id, version_number, content_format, content_body, \
                      placeholders_json, content_sha256, change_note) \
                     VALUES ('refresh-version-{index}', 'template-refresh-org', '{}', 1, 'markdown', '{}', \
                             '{}', 'old-hash', '{}')",
                    template_id,
                    sql_literal(body),
                    sql_literal(&default_template_placeholders(template)),
                    sql_literal(change_note)
                ))
                .await
                .expect("create test template version");
        }
        database
            .execute_unprepared(
                "INSERT INTO contract_template_versions \
                 (id, organization_id, template_id, version_number, content_format, content_body, \
                  placeholders_json, content_sha256, change_note) \
                 VALUES ('refresh-version-custom', 'template-refresh-org', 'template-refresh-0', 2, \
                         'markdown', 'My custom contract', '{}', 'custom-hash', \
                         'Customer customized template')",
            )
            .await
            .expect("create customer-edited template version");

        let manager = SchemaManager::new(&database);
        Migration.up(&manager).await.expect("refresh defaults");
        Migration
            .up(&manager)
            .await
            .expect("keep refresh idempotent");

        let rows = database
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT t.template_key, v.version_number, v.content_body, v.placeholders_json, v.change_note \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'template-refresh-org' \
                   AND v.version_number = (SELECT MAX(latest.version_number) \
                     FROM contract_template_versions latest WHERE latest.template_id = t.id) \
                 ORDER BY t.template_key",
            ))
            .await
            .expect("read refreshed templates");

        assert_eq!(rows.len(), 2);
        for row in rows {
            let template_key: String = row.try_get_by("template_key").expect("read template key");
            let version: i32 = row.try_get_by("version_number").expect("read version");
            let content: String = row.try_get_by("content_body").expect("read content");
            let placeholders: String = row
                .try_get_by("placeholders_json")
                .expect("read placeholders");
            let change_note: String = row.try_get_by("change_note").expect("read change note");
            if template_key == lease.key {
                assert_eq!(version, 2);
                assert_eq!(change_note, "Customer customized template");
                assert_eq!(content, "My custom contract");
                continue;
            }

            assert_eq!(version, 2);
            assert_eq!(change_note, "Refreshed default template design");
            let placeholders: Value =
                serde_json::from_str(&placeholders).expect("parse refreshed placeholders");
            let document: Value = serde_json::from_str(
                placeholders["docforge_document"]
                    .as_str()
                    .expect("read DocForge document"),
            )
            .expect("parse DocForge document");
            if template_key == receipt.key {
                assert_eq!(document["meta"]["pageSetup"]["size"], "A5");
                assert_eq!(document["meta"]["pageSetup"]["orientation"], "landscape");
            }
        }
    }
}
