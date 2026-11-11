use crate::{
    application::{ApplicationError, require_admin},
    domain::ActorContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySettings {
    pub code: String,
    pub name: String,
    pub name_ar: Option<String>,
    pub symbol: String,
    pub symbol_position: String,
    pub decimal_digits: i16,
    pub is_active: bool,
    pub is_enabled_for_organization: bool,
}

#[async_trait::async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn list(&self, organization_id: &str) -> Result<Vec<CurrencySettings>, ApplicationError>;

    async fn set_enabled(
        &self,
        organization_id: &str,
        actor_id: &str,
        code: &str,
        enabled: bool,
    ) -> Result<(), ApplicationError>;
}

pub struct CurrencyCatalog<R> {
    repository: R,
}

impl<R: CurrencyRepository> CurrencyCatalog<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: ActorContext,
    ) -> Result<Vec<CurrencySettings>, ApplicationError> {
        self.repository
            .list(&actor.organization_id().to_string())
            .await
    }

    pub async fn set_enabled(
        &self,
        actor: ActorContext,
        code: &str,
        enabled: bool,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        let code = code.trim().to_ascii_uppercase();
        if code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(crate::domain::DomainError::Validation(
                "Currency must be a three-letter code.",
            )
            .into());
        }
        self.repository
            .set_enabled(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                &code,
                enabled,
            )
            .await
    }
}
