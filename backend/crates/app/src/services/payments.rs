use crate::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PAYMENT_METHODS: &[&str] =
    &["cash", "bank_transfer", "card", "cheque", "online", "other"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub tenant_id: String,
    pub tenant_name: String,
    pub received_at: String,
    pub amount_minor: i64,
    pub currency_code: String,
    pub payment_method: String,
    pub external_reference: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub voided_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInput {
    pub tenant_id: String,
    pub received_at: String,
    pub amount_minor: i64,
    pub currency_code: String,
    pub payment_method: String,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
}

#[async_trait::async_trait]
pub trait PaymentRepository {
    async fn list(&self, organization_id: &str) -> Result<Vec<Payment>, AppError>;
    async fn record(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PaymentInput,
    ) -> Result<Payment, AppError>;
    async fn void(
        &self,
        organization_id: &str,
        actor_id: &str,
        payment_id: &str,
    ) -> Result<(), AppError>;
}

pub struct PaymentService<R> {
    repository: R,
}

impl<R: PaymentRepository> PaymentService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(&self, organization_id: &str) -> Result<Vec<Payment>, AppError> {
        self.repository.list(organization_id).await
    }

    pub async fn record(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PaymentInput,
    ) -> Result<Payment, AppError> {
        let input = validate(input)?;
        self.repository
            .record(organization_id, actor_id, input)
            .await
    }

    pub async fn void(
        &self,
        organization_id: &str,
        actor_id: &str,
        payment_id: &str,
    ) -> Result<(), AppError> {
        let payment_id = payment_id.trim();
        if payment_id.is_empty() {
            return Err(AppError::Validation(
                "Payment identifier is required.".to_string(),
            ));
        }
        self.repository
            .void(organization_id, actor_id, payment_id)
            .await
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate(mut input: PaymentInput) -> Result<PaymentInput, AppError> {
    input.tenant_id = input.tenant_id.trim().to_string();
    input.currency_code = input.currency_code.trim().to_uppercase();
    input.payment_method = input.payment_method.trim().to_lowercase();
    input.external_reference = clean_optional(input.external_reference);
    input.notes = clean_optional(input.notes);
    if input.tenant_id.is_empty() {
        return Err(AppError::Validation("A tenant is required.".to_string()));
    }
    if input.amount_minor <= 0 {
        return Err(AppError::Validation(
            "Payment amount must be greater than zero.".to_string(),
        ));
    }
    if input.currency_code.len() != 3
        || !input
            .currency_code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase())
    {
        return Err(AppError::Validation(
            "Currency must be a three-letter currency code.".to_string(),
        ));
    }
    if !PAYMENT_METHODS.contains(&input.payment_method.as_str()) {
        return Err(AppError::Validation(
            "Unsupported payment method.".to_string(),
        ));
    }
    if input
        .external_reference
        .as_ref()
        .is_some_and(|value| value.len() > 120)
        || input.notes.as_ref().is_some_and(|value| value.len() > 2000)
    {
        return Err(AppError::Validation(
            "Payment reference or notes exceed the allowed length.".to_string(),
        ));
    }
    let received_at = DateTime::parse_from_rfc3339(input.received_at.trim()).map_err(|_| {
        AppError::Validation("Payment date must be a valid ISO-8601 timestamp.".to_string())
    })?;
    input.received_at = received_at.with_timezone(&Utc).to_rfc3339();
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_payment_input_and_validates_catalog_values() {
        let input = validate(PaymentInput {
            tenant_id: " tenant-1 ".to_string(),
            received_at: "2026-11-11T12:00:00+03:00".to_string(),
            amount_minor: 12500,
            currency_code: "sar".to_string(),
            payment_method: "CASH".to_string(),
            external_reference: Some("  RCPT-1 ".to_string()),
            notes: Some(" ".to_string()),
        })
        .expect("valid payment input");
        assert_eq!(input.tenant_id, "tenant-1");
        assert_eq!(input.currency_code, "SAR");
        assert_eq!(input.payment_method, "cash");
        assert_eq!(input.external_reference.as_deref(), Some("RCPT-1"));
        assert_eq!(input.notes, None);
        assert!(input.received_at.ends_with('Z'));
    }

    #[test]
    fn rejects_invalid_payment_methods_and_amounts() {
        let input = PaymentInput {
            tenant_id: "tenant-1".to_string(),
            received_at: "2026-11-11T12:00:00Z".to_string(),
            amount_minor: 0,
            currency_code: "SAR".to_string(),
            payment_method: "wire".to_string(),
            external_reference: None,
            notes: None,
        };
        assert!(matches!(validate(input), Err(AppError::Validation(_))));
        let input = PaymentInput {
            tenant_id: "tenant-1".to_string(),
            received_at: "invalid date".to_string(),
            amount_minor: 10,
            currency_code: "SAR".to_string(),
            payment_method: "cash".to_string(),
            external_reference: None,
            notes: None,
        };
        assert!(matches!(validate(input), Err(AppError::Validation(_))));
    }
}
