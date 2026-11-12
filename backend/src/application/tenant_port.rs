use crate::{
    application::{ApplicationError, Page, PageRequest},
    domain::{OrganizationId, Tenant, TenantId, UserId},
};
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait TenantRepository: Send + Sync {
    async fn list(&self, organization_id: OrganizationId) -> Result<Vec<Tenant>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Tenant>, ApplicationError> {
        Ok(Page::from_items(self.list(organization_id).await?, page.limit()))
    }

    async fn names(
        &self,
        organization_id: OrganizationId,
        tenant_ids: &[TenantId],
    ) -> Result<HashMap<TenantId, String>, ApplicationError>;

    async fn insert(&self, tenant: Tenant, actor_id: UserId) -> Result<Tenant, ApplicationError>;

    async fn update(&self, tenant: Tenant, actor_id: UserId) -> Result<Tenant, ApplicationError>;

    async fn archive(
        &self,
        organization_id: OrganizationId,
        tenant_id: TenantId,
        actor_id: UserId,
    ) -> Result<(), ApplicationError>;
}
