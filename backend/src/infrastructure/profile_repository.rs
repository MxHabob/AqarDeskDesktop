use crate::{
    application::{ApplicationError, ProfileRepository, ProfileSettings, ProfileSettingsInput},
    infrastructure::{
        audit,
        identity_repository::{hash_password, verify_password},
        models::user,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, TransactionTrait,
};

pub struct SeaOrmProfileRepository {
    database: DatabaseConnection,
}

impl SeaOrmProfileRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ProfileRepository for SeaOrmProfileRepository {
    async fn get(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<ProfileSettings, ApplicationError> {
        let account = user::Entity::find_by_id(user_id)
            .filter(user::Column::OrganizationId.eq(organization_id))
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        Ok(profile(account))
    }

    async fn update(
        &self,
        organization_id: &str,
        user_id: &str,
        input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let account = user::Entity::find_by_id(user_id)
            .filter(user::Column::OrganizationId.eq(organization_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;

        if let Some(new_password) = input.new_password.as_deref() {
            let current_password = input
                .current_password
                .as_deref()
                .ok_or(ApplicationError::Authentication)?;
            if !verify_password(current_password, &account.password_hash)? {
                return Err(ApplicationError::Authentication);
            }
            let password_hash = hash_password(new_password)?;
            let mut active = account.into_active_model();
            active.full_name = Set(input.name);
            active.email = Set(input.email);
            active.password_hash = Set(password_hash);
            active.updated_at = Set(Utc::now());
            let updated = active
                .update(&transaction)
                .await
                .map_err(persistence_error)?;
            audit::record(
                &transaction,
                organization_id,
                user_id,
                "profile_and_password_updated",
                "user",
                user_id,
                serde_json::json!({"email": updated.email}),
            )
            .await?;
            transaction.commit().await.map_err(persistence_error)?;
            Ok(profile(updated))
        } else {
            let mut active = account.into_active_model();
            active.full_name = Set(input.name);
            active.email = Set(input.email);
            active.updated_at = Set(Utc::now());
            let updated = active
                .update(&transaction)
                .await
                .map_err(persistence_error)?;
            audit::record(
                &transaction,
                organization_id,
                user_id,
                "profile_updated",
                "user",
                user_id,
                serde_json::json!({"email": updated.email}),
            )
            .await?;
            transaction.commit().await.map_err(persistence_error)?;
            Ok(profile(updated))
        }
    }
}

fn profile(account: user::Model) -> ProfileSettings {
    ProfileSettings {
        id: account.id,
        name: account.full_name,
        email: account.email,
        role: account.access_role,
        is_active: account.is_active,
    }
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Profile persistence operation failed");
    ApplicationError::Persistence
}
