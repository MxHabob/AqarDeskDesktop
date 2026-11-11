use crate::{
    application::{ApplicationError, CurrencyRepository, CurrencySettings},
    infrastructure::{audit, models::organization_currency},
};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, TransactionTrait,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "currencies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub code: String,
    pub name: String,
    pub name_ar: Option<String>,
    pub symbol: String,
    pub symbol_position: String,
    pub decimal_digits: i16,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct SeaOrmCurrencyRepository {
    database: DatabaseConnection,
}

impl SeaOrmCurrencyRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl CurrencyRepository for SeaOrmCurrencyRepository {
    async fn list(&self, organization_id: &str) -> Result<Vec<CurrencySettings>, ApplicationError> {
        let currencies = Entity::find()
            .order_by_asc(Column::Code)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let enabled = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization_id))
            .all(&self.database)
            .await
            .map_err(persistence_error)?
            .into_iter()
            .filter(|currency| currency.is_enabled)
            .map(|currency| currency.currency_code)
            .collect::<std::collections::HashSet<_>>();
        Ok(currencies
            .into_iter()
            .map(|currency| CurrencySettings {
                is_enabled_for_organization: enabled.contains(&currency.code),
                code: currency.code,
                name: currency.name,
                name_ar: currency.name_ar,
                symbol: currency.symbol,
                symbol_position: currency.symbol_position,
                decimal_digits: currency.decimal_digits,
                is_active: currency.is_active,
            })
            .collect())
    }

    async fn set_enabled(
        &self,
        organization_id: &str,
        actor_id: &str,
        code: &str,
        enabled: bool,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let currency = Entity::find_by_id(code)
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .filter(|currency| currency.is_active)
            .ok_or(ApplicationError::NotFound)?;
        let organization =
            crate::infrastructure::models::organization::Entity::find_by_id(organization_id)
                .one(&transaction)
                .await
                .map_err(persistence_error)?
                .ok_or(ApplicationError::NotFound)?;
        if !enabled && organization.default_currency == currency.code {
            return Err(crate::domain::DomainError::Validation(
                "The organization's default currency cannot be disabled.",
            )
            .into());
        }

        let existing = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization_id))
            .filter(organization_currency::Column::CurrencyCode.eq(code))
            .one(&transaction)
            .await
            .map_err(persistence_error)?;
        if let Some(existing) = existing {
            let mut active = existing.into_active_model();
            active.is_enabled = Set(enabled);
            active
                .update(&transaction)
                .await
                .map_err(persistence_error)?;
        } else {
            organization_currency::ActiveModel {
                organization_id: Set(organization_id.to_string()),
                currency_code: Set(code.to_string()),
                is_enabled: Set(enabled),
                created_by: Set(Some(actor_id.to_string())),
                created_at: Set(Utc::now()),
            }
            .insert(&transaction)
            .await
            .map_err(persistence_error)?;
        }
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "currency_enabled_changed",
            "currency",
            code,
            serde_json::json!({"is_enabled": enabled}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Currency persistence operation failed");
    ApplicationError::Persistence
}
