use crate::{
    application::{ApplicationError, require_permission},
    domain::{ActorContext, Permission},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDocument {
    pub id: String,
    pub organization_id: String,
    pub contract_number: String,
    pub template_id: String,
    pub template_version_id: String,
    pub template_name: String,
    pub template_locale: String,
    pub property_id: String,
    pub property_name: String,
    pub property_code: String,
    pub unit_id: String,
    pub unit_number: String,
    pub tenant_id: String,
    pub tenant_name: String,
    pub start_date: String,
    pub end_date: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub rent_amount_minor: i64,
    pub deposit_amount_minor: i64,
    pub document_body: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDocumentInput {
    pub template_id: String,
    pub tenant_id: String,
    pub property_id: String,
    pub unit_id: String,
    pub start_date: String,
    pub end_date: String,
    pub currency_code: String,
    pub rent_amount_minor: i64,
    pub deposit_amount_minor: i64,
}

#[async_trait::async_trait]
pub trait ContractDocumentRepository: Send + Sync {
    async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<ContractDocument>, ApplicationError>;

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractDocumentInput,
    ) -> Result<ContractDocument, ApplicationError>;
}

pub struct ContractDocumentService<R> {
    repository: R,
}

impl<R: ContractDocumentRepository> ContractDocumentService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        actor: ActorContext,
        property_id: Option<&str>,
    ) -> Result<Vec<ContractDocument>, ApplicationError> {
        require_permission(actor, Permission::ViewContracts)?;
        self.repository
            .list(&actor.organization_id().to_string(), property_id)
            .await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: ContractDocumentInput,
    ) -> Result<ContractDocument, ApplicationError> {
        require_permission(actor, Permission::CreateContracts)?;
        require_permission(actor, Permission::ViewContracts)?;
        validate(&input)?;
        self.repository
            .create(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                input,
            )
            .await
    }
}

fn validate(input: &ContractDocumentInput) -> Result<(), ApplicationError> {
    let start_date = NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d");
    let end_date = NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d");
    if start_date.is_err()
        || end_date.is_err()
        || start_date.ok() > end_date.ok()
        || input.template_id.trim().is_empty()
        || input.tenant_id.trim().is_empty()
        || input.property_id.trim().is_empty()
        || input.unit_id.trim().is_empty()
    {
        return Err(validation("Contract details or lease dates are invalid."));
    }
    if input.currency_code.len() != 3
        || !input
            .currency_code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase())
        || input.rent_amount_minor <= 0
        || input.deposit_amount_minor < 0
    {
        return Err(validation("Contract currency or amounts are invalid."));
    }
    Ok(())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UnreachableRepository;

    #[async_trait::async_trait]
    impl ContractDocumentRepository for UnreachableRepository {
        async fn list(
            &self,
            _: &str,
            _: Option<&str>,
        ) -> Result<Vec<ContractDocument>, ApplicationError> {
            Ok(Vec::new())
        }

        async fn create(
            &self,
            _: &str,
            _: &str,
            _: ContractDocumentInput,
        ) -> Result<ContractDocument, ApplicationError> {
            unreachable!()
        }
    }

    fn input() -> ContractDocumentInput {
        ContractDocumentInput {
            template_id: "template".to_string(),
            tenant_id: "tenant".to_string(),
            property_id: "property".to_string(),
            unit_id: "unit".to_string(),
            start_date: "2026-01-01".to_string(),
            end_date: "2026-12-31".to_string(),
            currency_code: "SAR".to_string(),
            rent_amount_minor: 120_000,
            deposit_amount_minor: 0,
        }
    }

    #[tokio::test]
    async fn contract_operations_require_their_specific_permissions() {
        let actor = ActorContext::with_permissions(
            crate::domain::UserId::new(),
            crate::domain::OrganizationId::new(),
            crate::domain::AccessRole::Staff,
            vec![Permission::ViewContracts],
        );
        let service = ContractDocumentService::new(UnreachableRepository);
        assert!(service.list(actor, None).await.is_ok());
        assert!(matches!(
            service.create(actor, input()).await,
            Err(ApplicationError::Forbidden)
        ));
    }

    #[test]
    fn contract_input_rejects_invalid_dates_and_amounts() {
        let mut input = input();
        input.end_date = "2026-02-30".to_string();
        assert!(validate(&input).is_err());
        input.end_date = "2025-12-31".to_string();
        assert!(validate(&input).is_err());
        input.end_date = "2026-12-31".to_string();
        input.rent_amount_minor = 0;
        assert!(validate(&input).is_err());
    }
}
