use crate::{
    application::{ApplicationError, Page, PageRequest},
    domain::{OrganizationId, Property, PropertyId, UserId},
};

#[async_trait::async_trait]
pub trait PropertyRepository: Send + Sync {
    async fn list(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Property>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Property>, ApplicationError> {
        Ok(Page::from_items(
            self.list(organization_id).await?,
            page.limit(),
        ))
    }

    async fn insert(
        &self,
        property: Property,
        actor_id: UserId,
    ) -> Result<Property, ApplicationError>;

    async fn update(
        &self,
        property: Property,
        actor_id: UserId,
    ) -> Result<Property, ApplicationError>;

    async fn archive(
        &self,
        organization_id: OrganizationId,
        property_id: PropertyId,
        actor_id: UserId,
    ) -> Result<(), ApplicationError>;
}
