use super::{CurrencyCode, DomainError, Money, OrganizationId, PaymentId, TenantId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    BankTransfer,
    Card,
    Cheque,
    Online,
    Other,
}

impl PaymentMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::BankTransfer => "bank_transfer",
            Self::Card => "card",
            Self::Cheque => "cheque",
            Self::Online => "online",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "cash" => Ok(Self::Cash),
            "bank_transfer" => Ok(Self::BankTransfer),
            "card" => Ok(Self::Card),
            "cheque" => Ok(Self::Cheque),
            "online" => Ok(Self::Online),
            "other" => Ok(Self::Other),
            _ => Err(DomainError::Validation("Unsupported payment method.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Posted,
    Void,
}

impl PaymentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Posted => "posted",
            Self::Void => "void",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "posted" => Ok(Self::Posted),
            "void" => Ok(Self::Void),
            _ => Err(DomainError::Validation("Unsupported payment status.")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentInput {
    pub tenant_id: TenantId,
    pub received_at: DateTime<Utc>,
    pub amount_minor: i64,
    pub currency_code: CurrencyCode,
    pub payment_method: PaymentMethod,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payment {
    pub id: PaymentId,
    pub organization_id: OrganizationId,
    pub actor_id: Option<UserId>,
    pub tenant_id: TenantId,
    pub received_at: DateTime<Utc>,
    pub amount: Money,
    pub method: PaymentMethod,
    pub external_reference: Option<String>,
    pub status: PaymentStatus,
    pub notes: Option<String>,
    pub voided_at: Option<DateTime<Utc>>,
}

impl Payment {
    pub fn record(
        id: PaymentId,
        organization_id: OrganizationId,
        actor_id: UserId,
        mut input: PaymentInput,
    ) -> Result<Self, DomainError> {
        input.external_reference = clean_optional(input.external_reference);
        input.notes = clean_optional(input.notes);
        if input.amount_minor <= 0 {
            return Err(DomainError::Validation(
                "Payment amount must be greater than zero.",
            ));
        }
        if input
            .external_reference
            .as_ref()
            .is_some_and(|reference| reference.len() > 120)
            || input.notes.as_ref().is_some_and(|notes| notes.len() > 2000)
        {
            return Err(DomainError::Validation(
                "Payment reference or notes exceed the allowed length.",
            ));
        }
        let amount = Money::new(input.amount_minor, input.currency_code)?;
        Ok(Self {
            id,
            organization_id,
            actor_id: Some(actor_id),
            tenant_id: input.tenant_id,
            received_at: input.received_at,
            amount,
            method: input.payment_method,
            external_reference: input.external_reference,
            status: PaymentStatus::Posted,
            notes: input.notes,
            voided_at: None,
        })
    }

    pub fn void(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.status == PaymentStatus::Void {
            return Err(DomainError::Validation("Payment is already void."));
        }
        self.status = PaymentStatus::Void;
        self.voided_at = Some(now);
        Ok(())
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(amount_minor: i64) -> PaymentInput {
        let currency_code = CurrencyCode::try_from("sar".to_string()).expect("valid currency");
        PaymentInput {
            tenant_id: TenantId::new(),
            received_at: Utc::now(),
            amount_minor,
            currency_code,
            payment_method: PaymentMethod::Cash,
            external_reference: None,
            notes: None,
        }
    }

    fn payment() -> Payment {
        Payment::record(
            PaymentId::new(),
            OrganizationId::new(),
            UserId::new(),
            input(10_000),
        )
        .expect("valid payment")
    }

    #[test]
    fn payment_void_is_a_one_way_state_transition() {
        let mut payment = payment();
        payment.void(Utc::now()).expect("void posted payment");
        assert_eq!(payment.status, PaymentStatus::Void);
        assert!(payment.voided_at.is_some());
        assert!(payment.void(Utc::now()).is_err());
    }

    #[test]
    fn payment_rejects_zero_amount() {
        assert!(matches!(
            Payment::record(
                PaymentId::new(),
                OrganizationId::new(),
                UserId::new(),
                input(0)
            ),
            Err(DomainError::Validation(_))
        ));
    }
}
