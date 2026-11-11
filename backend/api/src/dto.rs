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

impl From<PropertyInput> for aqardesk_infrastructure::services::properties::PropertyInput {
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

impl From<PropertyUnitInput> for aqardesk_infrastructure::services::units::UnitInput {
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

impl From<TenantInput> for aqardesk_infrastructure::services::tenants::TenantInput {
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
