use crate::{
    application::ApplicationError,
    domain::{OrganizationId, Owner, UserId},
};

#[async_trait::async_trait]
pub trait OwnerRepository: Send + Sync {
    async fn list(&self, organization_id: OrganizationId) -> Result<Vec<Owner>, ApplicationError>;

    async fn insert(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError>;

    async fn update(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError>;
}
