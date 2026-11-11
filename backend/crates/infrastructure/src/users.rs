use crate::models::{
    audit_event, contract_template, contract_template_version, organization, organization_currency,
    user,
};
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
use sha2::{Digest, Sha256};
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

    let template_id = Uuid::new_v4().to_string();
    let template_body = "مسودة عقد إيجار سكني — يرجى مراجعتها واستكمال بياناتها واعتمادها من الجهة المختصة قبل استخدامها.\n\n\
المؤجر: {{organization_name}}\n\
المستأجر: {{tenant_name}}\n\
العقار: {{property_name}}\n\
الوحدة: {{unit_number}}\n\
مدة الإيجار: من {{start_date}} إلى {{end_date}}\n\
قيمة الإيجار: {{rent_amount}} {{currency_code}}\n\
تاريخ الاستحقاق: اليوم {{due_day}} من كل فترة\n\
مبلغ التأمين: {{deposit_amount}} {{currency_code}}\n\n\
الشروط الإضافية:\n{{terms}}";
    let placeholders = serde_json::json!({
        "organization_name": "string",
        "tenant_name": "string",
        "property_name": "string",
        "unit_number": "string",
        "start_date": "date",
        "end_date": "date",
        "rent_amount": "money",
        "currency_code": "string",
        "due_day": "integer",
        "deposit_amount": "money",
        "terms": "string"
    });
    contract_template::ActiveModel {
        id: Set(template_id.clone()),
        organization_id: Set(model.organization_id.clone()),
        template_key: Set("residential-default-ar".to_string()),
        name: Set("قالب عقد إيجار سكني (مسودة)".to_string()),
        description: Set(Some(
            "قالب أولي قابل للتعديل؛ ليس عقداً قانونياً معتمداً.".to_string(),
        )),
        lease_type: Set("residential".to_string()),
        locale: Set("ar".to_string()),
        is_active: Set(true),
        created_by: Set(Some(model.id.clone())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        archived_at: Set(None),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to create default contract template: {error}"))?;
    contract_template_version::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(model.organization_id.clone()),
        template_id: Set(template_id),
        version_number: Set(1),
        content_format: Set("text".to_string()),
        content_body: Set(Some(template_body.to_string())),
        source_storage_key: Set(None),
        placeholders_json: Set(placeholders.to_string()),
        content_sha256: Set(format!("{:x}", Sha256::digest(template_body.as_bytes()))),
        change_note: Set(Some("Initial onboarding template".to_string())),
        created_by: Set(Some(model.id.clone())),
        created_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to create default contract template version: {error}"))?;

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

pub async fn list_users(
    database: &DatabaseConnection,
    organization_id: &str,
) -> Result<Vec<UserSummary>, String> {
    user::Entity::find()
        .filter(user::Column::OrganizationId.eq(organization_id))
        .order_by_asc(user::Column::CreatedAt)
        .all(database)
        .await
        .map(|models| models.into_iter().map(summary).collect())
        .map_err(|error| format!("Failed to list users: {error}"))
}

pub async fn active_admin_count(
    database: &DatabaseConnection,
    organization_id: &str,
) -> Result<u64, String> {
    user::Entity::find()
        .filter(user::Column::OrganizationId.eq(organization_id))
        .filter(user::Column::AccessRole.eq("admin"))
        .filter(user::Column::IsActive.eq(true))
        .count(database)
        .await
        .map_err(|error| format!("Failed to check active administrators: {error}"))
}

pub async fn create_user(
    database: &DatabaseConnection,
    organization_id: &str,
    actor_id: &str,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Result<UserSummary, String> {
    validate_user_input(name, email, password)?;
    if !matches!(role, "admin" | "staff" | "readonly") {
        return Err("Role must be admin, staff, or readonly.".to_string());
    }

    let actor = user::Entity::find_by_id(actor_id)
        .filter(user::Column::OrganizationId.eq(organization_id))
        .filter(user::Column::AccessRole.eq("admin"))
        .filter(user::Column::IsActive.eq(true))
        .one(database)
        .await
        .map_err(|error| format!("Failed to verify administrator: {error}"))?;
    if actor.is_none() {
        return Err("Administrator access is required.".to_string());
    }

    let transaction = database
        .begin()
        .await
        .map_err(|error| format!("Failed to begin user creation: {error}"))?;
    let created = user::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
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
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to create user: {error}"))?;
    audit_event::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        actor_user_id: Set(Some(actor_id.to_string())),
        action: Set("created".to_string()),
        entity_type: Set("user".to_string()),
        entity_id: Set(created.id.clone()),
        details_json: Set(Some(serde_json::json!({"role": role}).to_string())),
        created_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to write user creation audit event: {error}"))?;
    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to save user creation: {error}"))?;
    Ok(summary(created))
}

pub async fn set_user_active(
    database: &DatabaseConnection,
    organization_id: &str,
    actor_id: &str,
    user_id: &str,
    is_active: bool,
) -> Result<UserSummary, String> {
    let actor = user::Entity::find_by_id(actor_id)
        .filter(user::Column::OrganizationId.eq(organization_id))
        .filter(user::Column::AccessRole.eq("admin"))
        .filter(user::Column::IsActive.eq(true))
        .one(database)
        .await
        .map_err(|error| format!("Failed to verify administrator: {error}"))?;
    if actor.is_none() {
        return Err("Administrator access is required.".to_string());
    }
    if actor_id == user_id && !is_active {
        return Err("You cannot deactivate the account currently in use.".to_string());
    }
    let transaction = database
        .begin()
        .await
        .map_err(|error| format!("Failed to begin user status update: {error}"))?;
    let model = user::Entity::find_by_id(user_id)
        .filter(user::Column::OrganizationId.eq(organization_id))
        .one(&transaction)
        .await
        .map_err(|error| format!("Failed to find user: {error}"))?
        .ok_or_else(|| "User was not found.".to_string())?;
    if model.is_active == is_active {
        return Ok(summary(model));
    }
    if !is_active && model.access_role == "admin" {
        let admin_count = user::Entity::find()
            .filter(user::Column::OrganizationId.eq(organization_id))
            .filter(user::Column::AccessRole.eq("admin"))
            .filter(user::Column::IsActive.eq(true))
            .count(&transaction)
            .await
            .map_err(|error| format!("Failed to check active administrators: {error}"))?;
        if admin_count <= 1 {
            return Err("The last active administrator cannot be deactivated.".to_string());
        }
    }
    let mut active: user::ActiveModel = model.into();
    active.is_active = Set(is_active);
    active.updated_at = Set(Utc::now());
    let updated = active
        .update(&transaction)
        .await
        .map_err(|error| format!("Failed to update user: {error}"))?;
    audit_event::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        actor_user_id: Set(Some(actor_id.to_string())),
        action: Set(if is_active {
            "activated"
        } else {
            "deactivated"
        }
        .to_string()),
        entity_type: Set("user".to_string()),
        entity_id: Set(user_id.to_string()),
        details_json: Set(Some(
            serde_json::json!({"is_active": is_active}).to_string(),
        )),
        created_at: Set(Utc::now()),
    }
    .insert(&transaction)
    .await
    .map_err(|error| format!("Failed to write user status audit event: {error}"))?;
    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to save user status: {error}"))?;
    Ok(summary(updated))
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
    use crate::models::{organization, organization_currency, user};
    use sea_orm::{ActiveModelTrait, Database, EntityTrait, Set};
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
        let templates = crate::repositories::settings::SeaOrmSettingsRepository::new(&database);
        let templates = aqardesk_app::services::settings::SettingsService::new(templates)
            .list_templates(&admin.organization_id)
            .await
            .expect("load seeded contract template");
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].template_key, "residential-default-ar");
        assert_eq!(templates[0].versions.len(), 1);

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

    #[tokio::test]
    async fn user_administration_is_organization_scoped_and_audited() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        crate::migration::Migrator::up(&database, None)
            .await
            .expect("apply migrations");
        let admin = create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create initial administrator");
        let created = create_user(
            &database,
            &admin.organization_id,
            &admin.id,
            "Team member",
            "team@example.com",
            "another-secure-password",
            "staff",
        )
        .await
        .expect("create organization user");
        assert_eq!(
            list_users(&database, &admin.organization_id)
                .await
                .unwrap()
                .len(),
            2
        );

        let audit_events = crate::models::audit_event::Entity::find()
            .filter(crate::models::audit_event::Column::OrganizationId.eq(&admin.organization_id))
            .all(&database)
            .await
            .expect("query user audit event");
        assert_eq!(audit_events.len(), 1);
        assert_eq!(
            audit_events[0].actor_user_id.as_deref(),
            Some(admin.id.as_str())
        );
        assert_eq!(audit_events[0].entity_id, created.id);

        let other_organization_id = Uuid::new_v4().to_string();
        organization::ActiveModel {
            id: Set(other_organization_id.clone()),
            name: Set("Other organization".to_string()),
            legal_name: Set(None),
            default_currency: Set("SAR".to_string()),
            timezone: Set("Asia/Riyadh".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&database)
        .await
        .expect("create second organization");
        let other_user_id = Uuid::new_v4().to_string();
        user::ActiveModel {
            id: Set(other_user_id.clone()),
            organization_id: Set(other_organization_id),
            full_name: Set("Other admin".to_string()),
            email: Set("other-admin@example.com".to_string()),
            password_hash: Set(hash_password("other-secure-password".to_string()).unwrap()),
            access_role: Set("admin".to_string()),
            is_active: Set(true),
            failed_login_attempts: Set(0),
            locked_until: Set(None),
            last_login_at: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&database)
        .await
        .expect("create user in second organization");

        let organization_users = list_users(&database, &admin.organization_id)
            .await
            .expect("list scoped users");
        assert_eq!(organization_users.len(), 2);
        assert!(
            organization_users
                .iter()
                .all(|candidate| candidate.id != other_user_id)
        );
        assert!(
            set_user_active(
                &database,
                &admin.organization_id,
                &admin.id,
                &other_user_id,
                false
            )
            .await
            .is_err()
        );
        let other_user = user::Entity::find_by_id(other_user_id)
            .one(&database)
            .await
            .expect("query second organization user")
            .expect("second organization user exists");
        assert!(other_user.is_active);
    }
}
