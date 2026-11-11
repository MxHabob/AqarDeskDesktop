mod authorization;
mod errors;
mod identity_management;
mod identity_port;
mod payment_ledger;
mod property_catalog;
mod property_port;
mod tenant_directory;
mod tenant_port;
mod unit_catalog;
mod unit_port;

pub use errors::ApplicationError;
pub use identity_management::IdentityService;
pub use identity_port::{IdentityRepository, NewUser, OrganizationBootstrap};
pub use payment_ledger::{PaymentLedger, PaymentRepository};
pub use property_catalog::PropertyCatalog;
pub use property_port::PropertyRepository;
pub use tenant_directory::TenantDirectory;
pub use tenant_port::TenantRepository;
pub use unit_catalog::UnitCatalog;
pub use unit_port::UnitRepository;

pub(crate) use authorization::require_write;
