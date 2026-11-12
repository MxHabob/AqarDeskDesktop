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
                .filter(|template| template.lease_type != "receipt")
            {
                let content = default_template_content(template);
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
                let content_hash = Sha256::digest(content.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let placeholders_json = default_template_placeholders(template);
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
                         VALUES ('{}', '{}', '{}', 1, 'markdown', '{}', '{}', '{}', \
                          'Default DocForge lease template')",
                        version_id,
                        sql_literal(&organization_id),
                        template_id,
                        sql_literal(&content),
                        sql_literal(&placeholders_json),
                        content_hash
                    ))
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Default lease templates are retained to protect customer data.".to_string(),
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
    async fn adds_missing_lease_templates_without_overwriting_existing_templates() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        crate::migration::initialize(&database)
            .await
            .expect("initialize schema");
        database
            .execute_unprepared(
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('lease-org', 'Lease organization', 'SAR', 'Asia/Riyadh')",
            )
            .await
            .expect("create test organization");
        database
            .execute_unprepared(
                "INSERT INTO contract_templates \
                 (id, organization_id, template_key, name, lease_type, locale, is_active) \
                 VALUES ('existing-lease', 'lease-org', 'residential-lease-ar', \
                         'My custom residential template', 'residential', 'ar', TRUE)",
            )
            .await
            .expect("create existing template");
        database
            .execute_unprepared(
                "INSERT INTO contract_template_versions \
                 (id, organization_id, template_id, version_number, content_format, content_body, \
                  placeholders_json, content_sha256, change_note) \
                 VALUES ('existing-lease-version', 'lease-org', 'existing-lease', 1, 'markdown', \
                         'Customer content', '{}', 'customer-hash', 'Customer template')",
            )
            .await
            .expect("create existing template version");

        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("add default lease templates");
        Migration
            .up(&SchemaManager::new(&database))
            .await
            .expect("keep migration idempotent");

        let rows = database
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT t.template_key, t.name, t.is_active, v.content_body \
                 FROM contract_templates t \
                 JOIN contract_template_versions v ON v.template_id = t.id \
                 WHERE t.organization_id = 'lease-org' AND t.lease_type != 'receipt'",
            ))
            .await
            .expect("read lease templates");
        assert_eq!(rows.len(), 6);
        for row in rows {
            let key: String = row.try_get_by("template_key").expect("read template key");
            let name: String = row.try_get_by("name").expect("read template name");
            let is_active: bool = row.try_get_by("is_active").expect("read active flag");
            let content: String = row
                .try_get_by("content_body")
                .expect("read template content");
            if key == "residential-lease-ar" {
                assert_eq!(name, "My custom residential template");
                assert!(is_active);
                assert_eq!(content, "Customer content");
            } else {
                assert!(!is_active);
                assert!(!content.trim().is_empty());
                assert!(content.contains("{{tenant.full_name}}"));
            }
        }
    }
}
