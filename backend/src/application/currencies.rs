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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyInput {
    pub name: String,
    pub name_ar: Option<String>,
    pub symbol: String,
    pub symbol_position: String,
    pub decimal_digits: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCurrencyInput {
    pub code: String,
    pub name: String,
    pub name_ar: Option<String>,
    pub symbol: String,
    pub symbol_position: String,
    pub decimal_digits: i16,
    pub enable_for_organization: bool,
}

#[async_trait::async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn list(&self, organization_id: &str) -> Result<Vec<CurrencySettings>, ApplicationError>;

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: NewCurrencyInput,
    ) -> Result<(), ApplicationError>;

    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        code: &str,
        input: CurrencyInput,
    ) -> Result<(), ApplicationError>;

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

    pub async fn create(
        &self,
        actor: ActorContext,
        input: NewCurrencyInput,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        let input = validate_new(input)?;
        self.repository
            .create(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                input,
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        code: &str,
        input: CurrencyInput,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        let code = validate_code(code)?;
        self.repository
            .update(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                &code,
                validate(input)?,
            )
            .await
    }

    pub async fn set_enabled(
        &self,
        actor: ActorContext,
        code: &str,
        enabled: bool,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        let code = validate_code(code)?;
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

fn validate_new(mut input: NewCurrencyInput) -> Result<NewCurrencyInput, ApplicationError> {
    input.code = validate_code(&input.code)?;
    let details = validate(CurrencyInput {
        name: input.name,
        name_ar: input.name_ar,
        symbol: input.symbol,
        symbol_position: input.symbol_position,
        decimal_digits: input.decimal_digits,
    })?;
    input.name = details.name;
    input.name_ar = details.name_ar;
    input.symbol = details.symbol;
    input.symbol_position = details.symbol_position;
    input.decimal_digits = details.decimal_digits;
    Ok(input)
}

fn validate(mut input: CurrencyInput) -> Result<CurrencyInput, ApplicationError> {
    input.name = input.name.trim().to_string();
    input.name_ar = clean_optional(input.name_ar);
    input.symbol = input.symbol.trim().to_string();
    input.symbol_position = input.symbol_position.trim().to_ascii_lowercase();

    if input.name.is_empty() || input.name.len() > 120 {
        return Err(validation("Currency name must be 1 to 120 characters."));
    }
    if input.name_ar.as_ref().is_some_and(|name| name.len() > 120) {
        return Err(validation(
            "Arabic currency name must not exceed 120 characters.",
        ));
    }
    if input.symbol.is_empty() || input.symbol.len() > 12 {
        return Err(validation("Currency symbol must be 1 to 12 characters."));
    }
    if !matches!(input.symbol_position.as_str(), "prefix" | "suffix") {
        return Err(validation(
            "Currency symbol position must be prefix or suffix.",
        ));
    }
    if !(0..=4).contains(&input.decimal_digits) {
        return Err(validation(
            "Currency decimal digits must be between 0 and 4.",
        ));
    }
    Ok(input)
}

fn validate_code(value: &str) -> Result<String, ApplicationError> {
    let code = value.trim().to_ascii_uppercase();
    if code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(validation("Currency must be a three-letter code."));
    }
    Ok(code)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}
