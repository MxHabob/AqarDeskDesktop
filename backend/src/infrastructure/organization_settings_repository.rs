use crate::{
    application::{
        ApplicationError, OrganizationSettings, OrganizationSettingsInput,
        OrganizationSettingsRepository,
    },
    infrastructure::{
        audit,
        currency_repository::{Column as CurrencyColumn, Entity as CurrencyEntity},
        models::organization,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, TransactionTrait,
};

pub struct SeaOrmOrganizationSettingsRepository {
    database: DatabaseConnection,
}

impl SeaOrmOrganizationSettingsRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl OrganizationSettingsRepository for SeaOrmOrganizationSettingsRepository {
    async fn get(&self, organization_id: &str) -> Result<OrganizationSettings, ApplicationError> {
        let organization = organization::Entity::find_by_id(organization_id)
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        Ok(OrganizationSettings {
            id: organization.id,
            name: organization.name,
            legal_name: organization.legal_name,
            logo_data_url: organization.logo_data_url,
            default_currency: organization.default_currency,
            timezone: organization.timezone,
        })
    }

    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let current = organization::Entity::find_by_id(organization_id)
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let currency_enabled = CurrencyEntity::find_by_id(&input.default_currency)
            .filter(CurrencyColumn::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some()
            && crate::infrastructure::models::organization_currency::Entity::find()
                .filter(
                    crate::infrastructure::models::organization_currency::Column::OrganizationId
                        .eq(organization_id),
                )
                .filter(
                    crate::infrastructure::models::organization_currency::Column::CurrencyCode
                        .eq(&input.default_currency),
                )
                .filter(
                    crate::infrastructure::models::organization_currency::Column::IsEnabled
                        .eq(true),
                )
                .one(&transaction)
                .await
                .map_err(persistence_error)?
                .is_some();
        if !currency_enabled {
            return Err(crate::domain::DomainError::Validation(
                "The default currency must be enabled for the organization.",
            )
            .into());
        }

        let mut active = current.into_active_model();
        active.name = Set(input.name);
        active.legal_name = Set(input.legal_name);
        active.logo_data_url = Set(input.logo_data_url);
        active.default_currency = Set(input.default_currency);
        active.timezone = Set(input.timezone);
        active.updated_at = Set(Utc::now());
        let updated = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "updated",
            "organization_settings",
            organization_id,
            serde_json::json!({"name": updated.name, "default_currency": updated.default_currency}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        Ok(OrganizationSettings {
            id: updated.id,
            name: updated.name,
            legal_name: updated.legal_name,
            logo_data_url: updated.logo_data_url,
            default_currency: updated.default_currency,
            timezone: updated.timezone,
        })
    }
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Organization settings persistence failed");
    ApplicationError::Persistence
}
