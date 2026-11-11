use crate::models::tenant;
use aqardesk_app::{
    AppError,
    services::tenants::{TenantInput, TenantRepository},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub struct SeaOrmTenantRepository<'db> {
    database: &'db DatabaseConnection,
}

impl<'db> SeaOrmTenantRepository<'db> {
    pub fn new(database: &'db DatabaseConnection) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl TenantRepository for SeaOrmTenantRepository<'_> {
    type Item = tenant::Model;

    async fn list(&self, organization_id: &str) -> Result<Vec<Self::Item>, AppError> {
        tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .order_by_asc(tenant::Column::FullName)
            .all(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to list tenants: {error}")))
    }

    async fn create(
        &self,
        organization_id: &str,
        input: TenantInput,
    ) -> Result<Self::Item, AppError> {
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
        .insert(self.database)
        .await
        .map_err(|error| AppError::Persistence(format!("Failed to create tenant: {error}")))
    }

    async fn update(
        &self,
        organization_id: &str,
        tenant_id: &str,
        input: TenantInput,
    ) -> Result<Self::Item, AppError> {
        let model = tenant::Entity::find_by_id(tenant_id)
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find tenant: {error}")))?
            .ok_or_else(|| AppError::NotFound("Tenant was not found.".to_string()))?;
        let mut active: tenant::ActiveModel = model.into();
        active.full_name = Set(input.full_name);
        active.email = Set(input.email);
        active.phone = Set(input.phone);
        active.mailing_address = Set(input.mailing_address);
        active.notes = Set(input.notes);
        active.updated_at = Set(Utc::now());
        active
            .update(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to update tenant: {error}")))
    }

    async fn archive(&self, organization_id: &str, tenant_id: &str) -> Result<(), AppError> {
        let model = tenant::Entity::find_by_id(tenant_id)
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find tenant: {error}")))?
            .ok_or_else(|| AppError::NotFound("Tenant was not found.".to_string()))?;
        let mut active: tenant::ActiveModel = model.into();
        active.deleted_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());
        active
            .update(self.database)
            .await
            .map(|_| ())
            .map_err(|error| AppError::Persistence(format!("Failed to archive tenant: {error}")))
    }
}
