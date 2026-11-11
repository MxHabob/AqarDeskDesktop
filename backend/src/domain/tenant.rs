use super::{DomainError, OrganizationId, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub organization_id: OrganizationId,
    pub input: TenantInput,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn create(
        id: TenantId,
        organization_id: OrganizationId,
        mut input: TenantInput,
    ) -> Result<Self, DomainError> {
        input.full_name = input.full_name.trim().to_string();
        input.email = clean_optional(input.email.map(|email| email.to_ascii_lowercase()));
        input.phone = clean_optional(input.phone);
        input.mailing_address = clean_optional(input.mailing_address);
        input.notes = clean_optional(input.notes);

        if input.full_name.is_empty() || input.full_name.len() > 160 {
            return Err(DomainError::Validation(
                "Tenant name is required and must be at most 160 characters.",
            ));
        }
        if input.email.as_ref().is_some_and(|email| {
            email.len() > 254
                || !email.contains('@')
                || email.bytes().any(|byte| byte.is_ascii_whitespace())
        }) {
            return Err(DomainError::Validation("Tenant email address is invalid."));
        }
        if input.phone.as_ref().is_some_and(|phone| phone.len() > 40)
            || input
                .mailing_address
                .as_ref()
                .is_some_and(|address| address.len() > 500)
            || input.notes.as_ref().is_some_and(|notes| notes.len() > 2000)
        {
            return Err(DomainError::Validation(
                "Tenant contact details exceed the allowed length.",
            ));
        }
        if input.email.is_none() && input.phone.is_none() {
            return Err(DomainError::Validation(
                "At least one tenant contact method is required.",
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
        id: TenantId,
        organization_id: OrganizationId,
        input: TenantInput,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let mut tenant = Self::create(id, organization_id, input)?;
        tenant.created_at = created_at;
        tenant.updated_at = updated_at;
        Ok(tenant)
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
    fn tenant_requires_contact_and_normalizes_email() {
        let input = TenantInput {
            full_name: " Tenant One ".to_string(),
            email: Some(" USER@example.com ".to_string()),
            phone: None,
            mailing_address: Some(" ".to_string()),
            notes: None,
        };
        let tenant =
            Tenant::create(TenantId::new(), OrganizationId::new(), input).expect("valid tenant");

        assert_eq!(tenant.input.full_name, "Tenant One");
        assert_eq!(tenant.input.email.as_deref(), Some("user@example.com"));
        assert_eq!(tenant.input.mailing_address, None);
    }

    #[test]
    fn tenant_rejects_missing_contact_details() {
        let input = TenantInput {
            full_name: "Tenant One".to_string(),
            email: None,
            phone: Some(" ".to_string()),
            mailing_address: None,
            notes: None,
        };
        assert!(Tenant::create(TenantId::new(), OrganizationId::new(), input).is_err());
    }
}
