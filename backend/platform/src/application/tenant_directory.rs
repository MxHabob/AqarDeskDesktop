use std::collections::HashMap;

use crate::{
    application::{ApplicationError, TenantRepository, require_write},
    domain::{ActorContext, Tenant, TenantId, TenantInput},
};

pub struct TenantDirectory<R> {
    tenants: R,
}

impl<R: TenantRepository> TenantDirectory<R> {
    pub fn new(tenants: R) -> Self {
        Self { tenants }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Tenant>, ApplicationError> {
        self.tenants.list(actor.organization_id()).await
    }

    pub async fn names(
        &self,
        actor: ActorContext,
        tenant_ids: &[TenantId],
    ) -> Result<HashMap<TenantId, String>, ApplicationError> {
        self.tenants
            .names(actor.organization_id(), tenant_ids)
            .await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: TenantInput,
    ) -> Result<Tenant, ApplicationError> {
        require_write(actor)?;
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
        require_write(actor)?;
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
        require_write(actor)?;
        self.tenants
            .archive(actor.organization_id(), tenant_id, actor.user_id())
            .await
    }
}
