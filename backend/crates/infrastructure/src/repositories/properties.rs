use crate::models::property;
use aqardesk_app::{
    AppError,
    services::properties::{PropertyInput, PropertyRepository},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub struct SeaOrmPropertyRepository<'db> {
    database: &'db DatabaseConnection,
}

impl<'db> SeaOrmPropertyRepository<'db> {
    pub fn new(database: &'db DatabaseConnection) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl PropertyRepository for SeaOrmPropertyRepository<'_> {
    type Item = property::Model;

    async fn list(&self, organization_id: &str) -> Result<Vec<Self::Item>, AppError> {
        property::Entity::find()
            .filter(property::Column::OrganizationId.eq(organization_id))
            .filter(property::Column::DeletedAt.is_null())
            .order_by_asc(property::Column::Name)
            .all(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to list properties: {error}")))
    }

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PropertyInput,
    ) -> Result<Self::Item, AppError> {
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
        .insert(self.database)
        .await
        .map_err(|error| AppError::Persistence(format!("Failed to create property: {error}")))
    }

    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
        input: PropertyInput,
    ) -> Result<Self::Item, AppError> {
        let model = property::Entity::find_by_id(property_id)
            .filter(property::Column::OrganizationId.eq(organization_id))
            .filter(property::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find property: {error}")))?
            .ok_or_else(|| AppError::NotFound("Property was not found.".to_string()))?;
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
            .update(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to update property: {error}")))
    }

    async fn archive(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
    ) -> Result<(), AppError> {
        let model = property::Entity::find_by_id(property_id)
            .filter(property::Column::OrganizationId.eq(organization_id))
            .filter(property::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find property: {error}")))?
            .ok_or_else(|| AppError::NotFound("Property was not found.".to_string()))?;
        let mut active: property::ActiveModel = model.into();
        let now = Utc::now();
        active.deleted_at = Set(Some(now));
        active.status = Set("archived".to_string());
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(now);
        active
            .update(self.database)
            .await
            .map(|_| ())
            .map_err(|error| AppError::Persistence(format!("Failed to archive property: {error}")))
    }
}
