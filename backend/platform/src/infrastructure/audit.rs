use crate::application::ApplicationError;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseTransaction};
use uuid::Uuid;

pub(crate) async fn record(
    transaction: &DatabaseTransaction,
    organization_id: &str,
    actor_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: serde_json::Value,
) -> Result<(), ApplicationError> {
    let event = audit_event::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        actor_user_id: Set(Some(actor_id.to_string())),
        action: Set(action.to_string()),
        entity_type: Set(entity_type.to_string()),
        entity_id: Set(entity_id.to_string()),
        details_json: Set(Some(details.to_string())),
        created_at: Set(Utc::now()),
    };
    event
        .insert(transaction)
        .await
        .map(|_| ())
        .map_err(|error| {
            tracing::error!(%error, "Audit event persistence failed");
            ApplicationError::Persistence
        })
}

mod audit_event {
    use sea_orm::entity::prelude::*;

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
}
