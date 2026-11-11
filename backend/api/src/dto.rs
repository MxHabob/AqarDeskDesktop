use aqardesk_app::services::payments::{Payment as AppPayment, PaymentInput as AppPaymentInput};
use aqardesk_infrastructure::models::{property, tenant, unit};
use sea_orm::prelude::{DateTimeUtc, Decimal};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(ToSchema)]
#[schema(rename_all = "snake_case")]
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

#[allow(dead_code)]
#[derive(ToSchema)]
#[schema(rename_all = "snake_case")]
pub enum PropertyStatus {
    Active,
    Inactive,
    Archived,
}

#[allow(dead_code)]
#[derive(ToSchema)]
#[schema(rename_all = "snake_case")]
pub enum UnitType {
    Apartment,
    Office,
    Retail,
    Warehouse,
    Parking,
    Other,
}

#[allow(dead_code)]
#[derive(ToSchema)]
#[schema(rename_all = "snake_case")]
pub enum UnitStatus {
    Available,
    Maintenance,
    Inactive,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PropertyInput {
    pub code: String,
    pub name: String,
    #[schema(value_type = PropertyType)]
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    #[schema(value_type = PropertyStatus)]
    pub status: String,
}

impl From<PropertyInput> for aqardesk_app::services::properties::PropertyInput {
    fn from(input: PropertyInput) -> Self {
        Self {
            code: input.code,
            name: input.name,
            property_type: input.property_type,
            description: input.description,
            address_line_1: input.address_line_1,
            address_line_2: input.address_line_2,
            city: input.city,
            region: input.region,
            postal_code: input.postal_code,
            country_code: input.country_code,
            status: input.status,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PropertyUnitInput {
    pub property_id: String,
    pub unit_number: String,
    #[schema(value_type = UnitType)]
    pub unit_type: String,
    pub floor_number: Option<i32>,
    #[schema(value_type = Option<f64>)]
    pub area_square_meters: Option<Decimal>,
    pub bedrooms: i32,
    #[schema(value_type = f64)]
    pub bathrooms: Decimal,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<String>,
    #[schema(value_type = UnitStatus)]
    pub status: String,
}

impl From<PropertyUnitInput> for aqardesk_app::services::units::UnitInput {
    fn from(input: PropertyUnitInput) -> Self {
        Self {
            property_id: input.property_id,
            unit_number: input.unit_number,
            unit_type: input.unit_type,
            floor_number: input.floor_number,
            area_square_meters: input.area_square_meters,
            bedrooms: input.bedrooms,
            bathrooms: input.bathrooms,
            market_rent_minor: input.market_rent_minor,
            currency_code: input.currency_code,
            status: input.status,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TenantInput {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
}

impl From<TenantInput> for aqardesk_app::services::tenants::TenantInput {
    fn from(input: TenantInput) -> Self {
        Self {
            full_name: input.full_name,
            email: input.email,
            phone: input.phone,
            mailing_address: input.mailing_address,
            notes: input.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    BankTransfer,
    Card,
    Cheque,
    Online,
    Other,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaymentInput {
    pub tenant_id: String,
    pub received_at: String,
    pub amount_minor: i64,
    pub currency_code: String,
    #[schema(value_type = PaymentMethod)]
    pub payment_method: String,
    pub external_reference: Option<String>,
    pub notes: Option<String>,
}

impl From<PaymentInput> for AppPaymentInput {
    fn from(input: PaymentInput) -> Self {
        Self {
            tenant_id: input.tenant_id,
            received_at: input.received_at,
            amount_minor: input.amount_minor,
            currency_code: input.currency_code,
            payment_method: input.payment_method,
            external_reference: input.external_reference,
            notes: input.notes,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Payment {
    pub id: String,
    pub tenant_id: String,
    pub tenant_name: String,
    pub received_at: String,
    pub amount_minor: i64,
    pub currency_code: String,
    #[schema(value_type = PaymentMethod)]
    pub payment_method: String,
    pub external_reference: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub voided_at: Option<String>,
}

impl From<AppPayment> for Payment {
    fn from(payment: AppPayment) -> Self {
        Self {
            id: payment.id,
            tenant_id: payment.tenant_id,
            tenant_name: payment.tenant_name,
            received_at: payment.received_at,
            amount_minor: payment.amount_minor,
            currency_code: payment.currency_code,
            payment_method: payment.payment_method,
            external_reference: payment.external_reference,
            status: payment.status,
            notes: payment.notes,
            voided_at: payment.voided_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Property {
    pub id: String,
    pub organization_id: String,
    pub code: String,
    pub name: String,
    #[schema(value_type = PropertyType)]
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    #[schema(value_type = PropertyStatus)]
    pub status: String,
    #[schema(value_type = String)]
    pub created_at: DateTimeUtc,
    #[schema(value_type = String)]
    pub updated_at: DateTimeUtc,
}

impl From<property::Model> for Property {
    fn from(model: property::Model) -> Self {
        Self {
            id: model.id,
            organization_id: model.organization_id,
            code: model.code,
            name: model.name,
            property_type: model.property_type,
            description: model.description,
            address_line_1: model.address_line_1,
            address_line_2: model.address_line_2,
            city: model.city,
            region: model.region,
            postal_code: model.postal_code,
            country_code: model.country_code,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PropertyUnit {
    pub id: String,
    pub organization_id: String,
    pub property_id: String,
    pub unit_number: String,
    #[schema(value_type = UnitType)]
    pub unit_type: String,
    pub floor_number: Option<i32>,
    #[schema(value_type = Option<f64>)]
    pub area_square_meters: Option<Decimal>,
    pub bedrooms: i32,
    #[schema(value_type = f64)]
    pub bathrooms: Decimal,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<String>,
    #[schema(value_type = UnitStatus)]
    pub status: String,
    #[schema(value_type = String)]
    pub created_at: DateTimeUtc,
    #[schema(value_type = String)]
    pub updated_at: DateTimeUtc,
}

impl From<unit::Model> for PropertyUnit {
    fn from(model: unit::Model) -> Self {
        Self {
            id: model.id,
            organization_id: model.organization_id,
            property_id: model.property_id,
            unit_number: model.unit_number,
            unit_type: model.unit_type,
            floor_number: model.floor_number,
            area_square_meters: model.area_square_meters,
            bedrooms: model.bedrooms,
            bathrooms: model.bathrooms,
            market_rent_minor: model.market_rent_minor,
            currency_code: model.currency_code,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Tenant {
    pub id: String,
    pub organization_id: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTimeUtc,
    #[schema(value_type = String)]
    pub updated_at: DateTimeUtc,
}

impl From<tenant::Model> for Tenant {
    fn from(model: tenant::Model) -> Self {
        Self {
            id: model.id,
            organization_id: model.organization_id,
            full_name: model.full_name,
            email: model.email,
            phone: model.phone,
            mailing_address: model.mailing_address,
            notes: model.notes,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OperationResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OrganizationSettingsInput {
    pub name: String,
    pub legal_name: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrencySettings {
    pub code: String,
    pub name: String,
    pub name_ar: Option<String>,
    pub symbol: String,
    pub symbol_position: String,
    pub decimal_digits: i16,
    pub is_active: bool,
    pub is_enabled_for_organization: bool,
}

impl From<aqardesk_app::services::settings::CurrencySettings> for CurrencySettings {
    fn from(value: aqardesk_app::services::settings::CurrencySettings) -> Self {
        Self {
            code: value.code,
            name: value.name,
            name_ar: value.name_ar,
            symbol: value.symbol,
            symbol_position: value.symbol_position,
            decimal_digits: value.decimal_digits,
            is_active: value.is_active,
            is_enabled_for_organization: value.is_enabled_for_organization,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OrganizationCurrencyStateInput {
    pub is_enabled: bool,
}

impl From<OrganizationSettingsInput>
    for aqardesk_app::services::settings::OrganizationSettingsInput
{
    fn from(input: OrganizationSettingsInput) -> Self {
        Self {
            name: input.name,
            legal_name: input.legal_name,
            default_currency: input.default_currency,
            timezone: input.timezone,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationSettings {
    pub id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub default_currency: String,
    pub timezone: String,
}

impl From<aqardesk_app::services::settings::OrganizationSettings> for OrganizationSettings {
    fn from(value: aqardesk_app::services::settings::OrganizationSettings) -> Self {
        Self {
            id: value.id,
            name: value.name,
            legal_name: value.legal_name,
            default_currency: value.default_currency,
            timezone: value.timezone,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProfileSettingsInput {
    pub name: String,
    pub email: String,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

impl From<ProfileSettingsInput> for aqardesk_app::services::settings::ProfileSettingsInput {
    fn from(input: ProfileSettingsInput) -> Self {
        Self {
            name: input.name,
            email: input.email,
            current_password: input.current_password,
            new_password: input.new_password,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileSettings {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
}

impl From<aqardesk_app::services::settings::ProfileSettings> for ProfileSettings {
    fn from(value: aqardesk_app::services::settings::ProfileSettings) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuditQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditEntry {
    pub id: String,
    pub actor_user_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details_json: Option<String>,
    pub created_at: String,
}

impl From<aqardesk_app::services::settings::AuditEntry> for AuditEntry {
    fn from(value: aqardesk_app::services::settings::AuditEntry) -> Self {
        Self {
            id: value.id,
            actor_user_id: value.actor_user_id,
            actor_name: value.actor_name,
            action: value.action,
            entity_type: value.entity_type,
            entity_id: value.entity_id,
            details_json: value.details_json,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContractTemplateInput {
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub lease_type: String,
    pub locale: String,
    pub content_format: String,
    pub content_body: String,
    pub placeholders_json: Option<String>,
    pub change_note: Option<String>,
}

impl From<ContractTemplateInput> for aqardesk_app::services::settings::ContractTemplateInput {
    fn from(input: ContractTemplateInput) -> Self {
        Self {
            template_key: input.template_key,
            name: input.name,
            description: input.description,
            lease_type: input.lease_type,
            locale: input.locale,
            content_format: input.content_format,
            content_body: input.content_body,
            placeholders_json: input.placeholders_json,
            change_note: input.change_note,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContractTemplateStateInput {
    pub is_active: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContractTemplateVersion {
    pub id: String,
    pub version_number: i32,
    pub content_format: String,
    pub content_body: Option<String>,
    pub placeholders_json: String,
    pub content_sha256: String,
    pub change_note: Option<String>,
    pub created_at: String,
}

impl From<aqardesk_app::services::settings::ContractTemplateVersion> for ContractTemplateVersion {
    fn from(value: aqardesk_app::services::settings::ContractTemplateVersion) -> Self {
        Self {
            id: value.id,
            version_number: value.version_number,
            content_format: value.content_format,
            content_body: value.content_body,
            placeholders_json: value.placeholders_json,
            content_sha256: value.content_sha256,
            change_note: value.change_note,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContractTemplate {
    pub id: String,
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub lease_type: String,
    pub locale: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub versions: Vec<ContractTemplateVersion>,
}

impl From<aqardesk_app::services::settings::ContractTemplate> for ContractTemplate {
    fn from(value: aqardesk_app::services::settings::ContractTemplate) -> Self {
        Self {
            id: value.id,
            template_key: value.template_key,
            name: value.name,
            description: value.description,
            lease_type: value.lease_type,
            locale: value.locale,
            is_active: value.is_active,
            created_at: value.created_at,
            updated_at: value.updated_at,
            versions: value.versions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct StatusCount {
    pub status: String,
    pub count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct MonthlyPayment {
    pub month: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub payment_count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct OpenChargesByCurrency {
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub amount_minor: i64,
    pub charge_count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct DashboardSummary {
    pub active_properties_count: u64,
    pub units_count: u64,
    pub available_units_count: u64,
    pub tenants_count: u64,
    pub active_leases_count: u64,
    pub open_charges_count: u64,
    pub posted_payments_count: u64,
    pub contract_documents_count: u64,
    pub expiring_leases_count: u64,
    pub payment_months: Vec<String>,
    pub monthly_payments: Vec<MonthlyPayment>,
    pub unit_status_counts: Vec<StatusCount>,
    pub lease_status_counts: Vec<StatusCount>,
    pub open_charges_by_currency: Vec<OpenChargesByCurrency>,
}

impl From<aqardesk_infrastructure::dashboard::StatusCount> for StatusCount {
    fn from(value: aqardesk_infrastructure::dashboard::StatusCount) -> Self {
        Self {
            status: value.status,
            count: value.count,
        }
    }
}

impl From<aqardesk_infrastructure::dashboard::MonthlyPayment> for MonthlyPayment {
    fn from(value: aqardesk_infrastructure::dashboard::MonthlyPayment) -> Self {
        Self {
            month: value.month,
            currency_code: value.currency_code,
            currency_decimal_digits: value.currency_decimal_digits,
            amount_minor: value.amount_minor,
            payment_count: value.payment_count,
        }
    }
}

impl From<aqardesk_infrastructure::dashboard::OpenChargesByCurrency> for OpenChargesByCurrency {
    fn from(value: aqardesk_infrastructure::dashboard::OpenChargesByCurrency) -> Self {
        Self {
            currency_code: value.currency_code,
            currency_decimal_digits: value.currency_decimal_digits,
            amount_minor: value.amount_minor,
            charge_count: value.charge_count,
        }
    }
}

impl From<aqardesk_infrastructure::dashboard::DashboardSummary> for DashboardSummary {
    fn from(value: aqardesk_infrastructure::dashboard::DashboardSummary) -> Self {
        Self {
            active_properties_count: value.active_properties_count,
            units_count: value.units_count,
            available_units_count: value.available_units_count,
            tenants_count: value.tenants_count,
            active_leases_count: value.active_leases_count,
            open_charges_count: value.open_charges_count,
            posted_payments_count: value.posted_payments_count,
            contract_documents_count: value.contract_documents_count,
            expiring_leases_count: value.expiring_leases_count,
            payment_months: value.payment_months,
            monthly_payments: value.monthly_payments.into_iter().map(Into::into).collect(),
            unit_status_counts: value
                .unit_status_counts
                .into_iter()
                .map(Into::into)
                .collect(),
            lease_status_counts: value
                .lease_status_counts
                .into_iter()
                .map(Into::into)
                .collect(),
            open_charges_by_currency: value
                .open_charges_by_currency
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
