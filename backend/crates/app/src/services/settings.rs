use crate::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettingsInput {
    pub name: String,
    pub legal_name: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

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
pub struct ProfileSettings {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettingsInput {
    pub name: String,
    pub email: String,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

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
pub trait SettingsRepository {
    async fn get_organization(
        &self,
        organization_id: &str,
    ) -> Result<OrganizationSettings, AppError>;
    async fn update_organization(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, AppError>;
    async fn list_currencies(
        &self,
        organization_id: &str,
    ) -> Result<Vec<CurrencySettings>, AppError>;
    async fn set_currency_enabled(
        &self,
        organization_id: &str,
        actor_id: &str,
        currency_code: &str,
        is_enabled: bool,
    ) -> Result<(), AppError>;
    async fn get_profile(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<ProfileSettings, AppError>;
    async fn update_profile(
        &self,
        organization_id: &str,
        user_id: &str,
        input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, AppError>;
    async fn list_audit(
        &self,
        organization_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, AppError>;
    async fn list_templates(
        &self,
        organization_id: &str,
    ) -> Result<Vec<ContractTemplate>, AppError>;
    async fn create_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError>;
    async fn update_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError>;
    async fn set_template_active(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), AppError>;
}

pub struct SettingsService<R> {
    repository: R,
}

impl<R: SettingsRepository> SettingsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_organization(
        &self,
        organization_id: &str,
    ) -> Result<OrganizationSettings, AppError> {
        self.repository.get_organization(organization_id).await
    }

    pub async fn update_organization(
        &self,
        organization_id: &str,
        actor_id: &str,
        mut input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, AppError> {
        input.name = input.name.trim().to_string();
        input.legal_name = clean_optional(input.legal_name);
        input.default_currency = input.default_currency.trim().to_uppercase();
        input.timezone = input.timezone.trim().to_string();
        if input.name.is_empty() {
            return Err(AppError::Validation(
                "Organization name is required.".to_string(),
            ));
        }
        if input.default_currency.len() != 3
            || !input
                .default_currency
                .bytes()
                .all(|byte| byte.is_ascii_uppercase())
        {
            return Err(AppError::Validation(
                "Default currency must be a three-letter currency code.".to_string(),
            ));
        }
        if input.timezone.is_empty()
            || input.timezone.len() > 64
            || !input.timezone.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+')
            })
        {
            return Err(AppError::Validation(
                "A valid time zone is required.".to_string(),
            ));
        }
        self.repository
            .update_organization(organization_id, actor_id, input)
            .await
    }

    pub async fn list_currencies(
        &self,
        organization_id: &str,
    ) -> Result<Vec<CurrencySettings>, AppError> {
        self.repository.list_currencies(organization_id).await
    }

    pub async fn set_currency_enabled(
        &self,
        organization_id: &str,
        actor_id: &str,
        currency_code: &str,
        is_enabled: bool,
    ) -> Result<(), AppError> {
        let currency_code = currency_code.trim().to_uppercase();
        if currency_code.len() != 3 || !currency_code.bytes().all(|byte| byte.is_ascii_uppercase())
        {
            return Err(AppError::Validation(
                "Currency must be a three-letter currency code.".to_string(),
            ));
        }
        self.repository
            .set_currency_enabled(organization_id, actor_id, &currency_code, is_enabled)
            .await
    }

    pub async fn get_profile(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<ProfileSettings, AppError> {
        self.repository.get_profile(organization_id, user_id).await
    }

    pub async fn update_profile(
        &self,
        organization_id: &str,
        user_id: &str,
        mut input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, AppError> {
        input.name = input.name.trim().to_string();
        input.email = input.email.trim().to_lowercase();
        input.current_password = clean_optional(input.current_password);
        input.new_password = clean_optional(input.new_password);
        if input.name.is_empty() {
            return Err(AppError::Validation("Name is required.".to_string()));
        }
        if input.email.len() > 254
            || !input.email.contains('@')
            || input.email.bytes().any(|byte| byte.is_ascii_whitespace())
        {
            return Err(AppError::Validation(
                "A valid email address is required.".to_string(),
            ));
        }
        if input.new_password.as_ref().is_some_and(|password| {
            password.chars().count() < 12 || input.current_password.is_none()
        }) {
            return Err(AppError::Validation(
                "Changing the password requires the current password and at least 12 new characters."
                    .to_string(),
            ));
        }
        if input.current_password.is_some() && input.new_password.is_none() {
            return Err(AppError::Validation(
                "Enter a new password to change your password.".to_string(),
            ));
        }
        self.repository
            .update_profile(organization_id, user_id, input)
            .await
    }

    pub async fn list_audit(
        &self,
        organization_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, AppError> {
        if !(1..=100).contains(&limit) || offset > 1_000_000 {
            return Err(AppError::Validation(
                "Audit page size must be from 1 to 100 and offset cannot exceed 1000000."
                    .to_string(),
            ));
        }
        self.repository
            .list_audit(organization_id, limit, offset)
            .await
    }

    pub async fn list_templates(
        &self,
        organization_id: &str,
    ) -> Result<Vec<ContractTemplate>, AppError> {
        self.repository.list_templates(organization_id).await
    }

    pub async fn create_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError> {
        self.repository
            .create_template(organization_id, actor_id, validate_template(input)?)
            .await
    }

    pub async fn update_template(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        input: ContractTemplateInput,
    ) -> Result<ContractTemplate, AppError> {
        self.repository
            .update_template(
                organization_id,
                actor_id,
                template_id,
                validate_template(input)?,
            )
            .await
    }

    pub async fn set_template_active(
        &self,
        organization_id: &str,
        actor_id: &str,
        template_id: &str,
        is_active: bool,
    ) -> Result<(), AppError> {
        self.repository
            .set_template_active(organization_id, actor_id, template_id, is_active)
            .await
    }
}

fn validate_template(mut input: ContractTemplateInput) -> Result<ContractTemplateInput, AppError> {
    input.template_key = input.template_key.trim().to_lowercase();
    input.name = input.name.trim().to_string();
    input.description = clean_optional(input.description);
    input.locale = input.locale.trim().to_lowercase();
    input.content_body = input.content_body.trim().to_string();
    input.placeholders_json =
        clean_optional(input.placeholders_json).or_else(|| Some("{}".to_string()));
    input.change_note = clean_optional(input.change_note);
    if input.template_key.is_empty()
        || input.template_key.len() > 80
        || !input.template_key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(AppError::Validation(
            "Template key must use lowercase letters, digits, hyphens, or underscores.".to_string(),
        ));
    }
    if input.name.is_empty() || input.content_body.is_empty() {
        return Err(AppError::Validation(
            "Template name and content are required.".to_string(),
        ));
    }
    if !matches!(
        input.lease_type.as_str(),
        "residential" | "commercial" | "short_term" | "other"
    ) {
        return Err(AppError::Validation("Unsupported lease type.".to_string()));
    }
    if !matches!(input.content_format.as_str(), "text" | "markdown" | "html") {
        return Err(AppError::Validation(
            "Template content format must be text, markdown, or html.".to_string(),
        ));
    }
    if input.locale.is_empty() || input.locale.len() > 35 {
        return Err(AppError::Validation(
            "Template locale must contain between 1 and 35 characters.".to_string(),
        ));
    }
    let placeholders = input.placeholders_json.as_deref().unwrap_or("{}");
    serde_json::from_str::<serde_json::Value>(placeholders).map_err(|_| {
        AppError::Validation("Template placeholders must be valid JSON.".to_string())
    })?;
    Ok(input)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates_template_inputs() {
        let input = validate_template(ContractTemplateInput {
            template_key: "  LEASE-AR ".to_string(),
            name: " Standard lease ".to_string(),
            description: Some(" ".to_string()),
            lease_type: "residential".to_string(),
            locale: "AR".to_string(),
            content_format: "markdown".to_string(),
            content_body: " Body ".to_string(),
            placeholders_json: None,
            change_note: None,
        })
        .expect("valid contract template");
        assert_eq!(input.template_key, "lease-ar");
        assert_eq!(input.description, None);
        assert_eq!(input.placeholders_json.as_deref(), Some("{}"));
    }

    #[test]
    fn rejects_invalid_placeholders_and_template_keys() {
        let mut input = ContractTemplateInput {
            template_key: "Bad Key".to_string(),
            name: "Lease".to_string(),
            description: None,
            lease_type: "residential".to_string(),
            locale: "ar".to_string(),
            content_format: "text".to_string(),
            content_body: "content".to_string(),
            placeholders_json: Some("{".to_string()),
            change_note: None,
        };
        assert!(matches!(
            validate_template(input.clone()),
            Err(AppError::Validation(_))
        ));
        input.template_key = "valid-key".to_string();
        assert!(matches!(
            validate_template(input),
            Err(AppError::Validation(_))
        ));
    }
}
