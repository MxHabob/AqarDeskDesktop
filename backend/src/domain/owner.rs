use super::{DomainError, OrganizationId, OwnerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerInput {
    pub name: String,
    pub phone_number: String,
    pub phone_label: Option<String>,
    pub is_whatsapp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owner {
    pub id: OwnerId,
    pub organization_id: OrganizationId,
    pub input: OwnerInput,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Owner {
    pub fn create(
        id: OwnerId,
        organization_id: OrganizationId,
        mut input: OwnerInput,
    ) -> Result<Self, DomainError> {
        input.name = input.name.trim().to_string();
        input.phone_number = input.phone_number.trim().to_string();
        input.phone_label = clean_optional(input.phone_label);

        if input.name.is_empty() || input.name.len() > 160 {
            return Err(DomainError::Validation(
                "Owner name is required and must be at most 160 characters.",
            ));
        }
        if input.phone_number.is_empty() || input.phone_number.len() > 64 {
            return Err(DomainError::Validation(
                "Owner phone number is required and must be at most 64 characters.",
            ));
        }
        if input
            .phone_label
            .as_ref()
            .is_some_and(|label| label.len() > 40)
        {
            return Err(DomainError::Validation(
                "Owner phone label must be at most 40 characters.",
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
        id: OwnerId,
        organization_id: OrganizationId,
        input: OwnerInput,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let mut owner = Self::create(id, organization_id, input)?;
        owner.created_at = created_at;
        owner.updated_at = updated_at;
        Ok(owner)
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

    #[test]
    fn owner_creation_normalizes_contact_details() {
        let owner = Owner::create(
            OwnerId::new(),
            OrganizationId::new(),
            OwnerInput {
                name: " Owner One ".to_string(),
                phone_number: " 0500000000 ".to_string(),
                phone_label: Some(" Mobile ".to_string()),
                is_whatsapp: true,
            },
        )
        .expect("valid owner");

        assert_eq!(owner.input.name, "Owner One");
        assert_eq!(owner.input.phone_number, "0500000000");
        assert_eq!(owner.input.phone_label.as_deref(), Some("Mobile"));
    }

    #[test]
    fn owner_requires_a_name_and_phone() {
        let result = Owner::create(
            OwnerId::new(),
            OrganizationId::new(),
            OwnerInput {
                name: " ".to_string(),
                phone_number: " ".to_string(),
                phone_label: None,
                is_whatsapp: false,
            },
        );
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }
}
