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
    pub allocations: Vec<PaymentAllocationInput>,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAllocationInput {
    pub contract_document_id: String,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAllocation {
    pub contract_document_id: String,
    pub contract_number: String,
    pub property_name: String,
    pub property_code: String,
    pub unit_number: String,
    pub amount_minor: i64,
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
    pub allocations: Vec<PaymentAllocation>,
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
        let mut allocation_total = 0_i64;
        let mut contract_ids = std::collections::HashSet::new();
        let allocations_are_valid = !input.allocations.is_empty()
            && input.allocations.iter().all(|allocation| {
                if allocation.contract_document_id.trim().is_empty()
                    || allocation.amount_minor <= 0
                    || !contract_ids.insert(allocation.contract_document_id.as_str())
                {
                    return false;
                }
                let Some(total) = allocation_total.checked_add(allocation.amount_minor) else {
                    return false;
                };
                allocation_total = total;
                true
            });
        if !allocations_are_valid || allocation_total != input.amount_minor {
            return Err(DomainError::Validation(
                "Payment allocations must be positive, unique, and equal the payment total.",
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
            allocations: input
                .allocations
                .into_iter()
                .map(|allocation| PaymentAllocation {
                    contract_document_id: allocation.contract_document_id,
                    contract_number: String::new(),
                    property_name: String::new(),
                    property_code: String::new(),
                    unit_number: String::new(),
                    amount_minor: allocation.amount_minor,
                })
                .collect(),
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
            allocations: vec![PaymentAllocationInput {
                contract_document_id: "contract-1".to_string(),
                amount_minor,
            }],
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

    #[test]
    fn payment_rejects_allocations_that_do_not_match_the_receipt() {
        let mut input = input(10_000);
        input.allocations[0].amount_minor = 9_999;
        assert!(
            Payment::record(
                PaymentId::new(),
                OrganizationId::new(),
                UserId::new(),
                input
            )
            .is_err()
        );
    }

    #[test]
    fn payment_rejects_duplicate_contract_allocations() {
        let mut input = input(10_000);
        input.allocations = vec![
            PaymentAllocationInput {
                contract_document_id: "contract-1".to_string(),
                amount_minor: 5_000,
            },
            PaymentAllocationInput {
                contract_document_id: "contract-1".to_string(),
                amount_minor: 5_000,
            },
        ];
        assert!(
            Payment::record(
                PaymentId::new(),
                OrganizationId::new(),
                UserId::new(),
                input
            )
            .is_err()
        );
    }
}
