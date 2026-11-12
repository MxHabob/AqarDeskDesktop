use super::{DomainError, OrganizationId};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertyId(Uuid);

impl PropertyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(value: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(value).map(Self)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PropertyId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PropertyId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Residential,
    Commercial,
    MixedUse,
    Land,
    Other,
    Building,
    HouseVilla,
    Investment,
    Warehouse,
    Yard,
    Apartment,
}

impl PropertyType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Residential => "residential",
            Self::Commercial => "commercial",
            Self::MixedUse => "mixed_use",
            Self::Land => "land",
            Self::Other => "other",
            Self::Building => "building",
            Self::HouseVilla => "house_villa",
            Self::Investment => "investment",
            Self::Warehouse => "warehouse",
            Self::Yard => "yard",
            Self::Apartment => "apartment",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "residential" => Ok(Self::Residential),
            "commercial" => Ok(Self::Commercial),
            "mixed_use" => Ok(Self::MixedUse),
            "land" => Ok(Self::Land),
            "other" => Ok(Self::Other),
            "building" => Ok(Self::Building),
            "house_villa" => Ok(Self::HouseVilla),
            "investment" => Ok(Self::Investment),
            "warehouse" => Ok(Self::Warehouse),
            "yard" => Ok(Self::Yard),
            "apartment" => Ok(Self::Apartment),
            _ => Err(DomainError::Validation("Unsupported property type.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyStatus {
    Active,
    Inactive,
    Archived,
}

impl PropertyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "archived" => Ok(Self::Archived),
            _ => Err(DomainError::Validation("Unsupported property status.")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyOwnerInput {
    pub id: Option<String>,
    pub name: String,
    pub phone_number: String,
    pub phone_label: Option<String>,
    pub is_whatsapp: bool,
    pub ownership_share_percent: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyInput {
    pub code: String,
    pub name: String,
    pub property_type: PropertyType,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub district: Option<String>,
    pub neighborhood: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub lot_area_square_meters: Option<Decimal>,
    pub building_area_square_meters: Option<Decimal>,
    pub frontage_meters: Option<Decimal>,
    pub depth_meters: Option<Decimal>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    #[serde(default)]
    pub owners: Vec<PropertyOwnerInput>,
    pub status: PropertyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Property {
    pub id: PropertyId,
    pub organization_id: OrganizationId,
    pub input: PropertyInput,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Property {
    pub fn create(
        id: PropertyId,
        organization_id: OrganizationId,
        mut input: PropertyInput,
    ) -> Result<Self, DomainError> {
        input.code = input.code.trim().to_ascii_uppercase();
        input.name = input.name.trim().to_string();
        input.address_line_1 = input.address_line_1.trim().to_string();
        input.city = input.city.trim().to_string();
        input.country_code = input.country_code.trim().to_ascii_uppercase();
        input.description = clean_optional(input.description);
        input.address_line_2 = clean_optional(input.address_line_2);
        input.region = clean_optional(input.region);
        input.district = clean_optional(input.district);
        input.neighborhood = clean_optional(input.neighborhood);
        input.postal_code = clean_optional(input.postal_code);

        if input.code.is_empty() || input.code.len() > 32 {
            return Err(DomainError::Validation(
                "Property code is required and must be at most 32 characters.",
            ));
        }
        if input.name.is_empty() || input.name.len() > 160 {
            return Err(DomainError::Validation(
                "Property name is required and must be at most 160 characters.",
            ));
        }
        if input.address_line_1.is_empty() || input.city.is_empty() {
            return Err(DomainError::Validation(
                "Property address and city are required.",
            ));
        }
        if input.country_code.len() != 2
            || !input
                .country_code
                .bytes()
                .all(|byte| byte.is_ascii_uppercase())
        {
            return Err(DomainError::Validation(
                "Country code must contain two ASCII letters.",
            ));
        }
        if input.owners.len() > 100 {
            return Err(DomainError::Validation(
                "A property cannot have more than 100 owners.",
            ));
        }

        for area in [
            input.lot_area_square_meters,
            input.building_area_square_meters,
            input.frontage_meters,
            input.depth_meters,
        ]
        .into_iter()
        .flatten()
        {
            if area < Decimal::ZERO {
                return Err(DomainError::Validation(
                    "Property measurements cannot be negative.",
                ));
            }
        }
        if input
            .latitude
            .is_some_and(|latitude| !(-Decimal::from(90)..=Decimal::from(90)).contains(&latitude))
            || input.longitude.is_some_and(|longitude| {
                !(-Decimal::from(180)..=Decimal::from(180)).contains(&longitude)
            })
        {
            return Err(DomainError::Validation(
                "Property coordinates are outside their valid range.",
            ));
        }
        let mut ownership_total = Decimal::ZERO;
        let mut owner_ids = HashSet::new();
        for owner in &mut input.owners {
            owner.name = owner.name.trim().to_string();
            owner.phone_number = owner.phone_number.trim().to_string();
            owner.phone_label = clean_optional(owner.phone_label.take());
            if owner.name.is_empty()
                || owner.name.len() > 160
                || owner.phone_number.is_empty()
                || owner.phone_number.len() > 64
            {
                return Err(DomainError::Validation(
                    "Each property owner needs a name and phone number.",
                ));
            }
            if owner
                .id
                .as_ref()
                .is_some_and(|id| !owner_ids.insert(id.clone()))
            {
                return Err(DomainError::Validation(
                    "A property owner cannot be listed more than once.",
                ));
            }
            if let Some(share) = owner.ownership_share_percent {
                if !(Decimal::ZERO..=Decimal::from(100)).contains(&share) || share.scale() > 2 {
                    return Err(DomainError::Validation(
                        "Ownership share must be between 0 and 100 percent.",
                    ));
                }
                ownership_total += share;
            }
        }
        if ownership_total > Decimal::from(100) {
            return Err(DomainError::Validation(
                "Total ownership share cannot exceed 100 percent.",
            ));
        }

        Ok(Self {
            id,
            organization_id,
            input,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn restore(
        id: PropertyId,
        organization_id: OrganizationId,
        input: PropertyInput,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let mut property = Self::create(id, organization_id, input)?;
        property.created_at = created_at;
        property.updated_at = updated_at;
        Ok(property)
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

    fn property_input() -> PropertyInput {
        PropertyInput {
            code: " apt-01 ".to_string(),
            name: " Apartment ".to_string(),
            property_type: PropertyType::Residential,
            description: Some(" ".to_string()),
            address_line_1: " Main Street ".to_string(),
            address_line_2: None,
            city: " City ".to_string(),
            region: None,
            district: None,
            neighborhood: None,
            postal_code: None,
            country_code: " us ".to_string(),
            lot_area_square_meters: None,
            building_area_square_meters: None,
            frontage_meters: None,
            depth_meters: None,
            latitude: None,
            longitude: None,
            owners: Vec::new(),
            status: PropertyStatus::Active,
        }
    }

    #[test]
    fn property_creation_normalizes_user_input() {
        let property = Property::create(PropertyId::new(), OrganizationId::new(), property_input())
            .expect("valid property");

        assert_eq!(property.input.code, "APT-01");
        assert_eq!(property.input.name, "Apartment");
        assert_eq!(property.input.description, None);
        assert_eq!(property.input.country_code, "US");
    }

    #[test]
    fn property_creation_rejects_invalid_country_codes() {
        let mut input = property_input();
        input.country_code = "USA".to_string();

        assert!(matches!(
            Property::create(PropertyId::new(), OrganizationId::new(), input),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn property_creation_rejects_invalid_dimensions_and_ownership_totals() {
        let mut invalid_area = property_input();
        invalid_area.lot_area_square_meters = Some(Decimal::new(-1, 0));
        assert!(matches!(
            Property::create(PropertyId::new(), OrganizationId::new(), invalid_area),
            Err(DomainError::Validation(_))
        ));

        let mut invalid_owners = property_input();
        invalid_owners.owners = vec![
            PropertyOwnerInput {
                id: None,
                name: "First owner".to_string(),
                phone_number: "0500000001".to_string(),
                phone_label: None,
                is_whatsapp: false,
                ownership_share_percent: Some(Decimal::from(60)),
            },
            PropertyOwnerInput {
                id: None,
                name: "Second owner".to_string(),
                phone_number: "0500000002".to_string(),
                phone_label: None,
                is_whatsapp: false,
                ownership_share_percent: Some(Decimal::from(50)),
            },
        ];
        assert!(matches!(
            Property::create(PropertyId::new(), OrganizationId::new(), invalid_owners),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn property_creation_rejects_coordinates_outside_geographic_ranges() {
        let mut input = property_input();
        input.latitude = Some(Decimal::from(91));
        assert!(matches!(
            Property::create(PropertyId::new(), OrganizationId::new(), input),
            Err(DomainError::Validation(_))
        ));
    }
}
