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

        for template in DEFAULT_CONTRACT_TEMPLATES {
            let query = format!(
                "SELECT t.id, t.organization_id, v.version_number \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.template_key = '{}' AND t.name = '{}' AND t.is_active = FALSE \
                   AND v.version_number = (SELECT MAX(latest.version_number) \
                       FROM contract_template_versions latest WHERE latest.template_id = t.id) \
                   AND COALESCE(TRIM(v.content_body), '') = '' \
                   AND v.change_note = 'Blank editable onboarding template'",
                template.key,
                template.legacy_name.replace('\'', "''")
            );
            let rows = connection
                .query_all(Statement::from_string(backend, query))
                .await?;

            for row in rows {
                let template_id: String = row.try_get_by("id")?;
                let organization_id: String = row.try_get_by("organization_id")?;
                let version_number: i32 = row.try_get_by("version_number")?;
                let next_version = version_number.checked_add(1).ok_or_else(|| {
                    DbErr::Custom("Contract template version overflow.".to_string())
                })?;
                let content_hash = Sha256::digest(template.content.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let insert = format!(
                    "INSERT INTO contract_template_versions \
                     (id, organization_id, template_id, version_number, content_format, content_body, \
                      placeholders_json, content_sha256, change_note, created_by, created_at) \
                     VALUES ('{}', '{}', '{}', {}, 'markdown', '{}', '{{}}', '{}', \
                      'Starter template with data fields; review terms before use.', NULL, CURRENT_TIMESTAMP)",
                    Uuid::new_v4(),
                    sql_literal(&organization_id),
                    sql_literal(&template_id),
                    next_version,
                    sql_literal(template.content),
                    content_hash
                );
                connection.execute_unprepared(&insert).await?;

                let update = format!(
                    "UPDATE contract_templates SET name = '{}', updated_at = CURRENT_TIMESTAMP \
                     WHERE id = '{}' AND organization_id = '{}'",
                    sql_literal(template.name),
                    sql_literal(&template_id),
                    sql_literal(&organization_id)
                );
                connection.execute_unprepared(&update).await?;
            }
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Contract template starter content is retained to protect customer data.".to_string(),
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
    async fn upgrades_only_untouched_blank_onboarding_templates_with_a_new_version() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('starter-org', 'Starter organization', 'SAR', 'Asia/Riyadh')",
            )
            .await
            .expect("create test organization");

        for (index, template) in DEFAULT_CONTRACT_TEMPLATES
            .iter()
            .filter(|template| template.lease_type != "receipt")
            .enumerate()
        {
            let id = format!("starter-template-{index}");
            let name = if index == 0 {
                "Renamed by customer"
            } else {
                template.legacy_name
            };
            let create_template = format!(
                "INSERT INTO contract_templates \
                 (id, organization_id, template_key, name, lease_type, locale, is_active) \
                 VALUES ('{id}', 'starter-org', '{}', '{}', '{}', '{}', FALSE)",
                template.key,
                sql_literal(name),
                template.lease_type,
                template.locale
            );
            database
                .execute_unprepared(&create_template)
                .await
                .expect("create legacy default template");
            let create_version = format!(
                "INSERT INTO contract_template_versions \
                 (id, organization_id, template_id, version_number, content_format, content_body, \
                  placeholders_json, content_sha256, change_note) \
                 VALUES ('starter-version-{index}', 'starter-org', '{id}', 1, 'markdown', '', '{{}}', \
                  'empty', 'Blank editable onboarding template')"
            );
            database
                .execute_unprepared(&create_version)
                .await
                .expect("create legacy empty version");
        }

        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("upgrade starter templates");

        let rows = database
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT t.template_key, t.name, v.version_number, v.content_body \
                 FROM contract_templates t JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'starter-org' \
                   AND v.version_number = (SELECT MAX(latest.version_number) \
                     FROM contract_template_versions latest WHERE latest.template_id = t.id)",
            ))
            .await
            .expect("read upgraded templates");
        assert_eq!(rows.len(), 6);
        for row in rows {
            let key: String = row.try_get_by("template_key").expect("read template key");
            let name: String = row.try_get_by("name").expect("read template name");
            let version: i32 = row.try_get_by("version_number").expect("read version");
            let content: String = row.try_get_by("content_body").expect("read content");
            if key == DEFAULT_CONTRACT_TEMPLATES[0].key {
                assert_eq!(name, "Renamed by customer");
                assert_eq!(version, 1);
                assert!(content.is_empty());
            } else {
                assert_eq!(version, 2);
                assert!(content.contains("{{tenant.full_name}}"));
            }
        }
    }
}
