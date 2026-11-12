use std::collections::HashMap;

use crate::{
    application::{ApplicationError, Page, PageRequest, TenantRepository, require_permission},
    domain::{ActorContext, Permission, Tenant, TenantId, TenantInput},
};

pub struct TenantDirectory<R> {
    tenants: R,
}

impl<R: TenantRepository> TenantDirectory<R> {
    pub fn new(tenants: R) -> Self {
        Self { tenants }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Tenant>, ApplicationError> {
        require_permission(actor, Permission::ViewTenants)?;
        self.tenants.list(actor.organization_id()).await
    }

    pub async fn list_page(
        &self,
        actor: ActorContext,
        page: PageRequest,
    ) -> Result<Page<Tenant>, ApplicationError> {
        require_permission(actor, Permission::ViewTenants)?;
        self.tenants.list_page(actor.organization_id(), page).await
    }

    pub async fn names(
        &self,
        actor: ActorContext,
        tenant_ids: &[TenantId],
    ) -> Result<HashMap<TenantId, String>, ApplicationError> {
        require_permission(actor, Permission::ViewTenants)?;
        self.tenants
            .names(actor.organization_id(), tenant_ids)
            .await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: TenantInput,
    ) -> Result<Tenant, ApplicationError> {
        require_permission(actor, Permission::CreateTenants)?;
        self.tenants
            .insert(
                Tenant::create(TenantId::new(), actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        tenant_id: TenantId,
        input: TenantInput,
    ) -> Result<Tenant, ApplicationError> {
        require_permission(actor, Permission::UpdateTenants)?;
        self.tenants
            .update(
                Tenant::create(tenant_id, actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }

    pub async fn archive(
        &self,
        actor: ActorContext,
        tenant_id: TenantId,
    ) -> Result<(), ApplicationError> {
        require_permission(actor, Permission::ArchiveTenants)?;
        self.tenants
            .archive(actor.organization_id(), tenant_id, actor.user_id())
            .await
    }
}
