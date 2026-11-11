use super::clean_optional;
use crate::models::tenant;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
}

pub async fn list_tenants(
    database: &DatabaseConnection,
    organization_id: &str,
) -> Result<Vec<tenant::Model>, String> {
    tenant::Entity::find()
        .filter(tenant::Column::OrganizationId.eq(organization_id))
        .filter(tenant::Column::DeletedAt.is_null())
        .order_by_asc(tenant::Column::FullName)
        .all(database)
        .await
        .map_err(|error| format!("Failed to list tenants: {error}"))
}

pub async fn create_tenant(
    database: &DatabaseConnection,
    organization_id: &str,
    input: TenantInput,
) -> Result<tenant::Model, String> {
    let input = validate_tenant(input)?;
    let now = Utc::now();
    tenant::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        full_name: Set(input.full_name),
        email: Set(input.email),
        phone: Set(input.phone),
        mailing_address: Set(input.mailing_address),
        notes: Set(input.notes),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(database)
    .await
    .map_err(|error| format!("Failed to create tenant: {error}"))
}

pub async fn update_tenant(
    database: &DatabaseConnection,
    organization_id: &str,
    tenant_id: &str,
    input: TenantInput,
) -> Result<tenant::Model, String> {
    let input = validate_tenant(input)?;
    let model = tenant::Entity::find_by_id(tenant_id)
        .filter(tenant::Column::OrganizationId.eq(organization_id))
        .filter(tenant::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find tenant: {error}"))?
        .ok_or_else(|| "Tenant was not found.".to_string())?;
    let mut active: tenant::ActiveModel = model.into();
    active.full_name = Set(input.full_name);
    active.email = Set(input.email);
    active.phone = Set(input.phone);
    active.mailing_address = Set(input.mailing_address);
    active.notes = Set(input.notes);
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map_err(|error| format!("Failed to update tenant: {error}"))
}

pub async fn archive_tenant(
    database: &DatabaseConnection,
    organization_id: &str,
    tenant_id: &str,
) -> Result<(), String> {
    let model = tenant::Entity::find_by_id(tenant_id)
        .filter(tenant::Column::OrganizationId.eq(organization_id))
        .filter(tenant::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find tenant: {error}"))?
        .ok_or_else(|| "Tenant was not found.".to_string())?;
    let mut active: tenant::ActiveModel = model.into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map(|_| ())
        .map_err(|error| format!("Failed to archive tenant: {error}"))
}

fn validate_tenant(mut input: TenantInput) -> Result<TenantInput, String> {
    input.full_name = input.full_name.trim().to_string();
    input.email = clean_optional(input.email).map(|value| value.to_lowercase());
    input.phone = clean_optional(input.phone);
    input.mailing_address = clean_optional(input.mailing_address);
    input.notes = clean_optional(input.notes);
    if input.full_name.is_empty() {
        return Err("Tenant name is required.".to_string());
    }
    if input.email.as_ref().is_some_and(|email| {
        email.len() > 254
            || !email.contains('@')
            || email.bytes().any(|byte| byte.is_ascii_whitespace())
    }) {
        return Err("A valid tenant email address is required.".to_string());
    }
    if input.phone.is_none() && input.email.is_none() {
        return Err("A tenant email address or phone number is required.".to_string());
    }
    Ok(input)
}
