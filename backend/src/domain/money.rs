use super::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CurrencyCode {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_ascii_uppercase();
        if value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(DomainError::Validation(
                "Currency must be a three-letter ASCII code.",
            ));
        }
        Ok(Self(value))
    }
}

impl From<CurrencyCode> for String {
    fn from(value: CurrencyCode) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount_minor: i64,
    currency: CurrencyCode,
}

impl Money {
    pub fn new(amount_minor: i64, currency: CurrencyCode) -> Result<Self, DomainError> {
        if amount_minor < 0 {
            return Err(DomainError::Validation("Money amount cannot be negative."));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_codes_are_normalized_and_checked() {
        assert_eq!(
            CurrencyCode::try_from(" sar ".to_string())
                .expect("valid code")
                .as_str(),
            "SAR"
        );
        assert!(CurrencyCode::try_from("US".to_string()).is_err());
    }

    #[test]
    fn money_rejects_negative_amounts() {
        let currency = CurrencyCode::try_from("SAR".to_string()).expect("valid code");
        assert!(Money::new(-1, currency).is_err());
    }
}
