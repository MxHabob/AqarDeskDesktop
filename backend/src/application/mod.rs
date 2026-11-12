mod audit_log;
mod authorization;
mod contract_documents;
mod contract_templates;
mod currencies;
mod dashboard;
mod errors;
mod identity_management;
mod identity_port;
mod organization_settings;
mod owner_directory;
mod owner_port;
mod pagination;
mod payment_ledger;
mod profile;
mod property_catalog;
mod property_port;
mod tenant_directory;
mod tenant_port;
mod unit_catalog;
mod unit_port;

pub use audit_log::{AuditEntry, AuditLog, AuditLogRepository};
pub use contract_documents::{
    ContractDocument, ContractDocumentInput, ContractDocumentRepository, ContractDocumentService,
};
pub use contract_templates::{
    ContractTemplate, ContractTemplateInput, ContractTemplateRepository, ContractTemplateService,
    ContractTemplateVersion,
};
pub use currencies::{
    CurrencyCatalog, CurrencyInput, CurrencyRepository, CurrencySettings, NewCurrencyInput,
};
pub use dashboard::{
    CurrencyAmountTotal, DashboardRepository, DashboardService, DashboardSummary, MonthlyPayment,
    PaymentCurrencyTotal, PropertyDashboardSummary, StatusCount,
};
pub use errors::ApplicationError;
pub use identity_management::IdentityService;
pub use identity_port::{IdentityRepository, NewUser, OrganizationBootstrap, UserUpdate};
pub use organization_settings::{
    OrganizationSettings, OrganizationSettingsInput, OrganizationSettingsRepository,
    OrganizationSettingsService,
};
pub use owner_directory::OwnerDirectory;
pub use owner_port::OwnerRepository;
pub use pagination::{Page, PageRequest};
pub use payment_ledger::{PaymentContract, PaymentLedger, PaymentRepository};
pub use profile::{ProfileRepository, ProfileService, ProfileSettings, ProfileSettingsInput};
pub use property_catalog::PropertyCatalog;
pub use property_port::PropertyRepository;
pub use tenant_directory::TenantDirectory;
pub use tenant_port::TenantRepository;
pub use unit_catalog::UnitCatalog;
pub use unit_port::UnitRepository;

pub(crate) use authorization::{require_admin, require_permission};
