use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
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

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

id_type!(OrganizationId);
id_type!(UserId);
id_type!(TenantId);
id_type!(UnitId);
id_type!(PaymentId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorContext {
    user_id: UserId,
    organization_id: OrganizationId,
    role: AccessRole,
}

impl ActorContext {
    pub fn new(user_id: UserId, organization_id: OrganizationId, role: AccessRole) -> Self {
        Self {
            user_id,
            organization_id,
            role,
        }
    }

    pub fn user_id(self) -> UserId {
        self.user_id
    }

    pub fn organization_id(self) -> OrganizationId {
        self.organization_id
    }

    pub fn role(self) -> AccessRole {
        self.role
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessRole {
    Admin,
    Staff,
    Readonly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: UserId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub email: String,
    pub role: AccessRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub legal_name: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

impl AccessRole {
    pub fn can_manage_users(self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_write_business_data(self) -> bool {
        !matches!(self, Self::Readonly)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Staff => "staff",
            Self::Readonly => "readonly",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "staff" | "manager" => Some(Self::Staff),
            "readonly" => Some(Self::Readonly),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_permissions_are_explicit() {
        assert!(AccessRole::Admin.can_manage_users());
        assert!(AccessRole::Staff.can_write_business_data());
        assert!(!AccessRole::Readonly.can_write_business_data());
        assert!(!AccessRole::Staff.can_manage_users());
    }

    #[test]
    fn identifiers_round_trip_through_strings() {
        let id = OrganizationId::new();
        assert_eq!(OrganizationId::parse(&id.to_string()), Ok(id));
    }
}
