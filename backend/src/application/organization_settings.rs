use crate::{
    application::{ApplicationError, require_admin},
    domain::ActorContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub logo_data_url: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettingsInput {
    pub name: String,
    pub legal_name: Option<String>,
    pub logo_data_url: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

#[async_trait::async_trait]
pub trait OrganizationSettingsRepository: Send + Sync {
    async fn get(&self, organization_id: &str) -> Result<OrganizationSettings, ApplicationError>;

    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, ApplicationError>;
}

pub struct OrganizationSettingsService<R> {
    repository: R,
}

impl<R: OrganizationSettingsRepository> OrganizationSettingsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get(&self, actor: ActorContext) -> Result<OrganizationSettings, ApplicationError> {
        self.repository
            .get(&actor.organization_id().to_string())
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        mut input: OrganizationSettingsInput,
    ) -> Result<OrganizationSettings, ApplicationError> {
        require_admin(actor)?;
        input.name = input.name.trim().to_string();
        input.legal_name = clean_optional(input.legal_name);
        input.logo_data_url = clean_optional(input.logo_data_url);
        input.default_currency = input.default_currency.trim().to_ascii_uppercase();
        input.timezone = input.timezone.trim().to_string();
        if input.name.is_empty() {
            return Err(validation("Organization name is required."));
        }
        if input.default_currency.len() != 3
            || !input
                .default_currency
                .bytes()
                .all(|byte| byte.is_ascii_uppercase())
        {
            return Err(validation("Default currency must be a three-letter code."));
        }
        if input.timezone.is_empty()
            || input.timezone.len() > 64
            || !input.timezone.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+')
            })
            || input.timezone.parse::<chrono_tz::Tz>().is_err()
        {
            return Err(validation("A valid IANA time zone is required."));
        }
        if input.logo_data_url.as_deref().is_some_and(|logo| {
            logo.len() > 1_500_000
                || ![
                    "data:image/png;base64,",
                    "data:image/jpeg;base64,",
                    "data:image/webp;base64,",
                ]
                .iter()
                .any(|prefix| {
                    logo.strip_prefix(prefix).is_some_and(|data| {
                        !data.is_empty()
                            && data.len() % 4 == 0
                            && data.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')
                            })
                    })
                })
        }) {
            return Err(validation(
                "Organization logo must be a PNG, JPEG, or WebP image up to 1 MiB.",
            ));
        }
        self.repository
            .update(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                input,
            )
            .await
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}
