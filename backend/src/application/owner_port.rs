use crate::{
    application::{ApplicationError, Page, PageRequest},
    domain::{OrganizationId, Owner, UserId},
};

#[async_trait::async_trait]
pub trait OwnerRepository: Send + Sync {
    async fn list(&self, organization_id: OrganizationId) -> Result<Vec<Owner>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Owner>, ApplicationError> {
        Ok(Page::from_items(self.list(organization_id).await?, page.limit()))
    }

    async fn insert(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError>;

    async fn update(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError>;
}
