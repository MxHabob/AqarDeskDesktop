use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AqarDesk API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Organization-scoped property management API.",
        license(name = "MIT")
    ),
    paths(
        super::health,
        super::readiness,
        super::get_current_organization,
        super::update_current_organization,
        super::get_my_profile,
        super::update_my_profile,
        super::list_audit_events,
        super::list_contract_templates,
        super::create_contract_template,
        super::update_contract_template,
        super::set_contract_template_state,
        super::dashboard_summary,
        super::list_properties,
        super::create_property,
        super::update_property,
        super::archive_property,
        super::list_units,
        super::create_unit,
        super::update_unit,
        super::archive_unit,
        super::list_tenants,
        super::create_tenant,
        super::update_tenant,
        super::archive_tenant
    ),
    components(schemas(
        super::HealthPayload,
        super::ReadinessPayload,
        super::dto::OrganizationSettings,
        super::dto::OrganizationSettingsInput,
        super::dto::ProfileSettings,
        super::dto::ProfileSettingsInput,
        super::dto::AuditEntry,
        super::dto::AuditQuery,
        super::dto::ContractTemplate,
        super::dto::ContractTemplateVersion,
        super::dto::ContractTemplateInput,
        super::dto::ContractTemplateStateInput,
        super::dto::Property,
        super::dto::PropertyInput,
        super::dto::PropertyType,
        super::dto::PropertyStatus,
        super::dto::PropertyUnit,
        super::dto::PropertyUnitInput,
        super::dto::UnitType,
        super::dto::UnitStatus,
        super::dto::Tenant,
        super::dto::TenantInput,
        super::dto::DashboardSummary,
        super::dto::StatusCount,
        super::dto::MonthlyPayment,
        super::dto::OpenChargesByCurrency,
        super::dto::OperationResult,
        super::dto::ErrorResponse
    )),
    modifiers(&BearerAuth),
    tags(
        (name = "System", description = "API health and readiness"),
        (name = "Organization", description = "Organization information"),
        (name = "Settings", description = "Organization, profile, audit, and contract template settings"),
        (name = "Dashboard", description = "Organization dashboard metrics"),
        (name = "Properties", description = "Property management"),
        (name = "Units", description = "Property unit management"),
        (name = "Tenants", description = "Tenant management")
    )
)]
pub struct ApiDoc;

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
    }
}
