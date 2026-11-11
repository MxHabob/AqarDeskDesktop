use crate::application::ApplicationError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PublicError {
    pub code: &'static str,
    pub message: &'static str,
}

impl From<&ApplicationError> for PublicError {
    fn from(error: &ApplicationError) -> Self {
        match error {
            ApplicationError::Domain(error) => Self {
                code: "validation",
                message: match error {
                    crate::domain::DomainError::Validation(message) => message,
                },
            },
            ApplicationError::NotFound => Self {
                code: "not_found",
                message: "The requested resource was not found.",
            },
            ApplicationError::Forbidden => Self {
                code: "forbidden",
                message: "This account is not permitted to perform this operation.",
            },
            ApplicationError::Authentication => Self {
                code: "authentication",
                message: "Email or password is incorrect.",
            },
            ApplicationError::AlreadyConfigured => Self {
                code: "already_configured",
                message: "Initial administrator setup has already been completed.",
            },
            ApplicationError::Persistence => Self {
                code: "persistence",
                message: "A database operation failed.",
            },
        }
    }
}
