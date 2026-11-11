use crate::{
    application::{ApplicationError, require_admin},
    domain::ActorContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub actor_user_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details_json: Option<String>,
    pub created_at: String,
}

#[async_trait::async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn list(
        &self,
        organization_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, ApplicationError>;
}

pub struct AuditLog<R> {
    repository: R,
}

impl<R: AuditLogRepository> AuditLog<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: ActorContext,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, ApplicationError> {
        require_admin(actor)?;
        if !(1..=100).contains(&limit) || offset > 1_000_000 {
            return Err(crate::domain::DomainError::Validation(
                "Audit page size must be from 1 to 100 and offset cannot exceed 1000000.",
            )
            .into());
        }
        self.repository
            .list(&actor.organization_id().to_string(), limit, offset)
            .await
    }
}
