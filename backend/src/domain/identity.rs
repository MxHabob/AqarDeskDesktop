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
id_type!(OwnerId);
id_type!(UnitId);
id_type!(PaymentId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorContext {
    user_id: UserId,
    organization_id: OrganizationId,
    role: AccessRole,
    permissions: Option<u32>,
}

impl ActorContext {
    pub fn new(user_id: UserId, organization_id: OrganizationId, role: AccessRole) -> Self {
        Self {
            user_id,
            organization_id,
            role,
            permissions: None,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessRole {
    Admin,
    Staff,
    Readonly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ViewProperties,
    CreateProperties,
    UpdateProperties,
    ArchiveProperties,
    ViewOwners,
    CreateOwners,
    UpdateOwners,
    ViewUnits,
    CreateUnits,
    UpdateUnits,
    ArchiveUnits,
    ViewTenants,
    CreateTenants,
    UpdateTenants,
    ArchiveTenants,
    ViewPayments,
    RecordPayments,
    VoidPayments,
    ViewContracts,
    CreateContracts,
    ManageAdministration,
}

impl Permission {
    pub const BUSINESS: [Self; 20] = [
        Self::ViewProperties,
        Self::CreateProperties,
        Self::UpdateProperties,
        Self::ArchiveProperties,
        Self::ViewOwners,
        Self::CreateOwners,
        Self::UpdateOwners,
        Self::ViewUnits,
        Self::CreateUnits,
        Self::UpdateUnits,
        Self::ArchiveUnits,
        Self::ViewTenants,
        Self::CreateTenants,
        Self::UpdateTenants,
        Self::ArchiveTenants,
        Self::ViewPayments,
        Self::RecordPayments,
        Self::VoidPayments,
        Self::ViewContracts,
        Self::CreateContracts,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ViewProperties => "view_properties",
            Self::CreateProperties => "create_properties",
            Self::UpdateProperties => "update_properties",
            Self::ArchiveProperties => "archive_properties",
            Self::ViewOwners => "view_owners",
            Self::CreateOwners => "create_owners",
            Self::UpdateOwners => "update_owners",
            Self::ViewUnits => "view_units",
            Self::CreateUnits => "create_units",
            Self::UpdateUnits => "update_units",
            Self::ArchiveUnits => "archive_units",
            Self::ViewTenants => "view_tenants",
            Self::CreateTenants => "create_tenants",
            Self::UpdateTenants => "update_tenants",
            Self::ArchiveTenants => "archive_tenants",
            Self::ViewPayments => "view_payments",
            Self::RecordPayments => "record_payments",
            Self::VoidPayments => "void_payments",
            Self::ViewContracts => "view_contracts",
            Self::CreateContracts => "create_contracts",
            Self::ManageAdministration => "manage_administration",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "view_properties" => Some(Self::ViewProperties),
            "create_properties" => Some(Self::CreateProperties),
            "update_properties" => Some(Self::UpdateProperties),
            "archive_properties" => Some(Self::ArchiveProperties),
            "view_owners" => Some(Self::ViewOwners),
            "create_owners" => Some(Self::CreateOwners),
            "update_owners" => Some(Self::UpdateOwners),
            "view_units" => Some(Self::ViewUnits),
            "create_units" => Some(Self::CreateUnits),
            "update_units" => Some(Self::UpdateUnits),
            "archive_units" => Some(Self::ArchiveUnits),
            "view_tenants" => Some(Self::ViewTenants),
            "create_tenants" => Some(Self::CreateTenants),
            "update_tenants" => Some(Self::UpdateTenants),
            "archive_tenants" => Some(Self::ArchiveTenants),
            "view_payments" => Some(Self::ViewPayments),
            "record_payments" => Some(Self::RecordPayments),
            "void_payments" => Some(Self::VoidPayments),
            "view_contracts" => Some(Self::ViewContracts),
            "create_contracts" => Some(Self::CreateContracts),
            "manage_administration" => Some(Self::ManageAdministration),
            _ => None,
        }
    }

    fn bit(self) -> u32 {
        match self {
            Self::ViewProperties => 1 << 0,
            Self::CreateProperties => 1 << 1,
            Self::UpdateProperties => 1 << 2,
            Self::ArchiveProperties => 1 << 3,
            Self::ViewOwners => 1 << 4,
            Self::CreateOwners => 1 << 5,
            Self::UpdateOwners => 1 << 6,
            Self::ViewUnits => 1 << 7,
            Self::CreateUnits => 1 << 8,
            Self::UpdateUnits => 1 << 9,
            Self::ArchiveUnits => 1 << 10,
            Self::ViewTenants => 1 << 11,
            Self::CreateTenants => 1 << 12,
            Self::UpdateTenants => 1 << 13,
            Self::ArchiveTenants => 1 << 14,
            Self::ViewPayments => 1 << 15,
            Self::RecordPayments => 1 << 16,
            Self::VoidPayments => 1 << 17,
            Self::ManageAdministration => 1 << 18,
            Self::ViewContracts => 1 << 19,
            Self::CreateContracts => 1 << 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RolePermissionSet {
    pub role_id: String,
    pub name: String,
    pub is_system: bool,
    pub system_role: Option<AccessRole>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: UserId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub email: String,
    pub role: AccessRole,
    pub role_id: String,
    pub role_name: Option<String>,
    pub permissions: Vec<Permission>,
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
    pub fn default_permissions(self) -> Vec<Permission> {
        match self {
            Self::Admin => Permission::BUSINESS
                .into_iter()
                .chain([Permission::ManageAdministration])
                .collect(),
            Self::Staff => Permission::BUSINESS.to_vec(),
            Self::Readonly => Vec::new(),
        }
    }

    pub fn can_manage_users(self) -> bool {
        self.has_permission(Permission::ManageAdministration)
    }

    pub fn can_write_business_data(self) -> bool {
        Permission::BUSINESS
            .into_iter()
            .any(|permission| self.has_permission(permission))
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        match self {
            Self::Admin => true,
            Self::Staff => permission != Permission::ManageAdministration,
            Self::Readonly => false,
        }
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

impl ActorContext {
    pub fn with_permissions(
        user_id: UserId,
        organization_id: OrganizationId,
        role: AccessRole,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            user_id,
            organization_id,
            role,
            permissions: Some(
                permissions
                    .into_iter()
                    .fold(0, |mask, permission| mask | permission.bit()),
            ),
        }
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        if permission == Permission::ManageAdministration && self.role == AccessRole::Admin {
            return true;
        }
        self.permissions.map_or_else(
            || self.role.has_permission(permission),
            |permissions| permissions & permission.bit() != 0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_permissions_are_explicit() {
        assert!(AccessRole::Admin.has_permission(Permission::ManageAdministration));
        assert!(AccessRole::Staff.has_permission(Permission::ViewProperties));
        assert!(AccessRole::Staff.has_permission(Permission::UpdateOwners));
        assert!(!AccessRole::Staff.has_permission(Permission::ManageAdministration));
        assert!(!AccessRole::Readonly.has_permission(Permission::ViewProperties));
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
