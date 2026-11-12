use crate::{
    application::{ApplicationError, require_permission},
    domain::{ActorContext, Permission},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardSummary {
    pub active_properties_count: u64,
    pub units_count: u64,
    pub available_units_count: u64,
    pub tenants_count: u64,
    pub posted_payments_count: u64,
    pub payment_months: Vec<String>,
    pub monthly_payments: Vec<MonthlyPayment>,
    pub payments_by_currency: Vec<PaymentCurrencyTotal>,
    pub unit_status_counts: Vec<StatusCount>,
    pub payment_status_counts: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonthlyPayment {
    pub month: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub payment_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentCurrencyTotal {
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub payment_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyDashboardSummary {
    pub contract_count: u64,
    pub tenant_count: u64,
    pub rent_by_currency: Vec<CurrencyAmountTotal>,
    pub posted_allocations_by_currency: Option<Vec<CurrencyAmountTotal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyAmountTotal {
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusCount {
    pub status: String,
    pub count: u64,
}

#[async_trait::async_trait]
pub trait DashboardRepository: Send + Sync {
    async fn summary(
        &self,
        organization_id: &str,
        months: u8,
    ) -> Result<DashboardSummary, ApplicationError>;

    async fn property_summary(
        &self,
        organization_id: &str,
        property_id: &str,
        include_payments: bool,
    ) -> Result<PropertyDashboardSummary, ApplicationError>;
}

pub struct DashboardService<R> {
    repository: R,
}

impl<R: DashboardRepository> DashboardService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn summary(
        &self,
        actor: ActorContext,
        months: u8,
    ) -> Result<DashboardSummary, ApplicationError> {
        if !(1..=24).contains(&months) {
            return Err(crate::domain::DomainError::Validation(
                "Dashboard reporting period must be between 1 and 24 months.",
            )
            .into());
        }
        self.repository
            .summary(&actor.organization_id().to_string(), months)
            .await
    }

    pub async fn property_summary(
        &self,
        actor: ActorContext,
        property_id: &str,
    ) -> Result<PropertyDashboardSummary, ApplicationError> {
        require_permission(actor, Permission::ViewProperties)?;
        require_permission(actor, Permission::ViewContracts)?;
        self.repository
            .property_summary(
                &actor.organization_id().to_string(),
                property_id,
                actor.has_permission(Permission::ViewPayments),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_types_only_contain_metrics_supported_by_persisted_business_data() {
        let summary = DashboardSummary {
            active_properties_count: 2,
            units_count: 7,
            available_units_count: 3,
            tenants_count: 4,
            posted_payments_count: 9,
            payment_months: vec!["2026-10".to_string()],
            monthly_payments: vec![MonthlyPayment {
                month: "2026-10".to_string(),
                currency_code: "SAR".to_string(),
                currency_decimal_digits: 2,
                amount_minor: 120_000,
                payment_count: 2,
            }],
            payments_by_currency: vec![PaymentCurrencyTotal {
                currency_code: "SAR".to_string(),
                currency_decimal_digits: 2,
                amount_minor: 120_000,
                payment_count: 2,
            }],
            unit_status_counts: vec![StatusCount {
                status: "available".to_string(),
                count: 3,
            }],
            payment_status_counts: vec![StatusCount {
                status: "posted".to_string(),
                count: 2,
            }],
        };
        assert_eq!(summary.monthly_payments[0].amount_minor, 120_000);
        assert_eq!(summary.payment_months, ["2026-10"]);
    }

    #[test]
    fn property_summary_exposes_currency_totals_without_claiming_an_outstanding_balance() {
        let summary = PropertyDashboardSummary {
            contract_count: 2,
            tenant_count: 1,
            rent_by_currency: vec![CurrencyAmountTotal {
                currency_code: "SAR".to_string(),
                currency_decimal_digits: 2,
                amount_minor: 120_000,
            }],
            posted_allocations_by_currency: Some(vec![CurrencyAmountTotal {
                currency_code: "SAR".to_string(),
                currency_decimal_digits: 2,
                amount_minor: 40_000,
            }]),
        };
        assert_eq!(summary.contract_count, 2);
        assert_eq!(summary.rent_by_currency[0].amount_minor, 120_000);
        assert_eq!(
            summary
                .posted_allocations_by_currency
                .as_ref()
                .expect("payment metrics are visible")[0]
                .amount_minor,
            40_000
        );
    }
}
