use super::clean_optional;
use crate::models::property;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInput {
    pub code: String,
    pub name: String,
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub status: String,
}

pub async fn list_properties(
    database: &DatabaseConnection,
    organization_id: &str,
) -> Result<Vec<property::Model>, String> {
    property::Entity::find()
        .filter(property::Column::OrganizationId.eq(organization_id))
        .filter(property::Column::DeletedAt.is_null())
        .order_by_asc(property::Column::Name)
        .all(database)
        .await
        .map_err(|error| format!("Failed to list properties: {error}"))
}

pub async fn create_property(
    database: &DatabaseConnection,
    organization_id: &str,
    actor_id: &str,
    input: PropertyInput,
) -> Result<property::Model, String> {
    let input = validate_property(input)?;
    let now = Utc::now();
    property::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        code: Set(input.code),
        name: Set(input.name),
        property_type: Set(input.property_type),
        description: Set(input.description),
        address_line_1: Set(input.address_line_1),
        address_line_2: Set(input.address_line_2),
        city: Set(input.city),
        region: Set(input.region),
        postal_code: Set(input.postal_code),
        country_code: Set(input.country_code),
        status: Set(input.status),
        created_by: Set(Some(actor_id.to_string())),
        updated_by: Set(Some(actor_id.to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(database)
    .await
    .map_err(|error| format!("Failed to create property: {error}"))
}

pub async fn update_property(
    database: &DatabaseConnection,
    organization_id: &str,
    actor_id: &str,
    property_id: &str,
    input: PropertyInput,
) -> Result<property::Model, String> {
    let input = validate_property(input)?;
    let model = property::Entity::find_by_id(property_id)
        .filter(property::Column::OrganizationId.eq(organization_id))
        .filter(property::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find property: {error}"))?
        .ok_or_else(|| "Property was not found.".to_string())?;
    let mut active: property::ActiveModel = model.into();
    active.code = Set(input.code);
    active.name = Set(input.name);
    active.property_type = Set(input.property_type);
    active.description = Set(input.description);
    active.address_line_1 = Set(input.address_line_1);
    active.address_line_2 = Set(input.address_line_2);
    active.city = Set(input.city);
    active.region = Set(input.region);
    active.postal_code = Set(input.postal_code);
    active.country_code = Set(input.country_code);
    active.status = Set(input.status);
    active.updated_by = Set(Some(actor_id.to_string()));
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map_err(|error| format!("Failed to update property: {error}"))
}

pub async fn archive_property(
    database: &DatabaseConnection,
    organization_id: &str,
    property_id: &str,
) -> Result<(), String> {
    let model = property::Entity::find_by_id(property_id)
        .filter(property::Column::OrganizationId.eq(organization_id))
        .filter(property::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find property: {error}"))?
        .ok_or_else(|| "Property was not found.".to_string())?;
    let mut active: property::ActiveModel = model.into();
    active.deleted_at = Set(Some(Utc::now()));
    active.status = Set("archived".to_string());
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map(|_| ())
        .map_err(|error| format!("Failed to archive property: {error}"))
}

fn validate_property(mut input: PropertyInput) -> Result<PropertyInput, String> {
    input.code = input.code.trim().to_uppercase();
    input.name = input.name.trim().to_string();
    input.property_type = input.property_type.trim().to_string();
    input.address_line_1 = input.address_line_1.trim().to_string();
    input.city = input.city.trim().to_string();
    input.country_code = input.country_code.trim().to_uppercase();
    input.description = clean_optional(input.description);
    input.address_line_2 = clean_optional(input.address_line_2);
    input.region = clean_optional(input.region);
    input.postal_code = clean_optional(input.postal_code);
    if input.code.is_empty()
        || input.name.is_empty()
        || input.address_line_1.is_empty()
        || input.city.is_empty()
    {
        return Err("Property code, name, address, and city are required.".to_string());
    }
    if !matches!(
        input.property_type.as_str(),
        "residential" | "commercial" | "mixed_use" | "land" | "other"
    ) {
        return Err("Unsupported property type.".to_string());
    }
    if !matches!(input.status.as_str(), "active" | "inactive" | "archived") {
        return Err("Unsupported property status.".to_string());
    }
    if input.country_code.len() != 2 || !input.country_code.is_ascii() {
        return Err("Country code must contain two ASCII letters.".to_string());
    }
    Ok(input)
}
