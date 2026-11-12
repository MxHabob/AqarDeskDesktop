use crate::{
    application::{ApplicationError, Page, PageRequest, TenantRepository},
    domain::{OrganizationId, Tenant, TenantId, TenantInput, UserId},
    infrastructure::{audit, models::tenant},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use std::collections::HashMap;

pub struct SeaOrmTenantRepository {
    database: DatabaseConnection,
}

impl SeaOrmTenantRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl TenantRepository for SeaOrmTenantRepository {
    async fn list(&self, organization_id: OrganizationId) -> Result<Vec<Tenant>, ApplicationError> {
        let models = tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(tenant::Column::DeletedAt.is_null())
            .order_by_asc(tenant::Column::FullName)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        models.into_iter().map(restore_tenant).collect()
    }

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Tenant>, ApplicationError> {
        let models = tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(tenant::Column::DeletedAt.is_null())
            .order_by_asc(tenant::Column::FullName)
            .order_by_asc(tenant::Column::Id)
            .limit(page.fetch_limit())
            .offset(page.offset())
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let tenants = models
            .into_iter()
            .map(restore_tenant)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::from_items(tenants, page.limit()))
    }

    async fn names(
        &self,
        organization_id: OrganizationId,
        tenant_ids: &[TenantId],
    ) -> Result<HashMap<TenantId, String>, ApplicationError> {
        if tenant_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = tenant_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let models = tenant::Entity::find()
            .filter(tenant::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(tenant::Column::Id.is_in(ids))
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        models
            .into_iter()
            .map(|model| {
                let id = TenantId::parse(&model.id).map_err(|error| {
                    tracing::error!(%error, "Persisted tenant identifier is invalid");
                    ApplicationError::Persistence
                })?;
                Ok((id, model.full_name))
            })
            .collect()
    }

    async fn insert(&self, tenant: Tenant, actor_id: UserId) -> Result<Tenant, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let model = tenant::ActiveModel {
            id: Set(tenant.id.to_string()),
            organization_id: Set(tenant.organization_id.to_string()),
            full_name: Set(tenant.input.full_name.clone()),
            email: Set(tenant.input.email.clone()),
            phone: Set(tenant.input.phone.clone()),
            mailing_address: Set(tenant.input.mailing_address.clone()),
            notes: Set(tenant.input.notes.clone()),
            created_at: Set(tenant.created_at),
            updated_at: Set(tenant.updated_at),
            deleted_at: Set(None),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &tenant.organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "tenant",
            &tenant.id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_tenant(model)
    }

    async fn update(&self, tenant: Tenant, actor_id: UserId) -> Result<Tenant, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = tenant::Entity::find_by_id(tenant.id.to_string())
            .filter(tenant::Column::OrganizationId.eq(tenant.organization_id.to_string()))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut active = existing.into_active_model();
        active.full_name = Set(tenant.input.full_name.clone());
        active.email = Set(tenant.input.email.clone());
        active.phone = Set(tenant.input.phone.clone());
        active.mailing_address = Set(tenant.input.mailing_address.clone());
        active.notes = Set(tenant.input.notes.clone());
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &tenant.organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "tenant",
            &tenant.id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_tenant(model)
    }

    async fn archive(
        &self,
        organization_id: OrganizationId,
        tenant_id: TenantId,
        actor_id: UserId,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = tenant::Entity::find_by_id(tenant_id.to_string())
            .filter(tenant::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut active = existing.into_active_model();
        let now = Utc::now();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "archived",
            "tenant",
            &tenant_id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }
}

fn restore_tenant(model: tenant::Model) -> Result<Tenant, ApplicationError> {
    let id = TenantId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted tenant identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&model.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let input = TenantInput {
        full_name: model.full_name,
        email: model.email,
        phone: model.phone,
        mailing_address: model.mailing_address,
        notes: model.notes,
    };
    Tenant::restore(
        id,
        organization_id,
        input,
        model.created_at,
        model.updated_at,
    )
    .map_err(Into::into)
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Tenant persistence operation failed");
    ApplicationError::Persistence
}
