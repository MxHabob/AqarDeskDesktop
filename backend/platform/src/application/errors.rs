use crate::domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("The requested resource was not found.")]
    NotFound,
    #[error("This account is not permitted to perform this operation.")]
    Forbidden,
    #[error("Authentication failed.")]
    Authentication,
    #[error("Initial administrator setup has already been completed.")]
    AlreadyConfigured,
    #[error("A persistence operation failed.")]
    Persistence,
}
