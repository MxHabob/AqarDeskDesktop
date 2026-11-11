use crate::models::{property, unit};
use aqardesk_app::{
    AppError,
    services::units::{UnitInput, UnitRepository},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub struct SeaOrmUnitRepository<'db> {
    database: &'db DatabaseConnection,
}

impl<'db> SeaOrmUnitRepository<'db> {
    pub fn new(database: &'db DatabaseConnection) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl UnitRepository for SeaOrmUnitRepository<'_> {
    type Item = unit::Model;

    async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<Self::Item>, AppError> {
        let mut query = unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null());
        if let Some(property_id) = property_id {
            query = query.filter(unit::Column::PropertyId.eq(property_id));
        }
        query
            .order_by_asc(unit::Column::PropertyId)
            .order_by_asc(unit::Column::UnitNumber)
            .all(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to list units: {error}")))
    }

    async fn create(
        &self,
        organization_id: &str,
        input: UnitInput,
    ) -> Result<Self::Item, AppError> {
        ensure_property(self.database, organization_id, &input.property_id).await?;
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
        .insert(self.database)
        .await
        .map_err(|error| AppError::Persistence(format!("Failed to create unit: {error}")))
    }

    async fn update(
        &self,
        organization_id: &str,
        unit_id: &str,
        input: UnitInput,
    ) -> Result<Self::Item, AppError> {
        ensure_property(self.database, organization_id, &input.property_id).await?;
        let model = unit::Entity::find_by_id(unit_id)
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find unit: {error}")))?
            .ok_or_else(|| AppError::NotFound("Unit was not found.".to_string()))?;
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
            .update(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to update unit: {error}")))
    }

    async fn archive(&self, organization_id: &str, unit_id: &str) -> Result<(), AppError> {
        let model = unit::Entity::find_by_id(unit_id)
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find unit: {error}")))?
            .ok_or_else(|| AppError::NotFound("Unit was not found.".to_string()))?;
        let mut active: unit::ActiveModel = model.into();
        let now = Utc::now();
        active.deleted_at = Set(Some(now));
        active.status = Set("inactive".to_string());
        active.updated_at = Set(now);
        active
            .update(self.database)
            .await
            .map(|_| ())
            .map_err(|error| AppError::Persistence(format!("Failed to archive unit: {error}")))
    }
}

async fn ensure_property(
    database: &DatabaseConnection,
    organization_id: &str,
    property_id: &str,
) -> Result<(), AppError> {
    property::Entity::find_by_id(property_id)
        .filter(property::Column::OrganizationId.eq(organization_id))
        .filter(property::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(|error| {
            AppError::Persistence(format!("Failed to validate the property: {error}"))
        })?
        .map(|_| ())
        .ok_or_else(|| {
            AppError::NotFound("Property was not found in this organization.".to_string())
        })
}
