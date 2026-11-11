use crate::models::{
    audit_event, contract_template, contract_template_version, currency, organization,
    organization_currency, user,
};
use aqardesk_app::{
    AppError,
    services::settings::{
        AuditEntry, ContractTemplate, ContractTemplateInput, ContractTemplateVersion,
        OrganizationSettings, OrganizationSettingsInput, ProfileSettings, ProfileSettingsInput,
        SettingsRepository,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SeaOrmSettingsRepository<'db> {
    database: &'db DatabaseConnection,
}

impl<'db> SeaOrmSettingsRepository<'db> {
    pub fn new(database: &'db DatabaseConnection) -> Self {
        Self { database }
    }

    async fn insert_audit(
        &self,
        database: &impl sea_orm::ConnectionTrait,
        organization_id: &str,
        actor_id: &str,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        audit_event::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(organization_id.to_string()),
            actor_user_id: Set(Some(actor_id.to_string())),
            action: Set(action.to_string()),
            entity_type: Set(entity_type.to_string()),
            entity_id: Set(entity_id.to_string()),
            details_json: Set(Some(details.to_string())),
            created_at: Set(Utc::now()),
        }
        .insert(database)
        .await
        .map(|_| ())
        .map_err(|error| AppError::Persistence(format!("Failed to write audit event: {error}")))
    }

    async fn load_template(
        &self,
        database: &impl sea_orm::ConnectionTrait,
        organization_id: &str,
        template_id: &str,
    ) -> Result<ContractTemplate, AppError> {
        let model = contract_template::Entity::find_by_id(template_id)
            .filter(contract_template::Column::OrganizationId.eq(organization_id))
            .one(database)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to load contract template: {error}"))
            })?
            .ok_or_else(|| AppError::NotFound("Contract template was not found.".to_string()))?;
        let versions = contract_template_version::Entity::find()
            .filter(contract_template_version::Column::TemplateId.eq(template_id))
            .filter(contract_template_version::Column::OrganizationId.eq(organization_id))
            .order_by_desc(contract_template_version::Column::VersionNumber)
            .all(database)
            .await
            .map_err(|error| {
                AppError::Persistence(format!(
                    "Failed to load contract template versions: {error}"
                ))
            })?;
        Ok(ContractTemplate {
            id: model.id,
            template_key: model.template_key,
            name: model.name,
            description: model.description,
            lease_type: model.lease_type,
            locale: model.locale,
            is_active: model.is_active && model.archived_at.is_none(),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
            versions: versions.into_iter().map(template_version).collect(),
        })
    }
}

#[async_trait::async_trait]
impl SettingsRepository for SeaOrmSettingsRepository<'_> {
    async fn get_organization(
        &self,
        organization_id: &str,
    ) -> Result<OrganizationSettings, AppError> {
        organization::Entity::find_by_id(organization_id)
            .one(self.database)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to load organization settings: {error}"))
            })?
            .map(organization_settings)
            .ok_or_else(|| AppError::NotFound("Organization was not found.".to_string()))
    }

    async fn update_organization(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin organization update: {error}"))
        })?;
        let model = organization::Entity::find_by_id(organization_id)
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to find organization: {error}"))
            })?
            .ok_or_else(|| AppError::NotFound("Organization was not found.".to_string()))?;

        let currency_enabled = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization_id))
            .filter(organization_currency::Column::CurrencyCode.eq(&input.default_currency))
            .filter(organization_currency::Column::IsEnabled.eq(true))
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to validate organization currency: {error}"))
            })?
            .is_some();
        let currency_active = currency::Entity::find_by_id(&input.default_currency)
            .filter(currency::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to validate currency: {error}"))
            })?
            .is_some();
        if !currency_enabled || !currency_active {
            return Err(AppError::Validation(
                "Default currency must be enabled for this organization.".to_string(),
            ));
        }

        let mut active = model.into_active_model();
        active.name = Set(input.name);
        active.legal_name = Set(input.legal_name);
        active.default_currency = Set(input.default_currency);
        active.timezone = Set(input.timezone);
        active.updated_at = Set(Utc::now());
        let updated = active.update(&transaction).await.map_err(|error| {
            AppError::Persistence(format!("Failed to update organization: {error}"))
        })?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            "updated",
            "organization",
            organization_id,
            serde_json::json!({"fields": ["name", "legal_name", "default_currency", "timezone"]}),
        )
        .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save organization settings: {error}"))
        })?;
        Ok(organization_settings(updated))
    }

    async fn get_profile(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<ProfileSettings, AppError> {
        user::Entity::find_by_id(user_id)
            .filter(user::Column::OrganizationId.eq(organization_id))
            .filter(user::Column::IsActive.eq(true))
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to load profile: {error}")))?
            .map(profile_settings)
            .ok_or_else(|| AppError::NotFound("Active user profile was not found.".to_string()))
    }

    async fn update_profile(
        &self,
        organization_id: &str,
        user_id: &str,
        input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin profile update: {error}"))
        })?;
        let model = user::Entity::find_by_id(user_id)
            .filter(user::Column::OrganizationId.eq(organization_id))
            .filter(user::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to find user profile: {error}"))
            })?
            .ok_or_else(|| AppError::NotFound("Active user profile was not found.".to_string()))?;

        let password_changed = if let Some(new_password) = input.new_password.as_deref() {
            let current_password = input
                .current_password
                .as_deref()
                .ok_or_else(|| AppError::Validation("Current password is required.".to_string()))?;
            let verified =
                crate::users::verify_password_value(current_password, &model.password_hash)
                    .map_err(|error| {
                        AppError::Persistence(format!("Could not verify password: {error}"))
                    })?;
            if !verified {
                return Err(AppError::Validation(
                    "Current password is incorrect.".to_string(),
                ));
            }
            Some(
                crate::users::hash_password_value(new_password).map_err(|error| {
                    AppError::Persistence(format!(
                        "Could not securely hash the new password: {error}"
                    ))
                })?,
            )
        } else {
            None
        };

        let mut active = model.into_active_model();
        active.full_name = Set(input.name);
        active.email = Set(input.email);
        if let Some(password_hash) = password_changed {
            active.password_hash = Set(password_hash);
        }
        active.updated_at = Set(Utc::now());
        let updated = active
            .update(&transaction)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to update profile: {error}")))?;
        self.insert_audit(
            &transaction,
            organization_id,
            user_id,
            "updated",
            "user_profile",
            user_id,
            serde_json::json!({"password_changed": input.new_password.is_some()}),
        )
        .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save profile changes: {error}"))
        })?;
        Ok(profile_settings(updated))
    }

    async fn list_audit(
        &self,
        organization_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, AppError> {
        let events = audit_event::Entity::find()
            .filter(audit_event::Column::OrganizationId.eq(organization_id))
            .order_by_desc(audit_event::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(self.database)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to list audit events: {error}"))
            })?;
        let actor_ids = events
            .iter()
            .filter_map(|event| event.actor_user_id.clone())
            .collect::<Vec<_>>();
        let actors = if actor_ids.is_empty() {
            Vec::new()
        } else {
            user::Entity::find()
                .filter(user::Column::OrganizationId.eq(organization_id))
                .filter(user::Column::Id.is_in(actor_ids))
                .all(self.database)
                .await
                .map_err(|error| {
                    AppError::Persistence(format!("Failed to resolve audit actors: {error}"))
                })?
        };
        let actors = actors
            .into_iter()
            .map(|actor| (actor.id, actor.full_name))
            .collect::<HashMap<_, _>>();
        Ok(events
            .into_iter()
            .map(|event| AuditEntry {
                id: event.id,
                actor_name: event
                    .actor_user_id
                    .as_ref()
                    .and_then(|actor_id| actors.get(actor_id).cloned()),
                actor_user_id: event.actor_user_id,
                action: event.action,
                entity_type: event.entity_type,
                entity_id: event.entity_id,
                details_json: event.details_json,
                created_at: event.created_at.to_rfc3339(),
            })
            .collect())
    }

    async fn list_templates(
        &self,
        organization_id: &str,
    ) -> Result<Vec<ContractTemplate>, AppError> {
        let models = contract_template::Entity::find()
            .filter(contract_template::Column::OrganizationId.eq(organization_id))
            .order_by_asc(contract_template::Column::Name)
            .all(self.database)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to list contract templates: {error}"))
            })?;
        let mut templates = Vec::with_capacity(models.len());
        for model in models {
            templates.push(
                self.load_template(self.database, organization_id, &model.id)
                    .await?,
            );
        }
        Ok(templates)
    }

    async fn create_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin template creation: {error}"))
        })?;
        let now = Utc::now();
        let template_id = Uuid::new_v4().to_string();
        contract_template::ActiveModel {
            id: Set(template_id.clone()),
            organization_id: Set(organization_id.to_string()),
            template_key: Set(input.template_key.clone()),
            name: Set(input.name.clone()),
            description: Set(input.description.clone()),
            lease_type: Set(input.lease_type.clone()),
            locale: Set(input.locale.clone()),
            is_active: Set(true),
            created_by: Set(Some(actor_id.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            archived_at: Set(None),
        }
        .insert(&transaction)
        .await
        .map_err(|error| AppError::Persistence(format!("Failed to create template: {error}")))?;
        insert_template_version(
            &transaction,
            organization_id,
            actor_id,
            &template_id,
            1,
            input,
        )
        .await?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            "created",
            "contract_template",
            &template_id,
            serde_json::json!({"version": 1}),
        )
        .await?;
        let template = self
            .load_template(&transaction, organization_id, &template_id)
            .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save new template: {error}"))
        })?;
        Ok(template)
    }

    async fn update_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin template update: {error}"))
        })?;
        let model = contract_template::Entity::find_by_id(template_id)
            .filter(contract_template::Column::OrganizationId.eq(organization_id))
            .filter(contract_template::Column::ArchivedAt.is_null())
            .one(&transaction)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find template: {error}")))?
            .ok_or_else(|| {
                AppError::NotFound("Active contract template was not found.".to_string())
            })?;
        let latest_version = contract_template_version::Entity::find()
            .filter(contract_template_version::Column::TemplateId.eq(template_id))
            .filter(contract_template_version::Column::OrganizationId.eq(organization_id))
            .order_by_desc(contract_template_version::Column::VersionNumber)
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to find template version: {error}"))
            })?
            .ok_or_else(|| {
                AppError::NotFound("Contract template version was not found.".to_string())
            })?;
        let version_number = latest_version.version_number + 1;
        let mut active = model.into_active_model();
        active.template_key = Set(input.template_key.clone());
        active.name = Set(input.name.clone());
        active.description = Set(input.description.clone());
        active.lease_type = Set(input.lease_type.clone());
        active.locale = Set(input.locale.clone());
        active.updated_at = Set(Utc::now());
        active.update(&transaction).await.map_err(|error| {
            AppError::Persistence(format!("Failed to update template metadata: {error}"))
        })?;
        insert_template_version(
            &transaction,
            organization_id,
            actor_id,
            template_id,
            version_number,
            input,
        )
        .await?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            "version_created",
            "contract_template",
            template_id,
            serde_json::json!({"version": version_number}),
        )
        .await?;
        let template = self
            .load_template(&transaction, organization_id, template_id)
            .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save template version: {error}"))
        })?;
        Ok(template)
    }

    async fn set_template_active(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin template archive: {error}"))
        })?;
        let model = contract_template::Entity::find_by_id(template_id)
            .filter(contract_template::Column::OrganizationId.eq(organization_id))
            .one(&transaction)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find template: {error}")))?
            .ok_or_else(|| AppError::NotFound("Contract template was not found.".to_string()))?;
        let mut active = model.into_active_model();
        active.is_active = Set(is_active);
        active.archived_at = Set(if is_active { None } else { Some(Utc::now()) });
        active.updated_at = Set(Utc::now());
        active.update(&transaction).await.map_err(|error| {
            AppError::Persistence(format!("Failed to update template state: {error}"))
        })?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            if is_active { "restored" } else { "archived" },
            "contract_template",
            template_id,
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save template state: {error}"))
        })
    }
}

fn organization_settings(model: organization::Model) -> OrganizationSettings {
    OrganizationSettings {
        id: model.id,
        name: model.name,
        legal_name: model.legal_name,
        default_currency: model.default_currency,
        timezone: model.timezone,
    }
}

fn profile_settings(model: user::Model) -> ProfileSettings {
    ProfileSettings {
        id: model.id,
        name: model.full_name,
        email: model.email,
        role: model.access_role,
        is_active: model.is_active,
    }
}

fn template_version(model: contract_template_version::Model) -> ContractTemplateVersion {
    ContractTemplateVersion {
        id: model.id,
        version_number: model.version_number,
        content_format: model.content_format,
        content_body: model.content_body,
        placeholders_json: model.placeholders_json,
        content_sha256: model.content_sha256,
        change_note: model.change_note,
        created_at: model.created_at.to_rfc3339(),
    }
}

async fn insert_template_version(
    transaction: &impl sea_orm::ConnectionTrait,
    organization_id: &str,
    actor_id: &str,
    template_id: &str,
    version_number: i32,
    input: ContractTemplateInput,
) -> Result<(), AppError> {
    let content_sha256 = format!("{:x}", Sha256::digest(input.content_body.as_bytes()));
    contract_template_version::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        template_id: Set(template_id.to_string()),
        version_number: Set(version_number),
        content_format: Set(input.content_format),
        content_body: Set(Some(input.content_body)),
        source_storage_key: Set(None),
        placeholders_json: Set(input.placeholders_json.unwrap_or_else(|| "{}".to_string())),
        content_sha256: Set(content_sha256),
        change_note: Set(input.change_note),
        created_by: Set(Some(actor_id.to_string())),
        created_at: Set(Utc::now()),
    }
    .insert(transaction)
    .await
    .map(|_| ())
    .map_err(|error| AppError::Persistence(format!("Failed to add template version: {error}")))
}
