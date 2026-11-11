use crate::models::{property, unit};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitInput {
    pub property_id: String,
    pub unit_number: String,
    pub unit_type: String,
    pub floor_number: Option<i32>,
    pub area_square_meters: Option<sea_orm::prelude::Decimal>,
    pub bedrooms: i32,
    pub bathrooms: sea_orm::prelude::Decimal,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<String>,
    pub status: String,
}

pub async fn list_units(
    database: &DatabaseConnection,
    organization_id: &str,
    property_id: Option<&str>,
) -> Result<Vec<unit::Model>, String> {
    let mut query = unit::Entity::find()
        .filter(unit::Column::OrganizationId.eq(organization_id))
        .filter(unit::Column::DeletedAt.is_null());
    if let Some(property_id) = property_id {
        query = query.filter(unit::Column::PropertyId.eq(property_id));
    }
    query
        .order_by_asc(unit::Column::PropertyId)
        .order_by_asc(unit::Column::UnitNumber)
        .all(database)
        .await
        .map_err(|error| format!("Failed to list units: {error}"))
}

pub async fn create_unit(
    database: &DatabaseConnection,
    organization_id: &str,
    input: UnitInput,
) -> Result<unit::Model, String> {
    let input = validate_unit(input)?;
    ensure_property(database, organization_id, &input.property_id).await?;
    let now = Utc::now();
    unit::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        property_id: Set(input.property_id),
        unit_number: Set(input.unit_number),
        unit_type: Set(input.unit_type),
        floor_number: Set(input.floor_number),
        area_square_meters: Set(input.area_square_meters),
        bedrooms: Set(input.bedrooms),
        bathrooms: Set(input.bathrooms),
        market_rent_minor: Set(input.market_rent_minor),
        currency_code: Set(input.currency_code),
        status: Set(input.status),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(database)
    .await
    .map_err(|error| format!("Failed to create unit: {error}"))
}

pub async fn update_unit(
    database: &DatabaseConnection,
    organization_id: &str,
    unit_id: &str,
    input: UnitInput,
) -> Result<unit::Model, String> {
    let input = validate_unit(input)?;
    ensure_property(database, organization_id, &input.property_id).await?;
    let model = unit::Entity::find_by_id(unit_id)
        .filter(unit::Column::OrganizationId.eq(organization_id))
        .filter(unit::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find unit: {error}"))?
        .ok_or_else(|| "Unit was not found.".to_string())?;
    let mut active: unit::ActiveModel = model.into();
    active.property_id = Set(input.property_id);
    active.unit_number = Set(input.unit_number);
    active.unit_type = Set(input.unit_type);
    active.floor_number = Set(input.floor_number);
    active.area_square_meters = Set(input.area_square_meters);
    active.bedrooms = Set(input.bedrooms);
    active.bathrooms = Set(input.bathrooms);
    active.market_rent_minor = Set(input.market_rent_minor);
    active.currency_code = Set(input.currency_code);
    active.status = Set(input.status);
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map_err(|error| format!("Failed to update unit: {error}"))
}

pub async fn archive_unit(
    database: &DatabaseConnection,
    organization_id: &str,
    unit_id: &str,
) -> Result<(), String> {
    let model = unit::Entity::find_by_id(unit_id)
        .filter(unit::Column::OrganizationId.eq(organization_id))
        .filter(unit::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to find unit: {error}"))?
        .ok_or_else(|| "Unit was not found.".to_string())?;
    let mut active: unit::ActiveModel = model.into();
    active.deleted_at = Set(Some(Utc::now()));
    active.status = Set("inactive".to_string());
    active.updated_at = Set(Utc::now());
    active
        .update(database)
        .await
        .map(|_| ())
        .map_err(|error| format!("Failed to archive unit: {error}"))
}

async fn ensure_property(
    database: &DatabaseConnection,
    organization_id: &str,
    property_id: &str,
) -> Result<(), String> {
    property::Entity::find_by_id(property_id)
        .filter(property::Column::OrganizationId.eq(organization_id))
        .filter(property::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| format!("Failed to validate the property: {error}"))?
        .map(|_| ())
        .ok_or_else(|| "Property was not found in this organization.".to_string())
}

fn validate_unit(mut input: UnitInput) -> Result<UnitInput, String> {
    input.property_id = input.property_id.trim().to_string();
    input.unit_number = input.unit_number.trim().to_string();
    input.currency_code = input.currency_code.map(|value| value.trim().to_uppercase());
    if input.property_id.is_empty() || input.unit_number.is_empty() {
        return Err("Property and unit number are required.".to_string());
    }
    if !matches!(
        input.unit_type.as_str(),
        "apartment" | "office" | "retail" | "warehouse" | "parking" | "other"
    ) {
        return Err("Unsupported unit type.".to_string());
    }
    if !matches!(
        input.status.as_str(),
        "available" | "maintenance" | "inactive"
    ) {
        return Err("Unsupported unit status.".to_string());
    }
    if input.bedrooms < 0
        || input.bathrooms < sea_orm::prelude::Decimal::ZERO
        || input
            .area_square_meters
            .is_some_and(|area| area <= sea_orm::prelude::Decimal::ZERO)
        || input.market_rent_minor.is_some_and(|rent| rent < 0)
    {
        return Err("Unit measurements and rent must be zero or greater.".to_string());
    }
    match (input.market_rent_minor, input.currency_code.as_ref()) {
        (None, None) => {}
        (Some(_), Some(code)) if code.len() == 3 && code.is_ascii() => {}
        _ => return Err("A three-letter currency is required with the rent amount.".to_string()),
    }
    Ok(input)
}
