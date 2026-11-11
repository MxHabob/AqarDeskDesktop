use crate::{
    application::{
        ApplicationError, IdentityRepository, NewUser, OrganizationBootstrap, UserUpdate,
    },
    domain::{AccessRole, Organization, OrganizationId, UserAccount, UserId},
    infrastructure::{
        audit,
        models::{installation, organization, organization_currency, user},
    },
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{Duration, Utc};
use rand::rngs::OsRng;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait, sea_query::Expr,
};
const MAX_FAILED_LOGINS: i32 = 5;
const LOCKOUT_MINUTES: i64 = 15;

pub struct SeaOrmIdentityRepository {
    database: DatabaseConnection,
}

impl SeaOrmIdentityRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl IdentityRepository for SeaOrmIdentityRepository {
    async fn has_users(&self) -> Result<bool, ApplicationError> {
        let count = user::Entity::find()
            .count(&self.database)
            .await
            .map_err(persistence_error)?;
        Ok(count > 0)
    }

    async fn organization_name(
        &self,
        organization_id: OrganizationId,
    ) -> Result<String, ApplicationError> {
        organization::Entity::find_by_id(organization_id.to_string())
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .map(|organization| organization.name)
            .ok_or(ApplicationError::NotFound)
    }

    async fn bootstrap_admin(
        &self,
        input: OrganizationBootstrap<'_>,
    ) -> Result<(Organization, UserAccount), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        if user::Entity::find()
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some()
        {
            return Err(ApplicationError::AlreadyConfigured);
        }

        let claimed = installation::Entity::update_many()
            .col_expr(installation::Column::Initialized, Expr::value(true))
            .filter(installation::Column::Id.eq(1))
            .filter(installation::Column::Initialized.eq(false))
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;
        if claimed.rows_affected != 1 {
            return Err(ApplicationError::AlreadyConfigured);
        }

        let now = Utc::now();
        let organization_id = OrganizationId::new();
        let organization_model = organization::ActiveModel {
            id: Set(organization_id.to_string()),
            name: Set(input.organization_name.trim().to_string()),
            legal_name: Set(None),
            default_currency: Set("SAR".to_string()),
            timezone: Set("Asia/Riyadh".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        let user_model = insert_user(
            &transaction,
            organization_id,
            input.name,
            input.email,
            input.password,
            AccessRole::Admin,
            now,
        )
        .await?;
        organization_currency::ActiveModel {
            organization_id: Set(organization_id.to_string()),
            currency_code: Set("SAR".to_string()),
            is_enabled: Set(true),
            created_by: Set(Some(user_model.id.clone())),
            created_at: Set(now),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &user_model.id,
            "created",
            "organization",
            &organization_id.to_string(),
            serde_json::json!({"name": input.organization_name.trim()}),
        )
        .await?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &user_model.id,
            "created",
            "user",
            &user_model.id,
            serde_json::json!({"role": AccessRole::Admin.as_str()}),
        )
        .await?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &user_model.id,
            "enabled",
            "organization_currency",
            "SAR",
            serde_json::json!({"currency_code": "SAR", "is_enabled": true}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        Ok((
            restore_organization(organization_model)?,
            restore_user(user_model)?,
        ))
    }

    async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<UserAccount, ApplicationError> {
        let normalized_email = email.trim().to_ascii_lowercase();
        let model = user::Entity::find()
            .filter(user::Column::Email.eq(&normalized_email))
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::Authentication)?;
        if !model.is_active
            || model
                .locked_until
                .is_some_and(|locked_until| locked_until > Utc::now())
        {
            return Err(ApplicationError::Authentication);
        }
        let password_matches = verify_password(password, &model.password_hash)?;
        if !password_matches {
            let mut active = model.into_active_model();
            let attempts = match active.failed_login_attempts {
                sea_orm::ActiveValue::Unchanged(value) => value.saturating_add(1),
                sea_orm::ActiveValue::Set(value) => value.saturating_add(1),
                sea_orm::ActiveValue::NotSet => 1,
            };
            active.failed_login_attempts = Set(attempts);
            if attempts >= MAX_FAILED_LOGINS {
                active.locked_until = Set(Some(Utc::now() + Duration::minutes(LOCKOUT_MINUTES)));
            }
            active.updated_at = Set(Utc::now());
            active
                .update(&self.database)
                .await
                .map_err(persistence_error)?;
            return Err(ApplicationError::Authentication);
        }

        let mut active = model.into_active_model();
        active.failed_login_attempts = Set(0);
        active.locked_until = Set(None);
        active.last_login_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&self.database)
            .await
            .map_err(persistence_error)?;
        restore_user(model)
    }

    async fn get_active_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserAccount>, ApplicationError> {
        let model = user::Entity::find_by_id(user_id.to_string())
            .filter(user::Column::IsActive.eq(true))
            .one(&self.database)
            .await
            .map_err(persistence_error)?;
        model.map(restore_user).transpose()
    }

    async fn list_users(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<UserAccount>, ApplicationError> {
        let models = user::Entity::find()
            .filter(user::Column::OrganizationId.eq(organization_id.to_string()))
            .order_by_asc(user::Column::FullName)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        models.into_iter().map(restore_user).collect()
    }

    async fn create_user(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        input: NewUser<'_>,
    ) -> Result<UserAccount, ApplicationError> {
        let email = input.email.trim().to_ascii_lowercase();
        if user::Entity::find()
            .filter(user::Column::Email.eq(&email))
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .is_some()
        {
            return Err(crate::domain::DomainError::Validation(
                "A user with this email address already exists.",
            )
            .into());
        }
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let model = insert_user(
            &transaction,
            organization_id,
            input.name,
            &email,
            input.password,
            input.role,
            Utc::now(),
        )
        .await?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "user",
            &model.id,
            serde_json::json!({"role": input.role.as_str()}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_user(model)
    }

    async fn set_user_active(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
        actor_id: UserId,
        is_active: bool,
    ) -> Result<UserAccount, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let model = user::Entity::find_by_id(user_id.to_string())
            .filter(user::Column::OrganizationId.eq(organization_id.to_string()))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut active = model.into_active_model();
        active.is_active = Set(is_active);
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            if is_active {
                "activated"
            } else {
                "deactivated"
            },
            "user",
            &user_id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_user(model)
    }

    async fn update_user(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
        actor_id: UserId,
        input: UserUpdate<'_>,
    ) -> Result<UserAccount, ApplicationError> {
        let email = input.email.trim().to_ascii_lowercase();
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let model = user::Entity::find_by_id(user_id.to_string())
            .filter(user::Column::OrganizationId.eq(organization_id.to_string()))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if user::Entity::find()
            .filter(user::Column::Email.eq(&email))
            .filter(user::Column::Id.ne(user_id.to_string()))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some()
        {
            return Err(crate::domain::DomainError::Validation(
                "A user with this email address already exists.",
            )
            .into());
        }

        let mut active = model.into_active_model();
        active.full_name = Set(input.name.trim().to_string());
        active.email = Set(email);
        active.access_role = Set(input.role.as_str().to_string());
        active.updated_at = Set(Utc::now());
        if let Some(password) = input.password {
            active.password_hash = Set(hash_password(password)?);
            active.failed_login_attempts = Set(0);
            active.locked_until = Set(None);
        }
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "user",
            &user_id.to_string(),
            serde_json::json!({
                "role": input.role.as_str(),
                "password_changed": input.password.is_some()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_user(model)
    }
}

async fn insert_user(
    transaction: &sea_orm::DatabaseTransaction,
    organization_id: OrganizationId,
    name: &str,
    email: &str,
    password: &str,
    role: AccessRole,
    now: chrono::DateTime<Utc>,
) -> Result<user::Model, ApplicationError> {
    let password_hash = hash_password(password)?;
    user::ActiveModel {
        id: Set(UserId::new().to_string()),
        organization_id: Set(organization_id.to_string()),
        full_name: Set(name.trim().to_string()),
        email: Set(email.trim().to_ascii_lowercase()),
        password_hash: Set(password_hash),
        access_role: Set(role.as_str().to_string()),
        is_active: Set(true),
        failed_login_attempts: Set(0),
        locked_until: Set(None),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(transaction)
    .await
    .map_err(persistence_error)
}

pub(crate) fn hash_password(password: &str) -> Result<String, ApplicationError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| {
            tracing::error!(%error, "Password hash generation failed");
            ApplicationError::Persistence
        })
}

pub(crate) fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, ApplicationError> {
    let hash = PasswordHash::new(password_hash).map_err(|error| {
        tracing::error!(%error, "Stored password hash is invalid");
        ApplicationError::Persistence
    })?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok())
}

fn restore_user(model: user::Model) -> Result<UserAccount, ApplicationError> {
    let id = UserId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted user identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&model.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let role = AccessRole::parse(&model.access_role).ok_or_else(|| {
        tracing::error!(role = %model.access_role, "Persisted user role is invalid");
        ApplicationError::Persistence
    })?;
    Ok(UserAccount {
        id,
        organization_id,
        name: model.full_name,
        email: model.email,
        role,
        is_active: model.is_active,
        created_at: model.created_at,
    })
}

fn restore_organization(model: organization::Model) -> Result<Organization, ApplicationError> {
    let id = OrganizationId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    Ok(Organization {
        id,
        name: model.name,
        legal_name: model.legal_name,
        default_currency: model.default_currency,
        timezone: model.timezone,
    })
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Identity persistence operation failed");
    ApplicationError::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration;
    use sea_orm::Database;

    #[tokio::test]
    async fn bootstrap_login_lockout_and_admin_user_operations_work_end_to_end() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        migration::initialize(&database)
            .await
            .expect("initialize schema");
        let repository = SeaOrmIdentityRepository::new(&database);
        let service = crate::application::IdentityService::new(repository);
        assert!(service.setup_status().await.expect("setup status"));

        let (organization, admin) = service
            .bootstrap_admin(OrganizationBootstrap {
                organization_name: "AqarDesk",
                name: "Admin",
                email: " ADMIN@example.com ",
                password: "a-secure-password-12",
            })
            .await
            .expect("bootstrap admin");
        assert_eq!(admin.email, "admin@example.com");
        let default_currency = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization.id.to_string()))
            .one(&database)
            .await
            .expect("load organization currency")
            .expect("default currency is enabled");
        assert_eq!(default_currency.currency_code, "SAR");
        assert!(default_currency.is_enabled);
        assert!(!service.setup_status().await.expect("setup status"));
        assert!(matches!(
            service
                .bootstrap_admin(OrganizationBootstrap {
                    organization_name: "Another",
                    name: "Other",
                    email: "other@example.com",
                    password: "another-secure-password",
                })
                .await,
            Err(ApplicationError::AlreadyConfigured)
        ));
        assert!(matches!(
            service.login("admin@example.com", "wrong-password").await,
            Err(ApplicationError::Authentication)
        ));
        let authenticated = service
            .login("admin@example.com", "a-secure-password-12")
            .await
            .expect("login");
        let actor =
            crate::domain::ActorContext::new(authenticated.id, organization.id, authenticated.role);
        let created = service
            .create_user(
                actor,
                NewUser {
                    name: "Read Only",
                    email: "readonly@example.com",
                    password: "another-secure-password",
                    role: AccessRole::Readonly,
                },
            )
            .await
            .expect("create read-only user");
        assert_eq!(created.role, AccessRole::Readonly);
        let updated = service
            .update_user(
                actor,
                created.id,
                crate::application::UserUpdate {
                    name: "Updated Read Only",
                    email: "updated-readonly@example.com",
                    password: Some("updated-secure-password"),
                    role: AccessRole::Staff,
                },
            )
            .await
            .expect("update user");
        assert_eq!(updated.name, "Updated Read Only");
        assert_eq!(updated.email, "updated-readonly@example.com");
        assert_eq!(updated.role, AccessRole::Staff);
        assert_eq!(
            service
                .login("updated-readonly@example.com", "updated-secure-password")
                .await
                .expect("login with the reset password")
                .id,
            updated.id
        );
        assert!(
            service
                .update_user(
                    actor,
                    updated.id,
                    crate::application::UserUpdate {
                        name: "Duplicate",
                        email: "admin@example.com",
                        password: None,
                        role: AccessRole::Staff,
                    },
                )
                .await
                .is_err(),
            "user emails must remain unique"
        );
        assert_eq!(
            service.list_users(actor).await.expect("list users").len(),
            2
        );
        service
            .set_user_active(actor, created.id, false)
            .await
            .expect("deactivate user");
        assert!(service.current_user(created.id).await.is_err());
        for _ in 0..5 {
            assert!(matches!(
                service.login("admin@example.com", "wrong-password").await,
                Err(ApplicationError::Authentication)
            ));
        }
        assert!(matches!(
            service
                .login("admin@example.com", "a-secure-password-12")
                .await,
            Err(ApplicationError::Authentication)
        ));
    }
}
