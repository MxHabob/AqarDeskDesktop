use crate::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
}

#[async_trait::async_trait]
pub trait TenantRepository {
    type Item;

    async fn list(&self, organization_id: &str) -> Result<Vec<Self::Item>, AppError>;
    async fn create(
        &self,
        organization_id: &str,
        input: TenantInput,
    ) -> Result<Self::Item, AppError>;
    async fn update(
        &self,
        organization_id: &str,
        tenant_id: &str,
        input: TenantInput,
    ) -> Result<Self::Item, AppError>;
    async fn archive(&self, organization_id: &str, tenant_id: &str) -> Result<(), AppError>;
}

pub struct TenantService<R> {
    repository: R,
}

impl<R: TenantRepository> TenantService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(&self, organization_id: &str) -> Result<Vec<R::Item>, AppError> {
        self.repository.list(organization_id).await
    }

    pub async fn create(
        &self,
        organization_id: &str,
        input: TenantInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .create(organization_id, validate(input)?)
            .await
    }

    pub async fn update(
        &self,
        organization_id: &str,
        tenant_id: &str,
        input: TenantInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .update(organization_id, tenant_id, validate(input)?)
            .await
    }

    pub async fn archive(&self, organization_id: &str, tenant_id: &str) -> Result<(), AppError> {
        self.repository.archive(organization_id, tenant_id).await
    }
}

fn validate(mut input: TenantInput) -> Result<TenantInput, AppError> {
    input.full_name = input.full_name.trim().to_string();
    input.email = super::clean_optional(input.email).map(|value| value.to_lowercase());
    input.phone = super::clean_optional(input.phone);
    input.mailing_address = super::clean_optional(input.mailing_address);
    input.notes = super::clean_optional(input.notes);
    if input.full_name.is_empty() {
        return Err(AppError::Validation("Tenant name is required.".to_string()));
    }
    if input.email.as_ref().is_some_and(|email| {
        email.len() > 254
            || !email.contains('@')
            || email.bytes().any(|byte| byte.is_ascii_whitespace())
    }) {
        return Err(AppError::Validation(
            "A valid tenant email address is required.".to_string(),
        ));
    }
    if input.phone.is_none() && input.email.is_none() {
        return Err(AppError::Validation(
            "A tenant email address or phone number is required.".to_string(),
        ));
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_at_least_one_tenant_contact_method() {
        let error = validate(TenantInput {
            full_name: "Tenant".to_string(),
            email: None,
            phone: None,
            mailing_address: None,
            notes: None,
        })
        .expect_err("tenant without contact details");

        assert!(matches!(error, AppError::Validation(_)));
    }
}
