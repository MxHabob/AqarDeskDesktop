use crate::{
    application::{
        ApplicationError, IdentityRepository, NewUser, OrganizationBootstrap, Page, PageRequest,
        UserUpdate,
    },
    domain::{
        AccessRole, Organization, OrganizationId, Permission, RolePermissionSet, UserAccount,
        UserId,
    },
    infrastructure::{
        audit,
        contract_template_repository::seed_onboarding_templates,
        models::{
            installation, organization, organization_currency, organization_role, role_permission,
            user, user_session,
        },
    },
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{Duration, Utc};
use rand::RngCore;
use rand::rngs::OsRng;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;
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
    async fn list_role_permissions(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<RolePermissionSet>, ApplicationError> {
        load_role_permissions(&self.database, organization_id).await
    }

    async fn update_role_permissions(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        role_id: &str,
        permissions: Vec<Permission>,
    ) -> Result<RolePermissionSet, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let role = organization_role::Entity::find()
            .filter(organization_role::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(organization_role::Column::RoleId.eq(role_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if role.system_role.as_deref() == Some("admin") {
            return Err(crate::domain::DomainError::Validation(
                "Administrator permissions are fixed and cannot be changed.",
            )
            .into());
        }
        role_permission::Entity::delete_many()
            .filter(role_permission::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(role_permission::Column::RoleId.eq(role_id))
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;

        let now = Utc::now();
        for permission in editable_permissions() {
            role_permission::ActiveModel {
                organization_id: Set(organization_id.to_string()),
                role_id: Set(role_id.to_string()),
                permission_code: Set(permission.as_str().to_string()),
                is_enabled: Set(permissions.contains(&permission)),
                updated_at: Set(now),
                updated_by: Set(Some(actor_id.to_string())),
            }
            .insert(&transaction)
            .await
            .map_err(persistence_error)?;
        }

        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "role_permissions",
            role_id,
            serde_json::json!({
                "role_id": role_id,
                "permissions": permissions.iter().map(|permission| permission.as_str()).collect::<Vec<_>>()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;

        let role_permissions = load_role_permissions(&self.database, organization_id).await?;
        role_permissions
            .into_iter()
            .find(|set| set.role_id == role_id)
            .ok_or(ApplicationError::Persistence)
    }

    async fn create_role(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        name: &str,
        permissions: Vec<Permission>,
    ) -> Result<RolePermissionSet, ApplicationError> {
        let permissions = editable_permissions()
            .into_iter()
            .filter(|permission| permissions.contains(permission))
            .collect::<Vec<_>>();
        let normalized_name = name.trim().to_lowercase();
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        if organization_role::Entity::find()
            .filter(organization_role::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(organization_role::Column::NormalizedName.eq(&normalized_name))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some()
        {
            return Err(crate::domain::DomainError::Validation(
                "A role with this name already exists.",
            )
            .into());
        }
        let role_id = uuid::Uuid::new_v4().to_string();
        organization_role::ActiveModel {
            organization_id: Set(organization_id.to_string()),
            role_id: Set(role_id.clone()),
            name: Set(name.trim().to_string()),
            normalized_name: Set(normalized_name),
            is_system: Set(false),
            system_role: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        let now = Utc::now();
        for permission in editable_permissions() {
            role_permission::ActiveModel {
                organization_id: Set(organization_id.to_string()),
                role_id: Set(role_id.clone()),
                permission_code: Set(permission.as_str().to_string()),
                is_enabled: Set(permissions.contains(&permission)),
                updated_at: Set(now),
                updated_by: Set(Some(actor_id.to_string())),
            }
            .insert(&transaction)
            .await
            .map_err(persistence_error)?;
        }
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "role",
            &role_id,
            serde_json::json!({
                "name": name.trim(),
                "permissions": permissions.iter().map(|permission| permission.as_str()).collect::<Vec<_>>()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        Ok(RolePermissionSet {
            role_id,
            name: name.trim().to_string(),
            is_system: false,
            system_role: None,
            permissions,
        })
    }

    async fn delete_role(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        role_id: &str,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let role = organization_role::Entity::find()
            .filter(organization_role::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(organization_role::Column::RoleId.eq(role_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if role.is_system {
            return Err(
                crate::domain::DomainError::Validation("System roles cannot be deleted.").into(),
            );
        }
        if user::Entity::find()
            .filter(user::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(user::Column::RoleId.eq(role_id))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some()
        {
            return Err(crate::domain::DomainError::Validation(
                "A role assigned to users cannot be deleted.",
            )
            .into());
        }
        organization_role::Entity::delete_by_id((organization_id.to_string(), role_id.to_string()))
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "deleted",
            "role",
            role_id,
            serde_json::json!({"name": role.name}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }

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
            logo_data_url: Set(None),
            default_currency: Set("SAR".to_string()),
            timezone: Set("Asia/Riyadh".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        seed_system_roles(&transaction, organization_id, now).await?;
        let user_model = insert_user(
            &transaction,
            organization_id,
            input.name,
            input.email,
            input.password,
            AccessRole::Admin,
            AccessRole::Admin.as_str(),
            now,
        )
        .await?;
        for currency_code in ["SAR", "YER", "USD"] {
            organization_currency::ActiveModel {
                organization_id: Set(organization_id.to_string()),
                currency_code: Set(currency_code.to_string()),
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
                "enabled",
                "organization_currency",
                currency_code,
                serde_json::json!({"currency_code": currency_code, "is_enabled": true}),
            )
            .await?;
        }
        seed_onboarding_templates(
            &transaction,
            &organization_id.to_string(),
            &user_model.id,
            now,
        )
        .await?;
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
        let user = restore_user(model)?;
        hydrate_user_permissions(&self.database, user).await
    }

    async fn create_session(
        &self,
        user_id: UserId,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<String, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let user_exists = user::Entity::find_by_id(user_id.to_string())
            .filter(user::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .is_some();
        if !user_exists {
            return Err(ApplicationError::Authentication);
        }

        let now = Utc::now();
        user_session::Entity::delete_many()
            .filter(user_session::Column::ExpiresAt.lte(now))
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;

        let token = generate_session_token();
        user_session::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id.to_string()),
            token_hash: Set(hash_session_token(&token)),
            created_at: Set(now),
            expires_at: Set(expires_at),
            revoked_at: Set(None),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        transaction.commit().await.map_err(persistence_error)?;
        Ok(token)
    }

    async fn resume_session(&self, token: &str) -> Result<Option<UserAccount>, ApplicationError> {
        if token.is_empty() {
            return Ok(None);
        }
        let now = Utc::now();
        let session = user_session::Entity::find()
            .filter(user_session::Column::TokenHash.eq(hash_session_token(token)))
            .filter(user_session::Column::RevokedAt.is_null())
            .filter(user_session::Column::ExpiresAt.gt(now))
            .one(&self.database)
            .await
            .map_err(persistence_error)?;
        let Some(session) = session else {
            return Ok(None);
        };
        let user_id = UserId::parse(&session.user_id).map_err(|error| {
            tracing::error!(%error, "Persisted session user identifier is invalid");
            ApplicationError::Persistence
        })?;
        let model = user::Entity::find_by_id(user_id.to_string())
            .filter(user::Column::IsActive.eq(true))
            .one(&self.database)
            .await
            .map_err(persistence_error)?;
        let Some(model) = model else {
            revoke_user_sessions(&self.database, &session.user_id, now).await?;
            return Ok(None);
        };
        let user = restore_user(model)?;
        hydrate_user_permissions(&self.database, user)
            .await
            .map(Some)
    }

    async fn revoke_session(&self, token: &str) -> Result<(), ApplicationError> {
        if token.is_empty() {
            return Ok(());
        }
        user_session::Entity::update_many()
            .col_expr(user_session::Column::RevokedAt, Expr::value(Utc::now()))
            .filter(user_session::Column::TokenHash.eq(hash_session_token(token)))
            .filter(user_session::Column::RevokedAt.is_null())
            .exec(&self.database)
            .await
            .map_err(persistence_error)?;
        Ok(())
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
        let mut users = models
            .into_iter()
            .map(restore_user)
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(first) = users.first() {
            let policies = load_role_permissions(&self.database, first.organization_id).await?;
            for user in &mut users {
                if let Some(policy) = policies
                    .iter()
                    .find(|policy| policy.role_id == user.role_id)
                {
                    user.permissions = policy.permissions.clone();
                    user.role_name = (!policy.is_system).then(|| policy.name.clone());
                    user.role = policy.system_role.unwrap_or(AccessRole::Staff);
                }
            }
        }
        Ok(users)
    }

    async fn list_users_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<UserAccount>, ApplicationError> {
        let models = user::Entity::find()
            .filter(user::Column::OrganizationId.eq(organization_id.to_string()))
            .order_by_asc(user::Column::FullName)
            .order_by_asc(user::Column::Id)
            .limit(page.fetch_limit())
            .offset(page.offset())
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut users = models
            .into_iter()
            .map(restore_user)
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(first) = users.first() {
            let policies = load_role_permissions(&self.database, first.organization_id).await?;
            for user in &mut users {
                if let Some(policy) = policies
                    .iter()
                    .find(|policy| policy.role_id == user.role_id)
                {
                    user.permissions = policy.permissions.clone();
                    user.role_name = (!policy.is_system).then(|| policy.name.clone());
                    user.role = policy.system_role.unwrap_or(AccessRole::Staff);
                }
            }
        }
        Ok(Page::from_items(users, page.limit()))
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
        let role = find_role(&transaction, organization_id, input.role_id).await?;
        let model = insert_user(
            &transaction,
            organization_id,
            input.name,
            &email,
            input.password,
            role.system_role
                .as_deref()
                .and_then(AccessRole::parse)
                .unwrap_or(AccessRole::Staff),
            input.role_id,
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
            serde_json::json!({"role_id": input.role_id}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        let user = restore_user(model)?;
        hydrate_user_permissions(&self.database, user).await
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
        if !is_active {
            revoke_user_sessions(&transaction, &user_id.to_string(), Utc::now()).await?;
        }
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
        let user = restore_user(model)?;
        hydrate_user_permissions(&self.database, user).await
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
        let role = find_role(&transaction, organization_id, input.role_id).await?;
        let base_role = role
            .system_role
            .as_deref()
            .and_then(AccessRole::parse)
            .unwrap_or(AccessRole::Staff);
        active.access_role = Set(base_role.as_str().to_string());
        active.role_id = Set(input.role_id.to_string());
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
        if input.password.is_some() {
            revoke_user_sessions(&transaction, &user_id.to_string(), Utc::now()).await?;
        }
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "user",
            &user_id.to_string(),
            serde_json::json!({
                "role_id": input.role_id,
                "password_changed": input.password.is_some()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        let user = restore_user(model)?;
        hydrate_user_permissions(&self.database, user).await
    }
}

async fn find_role<C: sea_orm::ConnectionTrait>(
    connection: &C,
    organization_id: OrganizationId,
    role_id: &str,
) -> Result<organization_role::Model, ApplicationError> {
    organization_role::Entity::find()
        .filter(organization_role::Column::OrganizationId.eq(organization_id.to_string()))
        .filter(organization_role::Column::RoleId.eq(role_id))
        .one(connection)
        .await
        .map_err(persistence_error)?
        .ok_or(ApplicationError::NotFound)
}

async fn seed_system_roles(
    transaction: &sea_orm::DatabaseTransaction,
    organization_id: OrganizationId,
    now: chrono::DateTime<Utc>,
) -> Result<(), ApplicationError> {
    for role in [AccessRole::Admin, AccessRole::Staff, AccessRole::Readonly] {
        organization_role::ActiveModel {
            organization_id: Set(organization_id.to_string()),
            role_id: Set(role.as_str().to_string()),
            name: Set(role.as_str().to_string()),
            normalized_name: Set(role.as_str().to_string()),
            is_system: Set(true),
            system_role: Set(Some(role.as_str().to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(transaction)
        .await
        .map_err(persistence_error)?;

        for permission in editable_permissions() {
            role_permission::ActiveModel {
                organization_id: Set(organization_id.to_string()),
                role_id: Set(role.as_str().to_string()),
                permission_code: Set(permission.as_str().to_string()),
                is_enabled: Set(role.default_permissions().contains(&permission)),
                updated_at: Set(now),
                updated_by: Set(None),
            }
            .insert(transaction)
            .await
            .map_err(persistence_error)?;
        }
    }
    Ok(())
}

async fn insert_user(
    transaction: &sea_orm::DatabaseTransaction,
    organization_id: OrganizationId,
    name: &str,
    email: &str,
    password: &str,
    role: AccessRole,
    role_id: &str,
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
        role_id: Set(role_id.to_string()),
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
        role_id: model.role_id,
        role_name: None,
        permissions: role.default_permissions(),
        is_active: model.is_active,
        created_at: model.created_at,
    })
}

async fn hydrate_user_permissions<C: sea_orm::ConnectionTrait>(
    database: &C,
    mut user: UserAccount,
) -> Result<UserAccount, ApplicationError> {
    let policies = load_role_permissions(database, user.organization_id).await?;
    let policy = policies
        .into_iter()
        .find(|policy| policy.role_id == user.role_id)
        .ok_or(ApplicationError::Persistence)?;
    user.permissions = policy.permissions;
    user.role_name = (!policy.is_system).then(|| policy.name.clone());
    user.role = policy.system_role.unwrap_or(AccessRole::Staff);
    Ok(user)
}

async fn load_role_permissions<C: sea_orm::ConnectionTrait>(
    database: &C,
    organization_id: OrganizationId,
) -> Result<Vec<RolePermissionSet>, ApplicationError> {
    let roles = organization_role::Entity::find()
        .filter(organization_role::Column::OrganizationId.eq(organization_id.to_string()))
        .order_by_desc(organization_role::Column::IsSystem)
        .order_by_asc(organization_role::Column::Name)
        .all(database)
        .await
        .map_err(persistence_error)?;
    let mut policies = roles
        .into_iter()
        .map(|role| {
            let system_role = role.system_role.as_deref().and_then(AccessRole::parse);
            let mut permissions = system_role
                .map(AccessRole::default_permissions)
                .unwrap_or_default();
            if system_role == Some(AccessRole::Admin)
                && !permissions.contains(&Permission::ManageAdministration)
            {
                permissions.push(Permission::ManageAdministration);
            }
            RolePermissionSet {
                role_id: role.role_id,
                name: role.name,
                is_system: role.is_system,
                system_role,
                permissions,
            }
        })
        .collect::<Vec<_>>();
    let models = role_permission::Entity::find()
        .filter(role_permission::Column::OrganizationId.eq(organization_id.to_string()))
        .all(database)
        .await
        .map_err(persistence_error)?;

    for model in models {
        let permission = Permission::parse(&model.permission_code).ok_or_else(|| {
            tracing::error!(permission = %model.permission_code, "Persisted role permission is invalid");
            ApplicationError::Persistence
        })?;
        let policy = policies
            .iter_mut()
            .find(|policy| policy.role_id == model.role_id)
            .ok_or_else(|| {
                tracing::error!(role_id = %model.role_id, "Persisted role permission references a missing role");
                ApplicationError::Persistence
            })?;
        policy
            .permissions
            .retain(|candidate| *candidate != permission);
        if model.is_enabled {
            policy.permissions.push(permission);
        }
    }

    for policy in &mut policies {
        let is_admin = policy.system_role == Some(AccessRole::Admin);
        policy.permissions = editable_permissions()
            .into_iter()
            .filter(|permission| policy.permissions.contains(permission))
            .collect();
        if is_admin
            && !policy
                .permissions
                .contains(&Permission::ManageAdministration)
        {
            policy.permissions.push(Permission::ManageAdministration);
        }
    }
    Ok(policies)
}

fn editable_permissions() -> [Permission; 20] {
    Permission::BUSINESS
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

fn generate_session_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_session_token(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn revoke_user_sessions<C: sea_orm::ConnectionTrait>(
    connection: &C,
    user_id: &str,
    revoked_at: chrono::DateTime<Utc>,
) -> Result<(), ApplicationError> {
    user_session::Entity::update_many()
        .col_expr(user_session::Column::RevokedAt, Expr::value(revoked_at))
        .filter(user_session::Column::UserId.eq(user_id))
        .filter(user_session::Column::RevokedAt.is_null())
        .exec(connection)
        .await
        .map_err(persistence_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration;
    use sea_orm::{ConnectionTrait, Database};

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
        assert_eq!(organization.default_currency, "SAR");
        let enabled_currency_codes = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization.id.to_string()))
            .filter(organization_currency::Column::IsEnabled.eq(true))
            .all(&database)
            .await
            .expect("list default currencies")
            .into_iter()
            .map(|currency| currency.currency_code)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            enabled_currency_codes,
            ["SAR", "USD", "YER"]
                .into_iter()
                .map(str::to_string)
                .collect()
        );
        let seeded_template_count = database
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                format!(
                    "SELECT COUNT(*) AS count FROM contract_templates WHERE organization_id = '{}'",
                    organization.id
                ),
            ))
            .await
            .expect("count onboarding templates")
            .expect("template count exists")
            .try_get_by::<i64, _>("count")
            .expect("read template count");
        assert_eq!(seeded_template_count, 8);
        let populated_template_count = database
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                format!(
                    "SELECT COUNT(*) AS count FROM contract_template_versions v JOIN contract_templates t ON t.id = v.template_id WHERE t.organization_id = '{}' AND t.is_active = FALSE AND LENGTH(TRIM(v.content_body)) > 0",
                    organization.id
                ),
            ))
            .await
            .expect("count populated onboarding templates")
            .expect("populated template count exists")
            .try_get_by::<i64, _>("count")
            .expect("read populated template count");
        assert_eq!(populated_template_count, 6);
        let starter_contents = database
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                format!(
                    "SELECT v.content_body FROM contract_template_versions v JOIN contract_templates t ON t.id = v.template_id WHERE t.organization_id = '{}' ORDER BY t.template_key",
                    organization.id
                ),
            ))
            .await
            .expect("read onboarding starter content");
        let starter_contents = starter_contents
            .iter()
            .map(|row| {
                row.try_get_by::<String, _>("content_body")
                    .expect("read starter body")
            })
            .collect::<Vec<_>>();
        let all_starter_content = starter_contents.join("\n");
        assert!(
            starter_contents
                .iter()
                .all(|content| content.contains("{{")),
            "every starter template should include data fields"
        );
        for variable in [
            "{{tenant.full_name}}",
            "{{tenant.email}}",
            "{{tenant.phone}}",
            "{{tenant.address}}",
            "{{property.name}}",
            "{{property.code}}",
            "{{property.address}}",
            "{{property.city}}",
            "{{unit.number}}",
            "{{unit.type}}",
            "{{unit.floor}}",
            "{{unit.area}}",
            "{{unit.bedrooms}}",
            "{{unit.bathrooms}}",
            "{{unit.rent}}",
            "{{unit.currency}}",
            "{{contract.number}}",
            "{{contract.date}}",
            "{{contract.start_date}}",
            "{{contract.end_date}}",
            "{{contract.deposit}}",
            "{{organization.name}}",
        ] {
            assert!(
                all_starter_content.contains(variable),
                "starter templates should include the {variable} field"
            );
        }
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
        assert_eq!(authenticated.token.len(), 64);
        assert_ne!(
            hash_session_token(&authenticated.token),
            authenticated.token
        );
        assert_eq!(
            service
                .resume_session(&authenticated.token)
                .await
                .expect("resume authenticated session")
                .expect("session is active")
                .id,
            authenticated.user.id
        );
        let expired_token = SeaOrmIdentityRepository::new(&database)
            .create_session(authenticated.user.id, Utc::now() - Duration::seconds(1))
            .await
            .expect("create expired session");
        assert!(
            service
                .resume_session(&expired_token)
                .await
                .expect("check expired session")
                .is_none(),
            "expired sessions must not resume"
        );
        let actor = crate::domain::ActorContext::new(
            authenticated.user.id,
            organization.id,
            authenticated.user.role,
        );
        let role_permissions = service
            .list_role_permissions(actor)
            .await
            .expect("list role permissions");
        assert_eq!(
            role_permissions
                .iter()
                .find(|policy| policy.role_id == "staff")
                .expect("staff permissions")
                .permissions,
            AccessRole::Staff.default_permissions()
        );
        service
            .update_role_permissions(actor, "staff", vec![Permission::ViewProperties])
            .await
            .expect("save staff permissions");
        let sales_role = service
            .create_role(
                actor,
                "Sales",
                vec![
                    Permission::ViewPayments,
                    Permission::ViewTenants,
                    Permission::RecordPayments,
                ],
            )
            .await
            .expect("create custom sales role");
        assert_eq!(
            sales_role.permissions,
            vec![
                Permission::ViewTenants,
                Permission::ViewPayments,
                Permission::RecordPayments
            ]
        );
        let sales_user = service
            .create_user(
                actor,
                NewUser {
                    name: "Sales",
                    email: "sales@example.com",
                    password: "another-secure-password",
                    role_id: &sales_role.role_id,
                },
            )
            .await
            .expect("create sales user");
        assert_eq!(sales_user.role_name.as_deref(), Some("Sales"));
        assert_eq!(
            sales_user.permissions,
            vec![
                Permission::ViewTenants,
                Permission::ViewPayments,
                Permission::RecordPayments
            ]
        );
        let sales_actor = crate::domain::ActorContext::with_permissions(
            sales_user.id,
            sales_user.organization_id,
            sales_user.role,
            sales_user.permissions.clone(),
        );
        assert!(sales_actor.has_permission(Permission::RecordPayments));
        assert!(!sales_actor.has_permission(Permission::ViewProperties));
        let created = service
            .create_user(
                actor,
                NewUser {
                    name: "Read Only",
                    email: "readonly@example.com",
                    password: "another-secure-password",
                    role_id: "readonly",
                },
            )
            .await
            .expect("create read-only user");
        assert_eq!(created.role, AccessRole::Readonly);
        let created_session = service
            .start_session(created.id)
            .await
            .expect("create session before password change");
        let updated = service
            .update_user(
                actor,
                created.id,
                crate::application::UserUpdate {
                    name: "Updated Read Only",
                    email: "updated-readonly@example.com",
                    password: Some("updated-secure-password"),
                    role_id: "staff",
                },
            )
            .await
            .expect("update user");
        assert_eq!(updated.name, "Updated Read Only");
        assert_eq!(updated.email, "updated-readonly@example.com");
        assert_eq!(updated.role, AccessRole::Staff);
        assert_eq!(updated.permissions, vec![Permission::ViewProperties]);
        assert!(
            service
                .resume_session(&created_session)
                .await
                .expect("check password-change session")
                .is_none(),
            "password changes should revoke existing sessions"
        );
        assert!(
            !crate::domain::ActorContext::with_permissions(
                updated.id,
                updated.organization_id,
                updated.role,
                updated.permissions.clone(),
            )
            .has_permission(Permission::RecordPayments)
        );
        assert_eq!(
            service
                .list_role_permissions(actor)
                .await
                .expect("read saved role permissions")
                .into_iter()
                .find(|policy| policy.role_id == "staff")
                .expect("saved staff permissions")
                .permissions,
            vec![Permission::ViewProperties]
        );
        let updated_session = service
            .login("updated-readonly@example.com", "updated-secure-password")
            .await
            .expect("login with the reset password");
        assert_eq!(updated_session.user.id, updated.id);
        assert!(
            service
                .update_user(
                    actor,
                    updated.id,
                    crate::application::UserUpdate {
                        name: "Duplicate",
                        email: "admin@example.com",
                        password: None,
                        role_id: "staff",
                    },
                )
                .await
                .is_err(),
            "user emails must remain unique"
        );
        assert_eq!(
            service.list_users(actor).await.expect("list users").len(),
            3
        );
        assert!(
            service
                .delete_role(actor, &sales_role.role_id)
                .await
                .is_err()
        );
        let created_user_session = service
            .start_session(created.id)
            .await
            .expect("create session before deactivation");
        service
            .set_user_active(actor, created.id, false)
            .await
            .expect("deactivate user");
        assert!(
            service
                .resume_session(&created_user_session)
                .await
                .expect("check deactivated user's session")
                .is_none(),
            "deactivation should revoke existing sessions"
        );
        service
            .end_session(&authenticated.token)
            .await
            .expect("logout authenticated user");
        assert!(
            service
                .resume_session(&authenticated.token)
                .await
                .expect("check logged-out session")
                .is_none(),
            "revoked session should not resume"
        );
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
