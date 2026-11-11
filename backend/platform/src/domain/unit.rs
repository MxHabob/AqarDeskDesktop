use super::{CurrencyCode, DomainError, OrganizationId, PropertyId, UnitId};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    Apartment,
    Office,
    Retail,
    Warehouse,
    Parking,
    Other,
}

impl UnitType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apartment => "apartment",
            Self::Office => "office",
            Self::Retail => "retail",
            Self::Warehouse => "warehouse",
            Self::Parking => "parking",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "apartment" => Ok(Self::Apartment),
            "office" => Ok(Self::Office),
            "retail" => Ok(Self::Retail),
            "warehouse" => Ok(Self::Warehouse),
            "parking" => Ok(Self::Parking),
            "other" => Ok(Self::Other),
            _ => Err(DomainError::Validation("Unsupported unit type.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitStatus {
    Available,
    Maintenance,
    Inactive,
}

impl UnitStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Maintenance => "maintenance",
            Self::Inactive => "inactive",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "available" => Ok(Self::Available),
            "maintenance" => Ok(Self::Maintenance),
            "inactive" => Ok(Self::Inactive),
            _ => Err(DomainError::Validation("Unsupported unit status.")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyUnitInput {
    pub property_id: PropertyId,
    pub unit_number: String,
    pub unit_type: UnitType,
    pub floor_number: Option<i32>,
    pub area_square_meters: Option<Decimal>,
    pub bedrooms: u32,
    pub bathrooms: Decimal,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<CurrencyCode>,
    pub status: UnitStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyUnit {
    pub id: UnitId,
    pub organization_id: OrganizationId,
    pub input: PropertyUnitInput,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PropertyUnit {
    pub fn create(
        id: UnitId,
        organization_id: OrganizationId,
        mut input: PropertyUnitInput,
    ) -> Result<Self, DomainError> {
        input.unit_number = input.unit_number.trim().to_string();
        if input.unit_number.is_empty() || input.unit_number.len() > 64 {
            return Err(DomainError::Validation(
                "Unit number is required and must be at most 64 characters.",
            ));
        }
        if input
            .area_square_meters
            .is_some_and(|area| area <= Decimal::ZERO)
        {
            return Err(DomainError::Validation(
                "Unit area must be greater than zero.",
            ));
        }
        if input.bedrooms > 100 || input.bathrooms < Decimal::ZERO {
            return Err(DomainError::Validation(
                "Unit bedroom and bathroom counts are invalid.",
            ));
        }
        if input.market_rent_minor.is_some_and(|amount| amount < 0)
            || input.market_rent_minor.is_some() != input.currency_code.is_some()
        {
            return Err(DomainError::Validation(
                "A non-negative unit rent and its currency must be supplied together.",
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id,
            organization_id,
            input,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn restore(
        id: UnitId,
        organization_id: OrganizationId,
        input: PropertyUnitInput,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let mut unit = Self::create(id, organization_id, input)?;
        unit.created_at = created_at;
        unit.updated_at = updated_at;
        Ok(unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PropertyUnitInput {
        PropertyUnitInput {
            property_id: PropertyId::new(),
            unit_number: " A-1 ".to_string(),
            unit_type: UnitType::Apartment,
            floor_number: Some(1),
            area_square_meters: Some(Decimal::new(850, 1)),
            bedrooms: 2,
            bathrooms: Decimal::ONE,
            market_rent_minor: None,
            currency_code: None,
            status: UnitStatus::Available,
        }
    }

    #[test]
    fn unit_requires_rent_and_currency_as_a_pair() {
        let mut value = input();
        value.market_rent_minor = Some(100_000);
        assert!(PropertyUnit::create(UnitId::new(), OrganizationId::new(), value).is_err());
    }

    #[test]
    fn unit_normalizes_number_and_accepts_positive_area() {
        let unit = PropertyUnit::create(UnitId::new(), OrganizationId::new(), input())
            .expect("valid unit");
        assert_eq!(unit.input.unit_number, "A-1");
    }
}
