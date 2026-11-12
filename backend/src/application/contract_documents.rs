use crate::{
    application::{ApplicationError, Page, PageRequest, require_permission},
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
    pub document_style: ContractDocumentStyle,
    pub created_by: String,
    pub created_at: String,
    pub status: String,
    pub status_reason: Option<String>,
    pub status_effective_date: Option<String>,
    pub parent_contract_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContractDocumentStyle {
    pub paper_size: String,
    pub accent_color: String,
    pub font_family: String,
    pub font_size: u8,
    pub alignment: String,
    pub page_margin_mm: u8,
    pub show_branding: bool,
    pub brand_label: String,
    pub show_contract_number: bool,
    pub show_footer: bool,
    pub footer_text: String,
}

impl Default for ContractDocumentStyle {
    fn default() -> Self {
        Self {
            paper_size: "a4".to_string(),
            accent_color: "#C7962F".to_string(),
            font_family: "Tahoma".to_string(),
            font_size: 16,
            alignment: "start".to_string(),
            page_margin_mm: 18,
            show_branding: true,
            brand_label: String::new(),
            show_contract_number: true,
            show_footer: true,
            footer_text: String::new(),
        }
    }
}

impl ContractDocumentStyle {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !matches!(self.paper_size.as_str(), "a4" | "letter")
            || !matches!(
                self.font_family.as_str(),
                "Tahoma" | "Arial" | "Georgia" | "Noto Naskh Arabic"
            )
            || !matches!(self.font_size, 12 | 14 | 16 | 18)
            || !matches!(self.alignment.as_str(), "start" | "justify")
            || !(12..=30).contains(&self.page_margin_mm)
            || self.brand_label.chars().count() > 100
            || self.footer_text.chars().count() > 400
            || self.accent_color.len() != 7
            || !self.accent_color.starts_with('#')
            || !self.accent_color[1..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(validation("Contract document design settings are invalid."));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDocumentInput {
    pub draft_id: Option<String>,
    pub parent_contract_id: Option<String>,
    pub status: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEndInput {
    pub effective_date: String,
    pub reason: String,
}

#[async_trait::async_trait]
pub trait ContractDocumentRepository: Send + Sync {
    async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<ContractDocument>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
        page: PageRequest,
    ) -> Result<Page<ContractDocument>, ApplicationError> {
        Ok(Page::from_items(
            self.list(organization_id, property_id).await?,
            page.limit(),
        ))
    }

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractDocumentInput,
    ) -> Result<ContractDocument, ApplicationError>;

    async fn delete_draft(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
    ) -> Result<(), ApplicationError>;

    async fn issue_draft(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
    ) -> Result<ContractDocument, ApplicationError>;

    async fn end(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
        input: ContractEndInput,
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

    pub async fn list_page(
        &self,
        actor: ActorContext,
        property_id: Option<&str>,
        page: PageRequest,
    ) -> Result<Page<ContractDocument>, ApplicationError> {
        require_permission(actor, Permission::ViewContracts)?;
        self.repository
            .list_page(
                &actor.organization_id().to_string(),
                property_id,
                page,
            )
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

    pub async fn delete_draft(
        &self,
        actor: ActorContext,
        contract_id: &str,
    ) -> Result<(), ApplicationError> {
        require_permission(actor, Permission::CreateContracts)?;
        self.repository
            .delete_draft(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                contract_id,
            )
            .await
    }

    pub async fn issue_draft(
        &self,
        actor: ActorContext,
        contract_id: &str,
    ) -> Result<ContractDocument, ApplicationError> {
        require_permission(actor, Permission::CreateContracts)?;
        self.repository
            .issue_draft(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                contract_id,
            )
            .await
    }

    pub async fn end(
        &self,
        actor: ActorContext,
        contract_id: &str,
        input: ContractEndInput,
    ) -> Result<ContractDocument, ApplicationError> {
        require_permission(actor, Permission::CreateContracts)?;
        validate_end(&input)?;
        self.repository
            .end(
                &actor.organization_id().to_string(),
                &actor.user_id().to_string(),
                contract_id,
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
    if !matches!(input.status.as_str(), "draft" | "active")
        || input
            .draft_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        || input
            .parent_contract_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        || (input.draft_id.is_some() && input.status != "draft")
    {
        return Err(validation("Contract state or relationship is invalid."));
    }
    Ok(())
}

fn validate_end(input: &ContractEndInput) -> Result<(), ApplicationError> {
    if NaiveDate::parse_from_str(&input.effective_date, "%Y-%m-%d").is_err()
        || input.reason.trim().is_empty()
        || input.reason.trim().len() > 1000
    {
        return Err(validation("Contract end date and reason are required and must be valid."));
    }
    Ok(())
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_document_style_is_valid_and_deserializes_legacy_settings() {
        let style = serde_json::from_str::<ContractDocumentStyle>("{}")
            .expect("legacy template settings use the default design");
        style.validate().expect("default design is valid");
        assert_eq!(style.paper_size, "a4");
        assert_eq!(style.font_size, 16);
    }

    #[test]
    fn document_style_rejects_unsafe_or_unsupported_values() {
        let mut style = ContractDocumentStyle::default();
        style.accent_color = "red;display:none".to_string();
        assert!(style.validate().is_err());

        style = ContractDocumentStyle::default();
        style.page_margin_mm = 31;
        assert!(style.validate().is_err());
    }

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

        async fn delete_draft(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<(), ApplicationError> {
            unreachable!()
        }

        async fn issue_draft(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<ContractDocument, ApplicationError> {
            unreachable!()
        }

        async fn end(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: ContractEndInput,
        ) -> Result<ContractDocument, ApplicationError> {
            unreachable!()
        }
    }

    fn input() -> ContractDocumentInput {
        ContractDocumentInput {
            draft_id: None,
            parent_contract_id: None,
            status: "active".to_string(),
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
