use crate::{
    application::{ApplicationError, Page, PageRequest},
    domain::{OrganizationId, PropertyUnit, UnitId, UserId},
};

#[async_trait::async_trait]
pub trait UnitRepository: Send + Sync {
    async fn list(
        &self,
        organization_id: OrganizationId,
        property_id: Option<crate::domain::PropertyId>,
    ) -> Result<Vec<PropertyUnit>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        property_id: Option<crate::domain::PropertyId>,
        page: PageRequest,
    ) -> Result<Page<PropertyUnit>, ApplicationError> {
        Ok(Page::from_items(
            self.list(organization_id, property_id).await?,
            page.limit(),
        ))
    }

    async fn insert(
        &self,
        unit: PropertyUnit,
        actor_id: UserId,
    ) -> Result<PropertyUnit, ApplicationError>;

    async fn update(
        &self,
        unit: PropertyUnit,
        actor_id: UserId,
    ) -> Result<PropertyUnit, ApplicationError>;

    async fn archive(
        &self,
        organization_id: OrganizationId,
        unit_id: UnitId,
        actor_id: UserId,
    ) -> Result<(), ApplicationError>;
}
