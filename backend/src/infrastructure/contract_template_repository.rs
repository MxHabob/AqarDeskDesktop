use crate::{
    application::{
        ApplicationError, ContractTemplate, ContractTemplateInput, ContractTemplateRepository,
        ContractTemplateVersion,
    },
    infrastructure::audit,
};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, TransactionTrait,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contract_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub organization_id: String,
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub lease_type: String,
    pub locale: String,
    pub is_active: bool,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

mod version {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "contract_template_versions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub template_id: String,
        pub version_number: i32,
        pub content_format: String,
        pub content_body: Option<String>,
        pub placeholders_json: String,
        pub content_sha256: String,
        pub change_note: Option<String>,
        pub created_by: Option<String>,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub struct SeaOrmContractTemplateRepository {
    database: DatabaseConnection,
}

impl SeaOrmContractTemplateRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }

    async fn template_with_versions(
        &self,
        template: Model,
        connection: &DatabaseConnection,
    ) -> Result<ContractTemplate, ApplicationError> {
        let versions = version::Entity::find()
            .filter(version::Column::TemplateId.eq(&template.id))
            .order_by_desc(version::Column::VersionNumber)
            .all(connection)
            .await
            .map_err(persistence_error)?;
        Ok(to_template(template, versions))
    }
}

pub(super) async fn seed_onboarding_templates(
    transaction: &DatabaseTransaction,
    organization_id: &str,
    actor_id: &str,
    now: chrono::DateTime<Utc>,
) -> Result<(), ApplicationError> {
    const TEMPLATES: [(&str, &str, &str, &str); 6] = [
        (
            "residential-lease-ar",
            "قالب إيجار سكني (فارغ)",
            "residential",
            "ar",
        ),
        (
            "commercial-lease-ar",
            "قالب إيجار تجاري (فارغ)",
            "commercial",
            "ar",
        ),
        (
            "short-term-lease-ar",
            "قالب إيجار قصير الأجل (فارغ)",
            "short_term",
            "ar",
        ),
        (
            "residential-lease-en",
            "Residential lease (blank)",
            "residential",
            "en",
        ),
        (
            "commercial-lease-en",
            "Commercial lease (blank)",
            "commercial",
            "en",
        ),
        (
            "short-term-lease-en",
            "Short-term lease (blank)",
            "short_term",
            "en",
        ),
    ];

    for (template_key, name, lease_type, locale) in TEMPLATES {
        let template_id = Uuid::new_v4().to_string();
        ActiveModel {
            id: Set(template_id.clone()),
            organization_id: Set(organization_id.to_string()),
            template_key: Set(template_key.to_string()),
            name: Set(name.to_string()),
            description: Set(None),
            lease_type: Set(lease_type.to_string()),
            locale: Set(locale.to_string()),
            is_active: Set(false),
            created_by: Set(Some(actor_id.to_string())),
            updated_by: Set(Some(actor_id.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(transaction)
        .await
        .map_err(persistence_error)?;

        let content_body = String::new();
        let content_sha256 = Sha256::digest(content_body.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        version::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(organization_id.to_string()),
            template_id: Set(template_id.clone()),
            version_number: Set(1),
            content_format: Set("markdown".to_string()),
            content_body: Set(Some(content_body)),
            placeholders_json: Set("{}".to_string()),
            content_sha256: Set(content_sha256),
            change_note: Set(Some("Blank editable onboarding template".to_string())),
            created_by: Set(Some(actor_id.to_string())),
            created_at: Set(now),
        }
        .insert(transaction)
        .await
        .map_err(persistence_error)?;

        audit::record(
            transaction,
            organization_id,
            actor_id,
            "created",
            "contract_template",
            &template_id,
            serde_json::json!({"template_key": template_key, "onboarding_default": true}),
        )
        .await?;
    }

    Ok(())
}

#[async_trait::async_trait]
impl ContractTemplateRepository for SeaOrmContractTemplateRepository {
    async fn list(&self, organization_id: &str) -> Result<Vec<ContractTemplate>, ApplicationError> {
        let templates = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .order_by_asc(Column::Name)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let ids = templates
            .iter()
            .map(|template| template.id.clone())
            .collect::<Vec<_>>();
        let versions = if ids.is_empty() {
            Vec::new()
        } else {
            version::Entity::find()
                .filter(version::Column::OrganizationId.eq(organization_id))
                .filter(version::Column::TemplateId.is_in(ids))
                .order_by_desc(version::Column::VersionNumber)
                .all(&self.database)
                .await
                .map_err(persistence_error)?
        };
        let mut versions_by_template = std::collections::HashMap::<String, Vec<_>>::new();
        for item in versions {
            versions_by_template
                .entry(item.template_id.clone())
                .or_default()
                .push(item);
        }
        Ok(templates
            .into_iter()
            .map(|template| {
                let versions = versions_by_template
                    .remove(&template.id)
                    .unwrap_or_default();
                to_template(template, versions)
            })
            .collect())
    }

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let now = Utc::now();
        let template = ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(organization_id.to_string()),
            template_key: Set(input.template_key.clone()),
            name: Set(input.name.clone()),
            description: Set(input.description.clone()),
            lease_type: Set(input.lease_type.clone()),
            locale: Set(input.locale.clone()),
            is_active: Set(true),
            created_by: Set(Some(actor_id.to_string())),
            updated_by: Set(Some(actor_id.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        insert_version(
            &transaction,
            organization_id,
            actor_id,
            &template.id,
            1,
            input,
        )
        .await?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "created",
            "contract_template",
            &template.id,
            serde_json::json!({"template_key": template.template_key}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        self.template_with_versions(template, &self.database).await
    }

    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = Entity::find_by_id(template_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let latest_version = version::Entity::find()
            .filter(version::Column::TemplateId.eq(template_id))
            .order_by_desc(version::Column::VersionNumber)
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .map_or(0, |item| item.version_number);
        let mut active = existing.into_active_model();
        active.template_key = Set(input.template_key.clone());
        active.name = Set(input.name.clone());
        active.description = Set(input.description.clone());
        active.lease_type = Set(input.lease_type.clone());
        active.locale = Set(input.locale.clone());
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(Utc::now());
        let template = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        let next_version = latest_version
            .checked_add(1)
            .ok_or(ApplicationError::Persistence)?;
        insert_version(
            &transaction,
            organization_id,
            actor_id,
            template_id,
            next_version,
            input,
        )
        .await?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "updated",
            "contract_template",
            template_id,
            serde_json::json!({"version": next_version}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        self.template_with_versions(template, &self.database).await
    }

    async fn set_active(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = Entity::find_by_id(template_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if is_active {
            let latest_content = version::Entity::find()
                .filter(version::Column::OrganizationId.eq(organization_id))
                .filter(version::Column::TemplateId.eq(template_id))
                .order_by_desc(version::Column::VersionNumber)
                .one(&transaction)
                .await
                .map_err(persistence_error)?
                .and_then(|item| item.content_body)
                .unwrap_or_default();
            if latest_content.trim().is_empty() {
                return Err(crate::domain::DomainError::Validation(
                    "Add contract content before activating this template.",
                )
                .into());
            }
        }
        let mut active = existing.into_active_model();
        active.is_active = Set(is_active);
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(Utc::now());
        active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            if is_active {
                "activated"
            } else {
                "deactivated"
            },
            "contract_template",
            template_id,
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }
}

async fn insert_version(
    transaction: &DatabaseTransaction,
    organization_id: &str,
    actor_id: &str,
    template_id: &str,
    version_number: i32,
    input: ContractTemplateInput,
) -> Result<(), ApplicationError> {
    let digest = Sha256::digest(input.content_body.as_bytes());
    let content_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    version::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        template_id: Set(template_id.to_string()),
        version_number: Set(version_number),
        content_format: Set(input.content_format),
        content_body: Set(Some(input.content_body)),
        placeholders_json: Set(input.placeholders_json.unwrap_or_else(|| "{}".to_string())),
        content_sha256: Set(content_sha256),
        change_note: Set(input.change_note),
        created_by: Set(Some(actor_id.to_string())),
        created_at: Set(Utc::now()),
    }
    .insert(transaction)
    .await
    .map(|_| ())
    .map_err(persistence_error)
}

fn to_template(template: Model, versions: Vec<version::Model>) -> ContractTemplate {
    ContractTemplate {
        id: template.id,
        template_key: template.template_key,
        name: template.name,
        description: template.description,
        lease_type: template.lease_type,
        locale: template.locale,
        is_active: template.is_active,
        created_at: template.created_at.to_rfc3339(),
        updated_at: template.updated_at.to_rfc3339(),
        versions: versions
            .into_iter()
            .map(|version| ContractTemplateVersion {
                id: version.id,
                version_number: version.version_number,
                content_format: version.content_format,
                content_body: version.content_body,
                placeholders_json: version.placeholders_json,
                content_sha256: version.content_sha256,
                change_note: version.change_note,
                created_at: version.created_at.to_rfc3339(),
            })
            .collect(),
    }
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Contract template persistence operation failed");
    ApplicationError::Persistence
}
