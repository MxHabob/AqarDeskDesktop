use crate::{application::ApplicationError, domain::ActorContext};
use serde::{Deserialize, Serialize};

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

#[async_trait::async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn get(
        &self,
        organization_id: &str,
        user_id: &str,
    ) -> Result<ProfileSettings, ApplicationError>;

    async fn update(
        &self,
        organization_id: &str,
        user_id: &str,
        input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, ApplicationError>;
}

pub struct ProfileService<R> {
    repository: R,
}

impl<R: ProfileRepository> ProfileService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get(&self, actor: ActorContext) -> Result<ProfileSettings, ApplicationError> {
        self.repository
            .get(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        mut input: ProfileSettingsInput,
    ) -> Result<ProfileSettings, ApplicationError> {
        input.name = input.name.trim().to_string();
        input.email = input.email.trim().to_ascii_lowercase();
        input.current_password = clean_optional(input.current_password);
        input.new_password = clean_optional(input.new_password);
        if input.name.is_empty() {
            return Err(validation("Name is required."));
        }
        if !valid_email(&input.email) {
            return Err(validation("A valid email address is required."));
        }
        match (&input.current_password, &input.new_password) {
            (Some(_), None) => return Err(validation("Enter a new password to change it.")),
            (None, Some(_)) => {
                return Err(validation(
                    "Changing the password requires the current password.",
                ));
            }
            (Some(_), Some(password)) if password.chars().count() < 12 => {
                return Err(validation(
                    "New passwords must contain at least 12 characters.",
                ));
            }
            _ => {}
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

fn valid_email(value: &str) -> bool {
    value.len() <= 254
        && value.contains('@')
        && !value.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}
