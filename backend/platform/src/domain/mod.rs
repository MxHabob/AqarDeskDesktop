mod identity;
mod money;
mod payment;
mod property;
mod tenant;
mod unit;

pub use identity::{
    AccessRole, ActorContext, Organization, OrganizationId, PaymentId, TenantId, UnitId,
    UserAccount, UserId,
};
pub use money::{CurrencyCode, Money};
pub use payment::{Payment, PaymentInput, PaymentMethod, PaymentStatus};
pub use property::{Property, PropertyId, PropertyInput, PropertyStatus, PropertyType};
pub use tenant::{Tenant, TenantInput};
pub use unit::{PropertyUnit, PropertyUnitInput, UnitStatus, UnitType};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("{0}")]
    Validation(&'static str),
}
