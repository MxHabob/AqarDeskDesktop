use crate::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInput {
    pub code: String,
    pub name: String,
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub status: String,
}

#[async_trait::async_trait]
pub trait PropertyRepository {
    type Item;

    async fn list(&self, organization_id: &str) -> Result<Vec<Self::Item>, AppError>;
    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PropertyInput,
    ) -> Result<Self::Item, AppError>;
    async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
        input: PropertyInput,
    ) -> Result<Self::Item, AppError>;
    async fn archive(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
    ) -> Result<(), AppError>;
}

pub struct PropertyService<R> {
    repository: R,
}

impl<R: PropertyRepository> PropertyService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(&self, organization_id: &str) -> Result<Vec<R::Item>, AppError> {
        self.repository.list(organization_id).await
    }

    pub async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PropertyInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .create(organization_id, actor_id, validate(input)?)
            .await
    }

    pub async fn update(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
        input: PropertyInput,
    ) -> Result<R::Item, AppError> {
        self.repository
            .update(organization_id, actor_id, property_id, validate(input)?)
            .await
    }

    pub async fn archive(
        &self,
        organization_id: &str,
        actor_id: &str,
        property_id: &str,
    ) -> Result<(), AppError> {
        self.repository
            .archive(organization_id, actor_id, property_id)
            .await
    }
}

fn validate(mut input: PropertyInput) -> Result<PropertyInput, AppError> {
    input.code = input.code.trim().to_uppercase();
    input.name = input.name.trim().to_string();
    input.property_type = input.property_type.trim().to_string();
    input.address_line_1 = input.address_line_1.trim().to_string();
    input.city = input.city.trim().to_string();
    input.country_code = input.country_code.trim().to_uppercase();
    input.description = super::clean_optional(input.description);
    input.address_line_2 = super::clean_optional(input.address_line_2);
    input.region = super::clean_optional(input.region);
    input.postal_code = super::clean_optional(input.postal_code);
    if input.code.is_empty()
        || input.name.is_empty()
        || input.address_line_1.is_empty()
        || input.city.is_empty()
    {
        return Err(AppError::Validation(
            "Property code, name, address, and city are required.".to_string(),
        ));
    }
    if !matches!(
        input.property_type.as_str(),
        "residential" | "commercial" | "mixed_use" | "land" | "other"
    ) {
        return Err(AppError::Validation(
            "Unsupported property type.".to_string(),
        ));
    }
    if !matches!(input.status.as_str(), "active" | "inactive" | "archived") {
        return Err(AppError::Validation(
            "Unsupported property status.".to_string(),
        ));
    }
    if input.country_code.len() != 2
        || !input
            .country_code
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return Err(AppError::Validation(
            "Country code must contain two ASCII letters.".to_string(),
        ));
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_property_identity_and_optional_fields() {
        let input = validate(PropertyInput {
            code: "  apt-01 ".to_string(),
            name: " Apartment ".to_string(),
            property_type: "residential".to_string(),
            description: Some("  ".to_string()),
            address_line_1: "  Main Street ".to_string(),
            address_line_2: None,
            city: " City ".to_string(),
            region: None,
            postal_code: None,
            country_code: " us ".to_string(),
            status: "active".to_string(),
        })
        .expect("valid property");

        assert_eq!(input.code, "APT-01");
        assert_eq!(input.name, "Apartment");
        assert_eq!(input.description, None);
        assert_eq!(input.country_code, "US");
    }

    #[test]
    fn rejects_unsupported_property_types() {
        let error = validate(PropertyInput {
            code: "APT-01".to_string(),
            name: "Apartment".to_string(),
            property_type: "unsupported".to_string(),
            description: None,
            address_line_1: "Main Street".to_string(),
            address_line_2: None,
            city: "City".to_string(),
            region: None,
            postal_code: None,
            country_code: "US".to_string(),
            status: "active".to_string(),
        })
        .expect_err("unsupported property type");

        assert!(matches!(error, AppError::Validation(_)));
    }
}
