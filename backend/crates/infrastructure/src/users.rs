use crate::models::{organization, organization_currency, user};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::Utc;
use rand::rngs::OsRng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

fn hash_password(password: String) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| format!("Failed to hash password: {error}"))
}

fn verify_password(password: String, password_hash: String) -> Result<bool, String> {
    let parsed = PasswordHash::new(&password_hash)
        .map_err(|error| format!("Stored password hash is invalid: {error}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn verify_password_value(password: &str, password_hash: &str) -> Result<bool, String> {
    verify_password(password.to_string(), password_hash.to_string())
}

pub fn hash_password_value(password: &str) -> Result<String, String> {
    hash_password(password.to_string())
}

pub async fn has_users(database: &DatabaseConnection) -> Result<bool, String> {
    user::Entity::find()
        .one(database)
        .await
        .map(|user| user.is_some())
        .map_err(|error| format!("Failed to check configured users: {error}"))
}

pub async fn create_initial_admin(
    database: &DatabaseConnection,
    organization_name: &str,
    name: &str,
    email: &str,
    password: &str,
) -> Result<UserSummary, String> {
    validate_user_input(name, email, password)?;
    if organization_name.trim().is_empty() {
        return Err("Organization name is required.".to_string());
    }

    let transaction = database
        .begin()
        .await
        .map_err(|error| format!("Failed to start initial setup: {error}"))?;
    if user::Entity::find()
        .one(&transaction)
        .await
        .map_err(|error| format!("Failed to check initial setup status: {error}"))?
        .is_some()
    {
        return Err("Initial administrator has already been created.".to_string());
    }

    let organization_id = Uuid::new_v4().to_string();
    organization::ActiveModel {
        id: Set(organization_id.clone()),
        name: Set(organization_name.trim().to_string()),
        legal_name: Set(None),
        default_currency: Set("SAR".to_string()),
        timezone: Set("Asia/Riyadh".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to create organization: {error}"))?;
    organization_currency::ActiveModel {
        organization_id: Set(organization_id.clone()),
        currency_code: Set("SAR".to_string()),
        is_enabled: Set(true),
        created_by: Set(None),
        created_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to enable the default currency: {error}"))?;

    let password_hash = hash_password(password.to_string())?;
    let model = user::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id),
        full_name: Set(name.trim().to_string()),
        email: Set(email.trim().to_lowercase()),
        password_hash: Set(password_hash),
        access_role: Set("admin".to_string()),
        is_active: Set(true),
        failed_login_attempts: Set(0),
        locked_until: Set(None),
        last_login_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to create administrator: {error}"))?;

    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to save initial setup: {error}"))?;

    Ok(summary(model))
}

pub async fn authenticate(
    database: &DatabaseConnection,
    email: &str,
    password: &str,
) -> Result<UserSummary, String> {
    let model = user::Entity::find()
        .filter(user::Column::Email.eq(email.trim().to_lowercase()))
        .one(database)
        .await
        .map_err(|error| format!("Failed to find account: {error}"))?
        .ok_or_else(|| "Email or password is incorrect.".to_string())?;

    if !model.is_active || !verify_password(password.to_string(), model.password_hash.clone())? {
        return Err("Email or password is incorrect.".to_string());
    }

    Ok(summary(model))
}

pub async fn get_active_user(
    database: &DatabaseConnection,
    user_id: &str,
) -> Result<Option<UserSummary>, String> {
    user::Entity::find_by_id(user_id)
        .one(database)
        .await
        .map(|model| model.filter(|user| user.is_active).map(summary))
        .map_err(|error| format!("Failed to restore the saved user session: {error}"))
}

pub async fn list_users(database: &DatabaseConnection) -> Result<Vec<UserSummary>, String> {
    user::Entity::find()
        .order_by_asc(user::Column::CreatedAt)
        .all(database)
        .await
        .map(|models| models.into_iter().map(summary).collect())
        .map_err(|error| format!("Failed to list users: {error}"))
}

pub async fn active_admin_count(database: &DatabaseConnection) -> Result<u64, String> {
    user::Entity::find()
        .filter(user::Column::AccessRole.eq("admin"))
        .filter(user::Column::IsActive.eq(true))
        .count(database)
        .await
        .map_err(|error| format!("Failed to check active administrators: {error}"))
}

pub async fn create_user(
    database: &DatabaseConnection,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Result<UserSummary, String> {
    validate_user_input(name, email, password)?;
    if !matches!(role, "admin" | "staff" | "readonly") {
        return Err("Role must be admin, staff, or readonly.".to_string());
    }

    let organization_id = organization::Entity::find()
        .one(database)
        .await
        .map_err(|error| format!("Failed to find the current organization: {error}"))?
        .ok_or_else(|| "Complete initial setup before creating users.".to_string())?
        .id;

    user::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id),
        full_name: Set(name.trim().to_string()),
        email: Set(email.trim().to_lowercase()),
        password_hash: Set(hash_password(password.to_string())?),
        access_role: Set(role.to_string()),
        is_active: Set(true),
        failed_login_attempts: Set(0),
        locked_until: Set(None),
        last_login_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(database)
    .await
    .map(summary)
    .map_err(|error| format!("Failed to create user: {error}"))
}

pub async fn set_user_active(
    database: &DatabaseConnection,
    user_id: &str,
    is_active: bool,
) -> Result<UserSummary, String> {
    let model = user::Entity::find_by_id(user_id)
        .one(database)
        .await
        .map_err(|error| format!("Failed to find user: {error}"))?
        .ok_or_else(|| "User was not found.".to_string())?;
    let mut active: user::ActiveModel = model.into();
    active.is_active = Set(is_active);
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map(summary)
        .map_err(|error| format!("Failed to update user: {error}"))
}

fn validate_user_input(name: &str, email: &str, password: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Name is required.".to_string());
    }
    if !email.trim().contains('@') || email.trim().len() > 254 {
        return Err("A valid email address is required.".to_string());
    }
    if password.chars().count() < 12 {
        return Err("Password must contain at least 12 characters.".to_string());
    }
    Ok(())
}

fn summary(model: user::Model) -> UserSummary {
    UserSummary {
        id: model.id,
        organization_id: model.organization_id,
        name: model.full_name,
        email: model.email,
        role: model.access_role,
        is_active: model.is_active,
        created_at: model.created_at.to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{organization_currency, user};
    use sea_orm::{Database, EntityTrait};
    use sea_orm_migration::MigratorTrait;

    #[tokio::test]
    async fn initial_setup_enables_default_currency_and_allows_login() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        crate::migration::Migrator::up(&database, None)
            .await
            .expect("apply migrations");

        assert!(!has_users(&database).await.expect("check initial setup"));
        let admin = create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create initial administrator");
        assert_eq!(admin.role, "admin");
        assert!(!admin.organization_id.is_empty());

        let stored_admin = user::Entity::find_by_id(&admin.id)
            .one(&database)
            .await
            .expect("query administrator")
            .expect("administrator exists");
        let enabled_currency_count = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(stored_admin.organization_id))
            .count(&database)
            .await
            .expect("count enabled currencies");
        assert_eq!(enabled_currency_count, 1);

        let authenticated = authenticate(&database, "ADMIN@example.com", "a-secure-password")
            .await
            .expect("authenticate initial administrator");
        assert_eq!(authenticated.id, admin.id);
        assert_eq!(authenticated.organization_id, admin.organization_id);
        assert!(
            create_initial_admin(
                &database,
                "Another Organization",
                "Second Admin",
                "second@example.com",
                "a-secure-password",
            )
            .await
            .is_err(),
            "initial setup must not create another organization administrator"
        );
    }
}
