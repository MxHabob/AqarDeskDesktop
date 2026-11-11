use crate::AppError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitInput {
    pub property_id: String,
    pub unit_number: String,
    pub unit_type: String,
    pub floor_number: Option<i32>,
    pub area_square_meters: Option<Decimal>,
    pub bedrooms: i32,
    pub bathrooms: Decimal,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<String>,
    pub status: String,
}

#[async_trait::async_trait]
pub trait UnitRepository {
    type Item;

    async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<Self::Item>, AppError>;
    async fn create(&self, organization_id: &str, input: UnitInput)
    -> Result<Self::Item, AppError>;
    async fn update(
        &self,
        organization_id: &str,
        unit_id: &str,
        input: UnitInput,
    ) -> Result<Self::Item, AppError>;
    async fn archive(&self, organization_id: &str, unit_id: &str) -> Result<(), AppError>;
}

pub struct UnitService<R> {
    repository: R,
}

impl<R: UnitRepository> UnitService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<R::Item>, AppError> {
        self.repository.list(organization_id, property_id).await
    }

    pub async fn create(
        &self,
        organization_id: &str,
        input: UnitInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .create(organization_id, validate(input)?)
            .await
    }

    pub async fn update(
        &self,
        organization_id: &str,
        unit_id: &str,
        input: UnitInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .update(organization_id, unit_id, validate(input)?)
            .await
    }

    pub async fn archive(&self, organization_id: &str, unit_id: &str) -> Result<(), AppError> {
        self.repository.archive(organization_id, unit_id).await
    }
}

fn validate(mut input: UnitInput) -> Result<UnitInput, AppError> {
    input.property_id = input.property_id.trim().to_string();
    input.unit_number = input.unit_number.trim().to_string();
    input.currency_code = input.currency_code.map(|value| value.trim().to_uppercase());
    if input.property_id.is_empty() || input.unit_number.is_empty() {
        return Err(AppError::Validation(
            "Property and unit number are required.".to_string(),
        ));
    }
    if !matches!(
        input.unit_type.as_str(),
        "apartment" | "office" | "retail" | "warehouse" | "parking" | "other"
    ) {
        return Err(AppError::Validation("Unsupported unit type.".to_string()));
    }
    if !matches!(
        input.status.as_str(),
        "available" | "maintenance" | "inactive"
    ) {
        return Err(AppError::Validation("Unsupported unit status.".to_string()));
    }
    if input.bedrooms < 0
        || input.bathrooms < Decimal::ZERO
        || input.market_rent_minor.is_some_and(|rent| rent < 0)
    {
        return Err(AppError::Validation(
            "Bedrooms, bathrooms, and rent cannot be negative.".to_string(),
        ));
    }
    if input
        .area_square_meters
        .is_some_and(|area| area <= Decimal::ZERO)
    {
        return Err(AppError::Validation(
            "Unit area must be greater than zero.".to_string(),
        ));
    }
    match (input.market_rent_minor, input.currency_code.as_ref()) {
        (None, None) => {}
        (Some(_), Some(code))
            if code.len() == 3 && code.bytes().all(|byte| byte.is_ascii_alphabetic()) => {}
        _ => {
            return Err(AppError::Validation(
                "A three-letter currency is required with the rent amount.".to_string(),
            ));
        }
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_a_currency_when_rent_is_set() {
        let error = validate(UnitInput {
            property_id: "property-1".to_string(),
            unit_number: "1A".to_string(),
            unit_type: "apartment".to_string(),
            floor_number: None,
            area_square_meters: None,
            bedrooms: 1,
            bathrooms: Decimal::ZERO,
            market_rent_minor: Some(100_000),
            currency_code: None,
            status: "available".to_string(),
        })
        .expect_err("rent without currency");

        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn rejects_non_positive_area() {
        let error = validate(UnitInput {
            property_id: "property-1".to_string(),
            unit_number: "1A".to_string(),
            unit_type: "apartment".to_string(),
            floor_number: None,
            area_square_meters: Some(Decimal::ZERO),
            bedrooms: 1,
            bathrooms: Decimal::ZERO,
            market_rent_minor: None,
            currency_code: None,
            status: "available".to_string(),
        })
        .expect_err("zero area");

        assert!(matches!(error, AppError::Validation(_)));
    }
}
