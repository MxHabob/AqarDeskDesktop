use crate::{
    application::{ApplicationError, require_admin},
    domain::ActorContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplateVersion {
    pub id: String,
    pub version_number: i32,
    pub content_format: String,
    pub content_body: Option<String>,
    pub placeholders_json: String,
    pub content_sha256: String,
    pub change_note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplate {
    pub id: String,
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub lease_type: String,
    pub locale: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub versions: Vec<ContractTemplateVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplateInput {
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub lease_type: String,
    pub locale: String,
    pub content_format: String,
    pub content_body: String,
    pub placeholders_json: Option<String>,
    pub change_note: Option<String>,
}

#[async_trait::async_trait]
pub trait ContractTemplateRepository: Send + Sync {
    async fn list(&self, organization_id: &str) -> Result<Vec<ContractTemplate>, ApplicationError>;
    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError>;
    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError>;
    async fn set_active(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), ApplicationError>;
}

pub struct ContractTemplateService<R> {
    repository: R,
}

impl<R: ContractTemplateRepository> ContractTemplateService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: ActorContext,
    ) -> Result<Vec<ContractTemplate>, ApplicationError> {
        require_admin(actor)?;
        self.repository
            .list(&actor.organization_id().to_string())
            .await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError> {
        require_admin(actor)?;
        self.repository
            .create(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                validate(input)?,
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, ApplicationError> {
        require_admin(actor)?;
        self.repository
            .update(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                template_id,
                validate(input)?,
            )
            .await
    }

    pub async fn set_active(
        &self,
        actor: ActorContext,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        self.repository
            .set_active(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                template_id,
                is_active,
            )
            .await
    }
}

fn validate(mut input: ContractTemplateInput) -> Result<ContractTemplateInput, ApplicationError> {
    input.template_key = input.template_key.trim().to_ascii_lowercase();
    input.name = input.name.trim().to_string();
    input.description = clean_optional(input.description);
    input.lease_type = input.lease_type.trim().to_ascii_lowercase();
    input.locale = input.locale.trim().to_ascii_lowercase();
    input.content_body = input.content_body.trim().to_string();
    input.placeholders_json =
        clean_optional(input.placeholders_json).or_else(|| Some("{}".to_string()));
    input.change_note = clean_optional(input.change_note);
    if input.template_key.is_empty()
        || input.template_key.len() > 64
        || !input.template_key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(validation(
            "Template key must be 1 to 64 lowercase letters, digits, hyphens, or underscores.",
        ));
    }
    if input.name.is_empty() || input.name.len() > 160 {
        return Err(validation("Template name is required."));
    }
    if !matches!(
        input.lease_type.as_str(),
        "residential" | "commercial" | "short_term" | "other"
    ) {
        return Err(validation("Unsupported lease type."));
    }
    if input.locale.is_empty() || input.locale.len() > 35 {
        return Err(validation("Template locale must be 1 to 35 characters."));
    }
    if !matches!(input.content_format.as_str(), "text" | "markdown" | "html") {
        return Err(validation("Unsupported template content format."));
    }
    if !serde_json::from_str::<serde_json::Value>(
        input.placeholders_json.as_deref().unwrap_or("{}"),
    )
    .is_ok_and(|value| value.is_object())
    {
        return Err(validation("Template placeholders must be a JSON object."));
    }
    Ok(input)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}
