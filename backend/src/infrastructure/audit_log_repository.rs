use crate::{
    application::{ApplicationError, AuditEntry, AuditLogRepository},
    infrastructure::models::user,
};
use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub organization_id: String,
    pub actor_user_id: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details_json: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct SeaOrmAuditLogRepository {
    database: DatabaseConnection,
}

impl SeaOrmAuditLogRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl AuditLogRepository for SeaOrmAuditLogRepository {
    async fn list(
        &self,
        organization_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, ApplicationError> {
        let events = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id)
            .limit(limit)
            .offset(offset)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let actor_ids = events
            .iter()
            .filter_map(|event| event.actor_user_id.clone())
            .collect::<std::collections::HashSet<_>>();
        let users = if actor_ids.is_empty() {
            Vec::new()
        } else {
            user::Entity::find()
                .filter(user::Column::OrganizationId.eq(organization_id))
                .filter(user::Column::Id.is_in(actor_ids))
                .all(&self.database)
                .await
                .map_err(persistence_error)?
        };
        let names = users
            .into_iter()
            .map(|account| (account.id, account.full_name))
            .collect::<std::collections::HashMap<_, _>>();

        Ok(events
            .into_iter()
            .map(|event| AuditEntry {
                actor_name: event
                    .actor_user_id
                    .as_ref()
                    .and_then(|id| names.get(id).cloned()),
                id: event.id,
                actor_user_id: event.actor_user_id,
                action: event.action,
                entity_type: event.entity_type,
                entity_id: event.entity_id,
                details_json: event.details_json,
                created_at: event.created_at.to_rfc3339(),
            })
            .collect())
    }
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Audit log persistence operation failed");
    ApplicationError::Persistence
}
