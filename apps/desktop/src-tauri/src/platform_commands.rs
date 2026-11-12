use crate::platform_runtime::{
    self, ConnectionConfig, ConnectionInput, ConnectionMode, ConnectionStatus, ConnectionView,
    PlatformState, SavedSession,
};
use aqardesk_backend::{
    application::{
        IdentityService, NewUser, OrganizationBootstrap, OrganizationSettingsService,
        OwnerDirectory, PaymentContract, PaymentLedger, PropertyCatalog, TenantDirectory,
        UnitCatalog, UserUpdate,
    },
    domain::{
        ActorContext, Owner, OwnerId, OwnerInput, Payment, PaymentInput, Permission, Property,
        PropertyId, PropertyInput, PropertyUnit, PropertyUnitInput, RolePermissionSet, Tenant,
        TenantId, TenantInput, UnitId, UserAccount, UserId,
    },
    infrastructure::{
        SeaOrmIdentityRepository, SeaOrmOrganizationSettingsRepository, SeaOrmOwnerRepository,
        SeaOrmPaymentRepository, SeaOrmPropertyRepository, SeaOrmTenantRepository,
        SeaOrmUnitRepository,
    },
};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PlatformSetupStatus {
    pub needs_admin: bool,
    pub current_user: Option<UserAccount>,
}

#[derive(Debug, Serialize)]
pub struct PropertyView {
    pub id: String,
    pub organization_id: String,
    pub code: String,
    pub name: String,
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub district: Option<String>,
    pub neighborhood: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub lot_area_square_meters: Option<f64>,
    pub building_area_square_meters: Option<f64>,
    pub frontage_meters: Option<f64>,
    pub depth_meters: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub owners: Vec<PropertyOwnerView>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct PropertyOwnerView {
    pub id: String,
    pub name: String,
    pub phone_number: String,
    pub phone_label: Option<String>,
    pub is_whatsapp: bool,
    pub ownership_share_percent: Option<f64>,
}

impl TryFrom<Property> for PropertyView {
    type Error = String;

    fn try_from(value: Property) -> Result<Self, Self::Error> {
        let input = value.input;
        let owners = input
            .owners
            .into_iter()
            .map(|owner| {
                Ok(PropertyOwnerView {
                    id: owner.id.ok_or_else(|| {
                        "A saved property owner is missing its identifier.".to_string()
                    })?,
                    name: owner.name,
                    phone_number: owner.phone_number,
                    phone_label: owner.phone_label,
                    is_whatsapp: owner.is_whatsapp,
                    ownership_share_percent: decimal_to_f64(owner.ownership_share_percent)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            code: input.code,
            name: input.name,
            property_type: input.property_type.as_str().to_string(),
            description: input.description,
            address_line_1: input.address_line_1,
            address_line_2: input.address_line_2,
            city: input.city,
            region: input.region,
            district: input.district,
            neighborhood: input.neighborhood,
            postal_code: input.postal_code,
            country_code: input.country_code,
            lot_area_square_meters: decimal_to_f64(input.lot_area_square_meters)?,
            building_area_square_meters: decimal_to_f64(input.building_area_square_meters)?,
            frontage_meters: decimal_to_f64(input.frontage_meters)?,
            depth_meters: decimal_to_f64(input.depth_meters)?,
            latitude: decimal_to_f64(input.latitude)?,
            longitude: decimal_to_f64(input.longitude)?,
            owners,
            status: input.status.as_str().to_string(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        })
    }
}

fn decimal_to_f64(value: Option<rust_decimal::Decimal>) -> Result<Option<f64>, String> {
    value
        .map(|value| {
            value
                .to_f64()
                .ok_or_else(|| "A property measurement cannot be represented safely.".to_string())
        })
        .transpose()
}

#[derive(Debug, Serialize)]
pub struct UnitView {
    pub id: String,
    pub organization_id: String,
    pub property_id: String,
    pub unit_number: String,
    pub unit_type: String,
    pub floor_number: Option<i32>,
    pub area_square_meters: Option<f64>,
    pub bedrooms: u32,
    pub bathrooms: f64,
    pub market_rent_minor: Option<i64>,
    pub currency_code: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl TryFrom<PropertyUnit> for UnitView {
    type Error = String;

    fn try_from(value: PropertyUnit) -> Result<Self, Self::Error> {
        let area_square_meters = value
            .input
            .area_square_meters
            .map(|value| {
                value
                    .to_f64()
                    .ok_or_else(|| "Unit area could not be represented.".to_string())
            })
            .transpose()?;
        let bathrooms = value
            .input
            .bathrooms
            .to_f64()
            .ok_or_else(|| "Bathroom count could not be represented.".to_string())?;
        Ok(Self {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            property_id: value.input.property_id.to_string(),
            unit_number: value.input.unit_number,
            unit_type: value.input.unit_type.as_str().to_string(),
            floor_number: value.input.floor_number,
            area_square_meters,
            bedrooms: value.input.bedrooms,
            bathrooms,
            market_rent_minor: value.input.market_rent_minor,
            currency_code: value
                .input
                .currency_code
                .map(|value| value.as_str().to_string()),
            status: value.input.status.as_str().to_string(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct TenantView {
    pub id: String,
    pub organization_id: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Tenant> for TenantView {
    fn from(value: Tenant) -> Self {
        Self {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            full_name: value.input.full_name,
            email: value.input.email,
            phone: value.input.phone,
            mailing_address: value.input.mailing_address,
            notes: value.input.notes,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OwnerView {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub phone_number: String,
    pub phone_label: Option<String>,
    pub is_whatsapp: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Owner> for OwnerView {
    fn from(value: Owner) -> Self {
        Self {
            id: value.id.to_string(),
            organization_id: value.organization_id.to_string(),
            name: value.input.name,
            phone_number: value.input.phone_number,
            phone_label: value.input.phone_label,
            is_whatsapp: value.input.is_whatsapp,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaymentView {
    pub id: String,
    pub tenant_id: String,
    pub tenant_name: String,
    pub received_at: String,
    pub amount_minor: i64,
    pub currency_code: String,
    pub payment_method: String,
    pub allocations: Vec<PaymentAllocationView>,
    pub external_reference: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub voided_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentAllocationView {
    pub contract_document_id: String,
    pub contract_number: String,
    pub property_name: String,
    pub property_code: String,
    pub unit_number: String,
    pub amount_minor: i64,
}

impl From<PaymentContract> for PaymentContractView {
    fn from(value: PaymentContract) -> Self {
        Self {
            id: value.id,
            contract_number: value.contract_number,
            property_name: value.property_name,
            property_code: value.property_code,
            unit_number: value.unit_number,
            start_date: value.start_date,
            end_date: value.end_date,
            currency_code: value.currency_code,
            currency_decimal_digits: value.currency_decimal_digits,
            rent_amount_minor: value.rent_amount_minor,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaymentContractView {
    pub id: String,
    pub contract_number: String,
    pub property_name: String,
    pub property_code: String,
    pub unit_number: String,
    pub start_date: String,
    pub end_date: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub rent_amount_minor: i64,
}

impl From<(Payment, String)> for PaymentView {
    fn from((value, tenant_name): (Payment, String)) -> Self {
        Self {
            id: value.id.to_string(),
            tenant_id: value.tenant_id.to_string(),
            tenant_name,
            received_at: value.received_at.to_rfc3339(),
            amount_minor: value.amount.amount_minor(),
            currency_code: value.amount.currency().as_str().to_string(),
            payment_method: value.method.as_str().to_string(),
            allocations: value
                .allocations
                .into_iter()
                .map(|allocation| PaymentAllocationView {
                    contract_document_id: allocation.contract_document_id,
                    contract_number: allocation.contract_number,
                    property_name: allocation.property_name,
                    property_code: allocation.property_code,
                    unit_number: allocation.unit_number,
                    amount_minor: allocation.amount_minor,
                })
                .collect(),
            external_reference: value.external_reference,
            status: value.status.as_str().to_string(),
            notes: value.notes,
            voided_at: value.voided_at.map(|value| value.to_rfc3339()),
        }
    }
}

pub(crate) fn application_error(error: aqardesk_backend::application::ApplicationError) -> String {
    match error {
        aqardesk_backend::application::ApplicationError::Domain(error) => error.to_string(),
        aqardesk_backend::application::ApplicationError::Authentication => {
            "Email or password is incorrect.".to_string()
        }
        aqardesk_backend::application::ApplicationError::Forbidden => {
            "This account is not permitted to perform this operation.".to_string()
        }
        error => {
            tracing::error!(%error, "Platform application operation failed");
            "The operation could not be completed.".to_string()
        }
    }
}

async fn local_identity(
    state: &PlatformState,
) -> Result<IdentityService<SeaOrmIdentityRepository>, String> {
    let database = state.database().await?;
    Ok(IdentityService::new(SeaOrmIdentityRepository::new(
        database.connection(),
    )))
}

pub(crate) async fn local_business_state(
    state: &PlatformState,
) -> Result<(ActorContext, sea_orm::DatabaseConnection), String> {
    let actor = active_actor(state).await?;
    let database = state.database().await?;
    Ok((actor, database.connection().clone()))
}

pub(crate) fn config_key(config: &ConnectionConfig) -> Result<String, String> {
    serde_json::to_string(config).map_err(|error| {
        tracing::error!(%error, "Could not identify active connection");
        "Could not validate the saved session.".to_string()
    })
}

pub(crate) async fn rebind_current_session(
    state: &PlatformState,
    config: &ConnectionConfig,
) -> Result<(), String> {
    let Some(user) = state.current_user.read().await.clone() else {
        return Ok(());
    };
    let Some(mut session) = platform_runtime::load_session()? else {
        return Ok(());
    };
    session.user_id = user.id.to_string();
    session.connection_key = config_key(config)?;
    platform_runtime::save_session(&session)
}

pub(crate) async fn active_actor(state: &PlatformState) -> Result<ActorContext, String> {
    let config = state.config().await?;
    if config.mode == ConnectionMode::Api {
        return Err("API authentication is provided by the configured server.".to_string());
    }
    let session_user = state
        .current_user
        .read()
        .await
        .clone()
        .ok_or_else(|| "Sign in before continuing.".to_string())?;
    let current_user = match local_identity(state)
        .await?
        .current_user(session_user.id)
        .await
    {
        Ok(user) => user,
        Err(aqardesk_backend::application::ApplicationError::Authentication) => {
            *state.current_user.write().await = None;
            platform_runtime::clear_session()?;
            return Err("The session is no longer valid. Sign in again.".to_string());
        }
        Err(error) => return Err(application_error(error)),
    };
    *state.current_user.write().await = Some(current_user.clone());
    Ok(ActorContext::with_permissions(
        current_user.id,
        current_user.organization_id,
        current_user.role,
        current_user.permissions,
    ))
}

async fn restore_local_session(state: &PlatformState) -> Result<Option<UserAccount>, String> {
    let config = state.config().await?;
    if config.mode == ConnectionMode::Api {
        return Ok(None);
    }
    let Some(saved) = platform_runtime::load_session()? else {
        *state.current_user.write().await = None;
        return Ok(None);
    };
    if saved.connection_key != config_key(&config)? {
        *state.current_user.write().await = None;
        platform_runtime::clear_session()?;
        return Ok(None);
    }
    let user_id = match UserId::parse(&saved.user_id) {
        Ok(user_id) => user_id,
        Err(_) => {
            *state.current_user.write().await = None;
            platform_runtime::clear_session()?;
            return Ok(None);
        }
    };
    let user = match local_identity(state).await?.current_user(user_id).await {
        Ok(user) => user,
        Err(aqardesk_backend::application::ApplicationError::Authentication) => {
            *state.current_user.write().await = None;
            platform_runtime::clear_session()?;
            return Ok(None);
        }
        Err(error) => return Err(application_error(error)),
    };
    *state.current_user.write().await = Some(user.clone());
    Ok(Some(user))
}

#[tauri::command]
pub async fn platform_connection_get_config(
    state: State<'_, PlatformState>,
) -> Result<Option<ConnectionView>, String> {
    Ok(state.config.read().await.as_ref().map(ConnectionView::from))
}

#[tauri::command]
pub async fn platform_connection_set_config(
    state: State<'_, PlatformState>,
    input: ConnectionInput,
) -> Result<ConnectionView, String> {
    let existing = state.config.read().await.clone();
    let config = platform_runtime::make_config(input, existing.as_ref(), &state.app_data_dir)?;
    platform_runtime::save_config(&state.app_data_dir, &config)?;
    let changed = existing.as_ref() != Some(&config);
    *state.config.write().await = Some(config.clone());
    *state.database.write().await = None;
    if changed {
        *state.current_user.write().await = None;
        *state.api_token.write().await = None;
        platform_runtime::clear_session()?;
    }
    Ok(ConnectionView::from(&config))
}

#[tauri::command]
pub async fn platform_connection_check_api(
    state: State<'_, PlatformState>,
    server_url: String,
) -> Result<ConnectionStatus, String> {
    let url = platform_runtime::ConnectionConfig {
        mode: ConnectionMode::Api,
        local: None,
        direct: None,
        api: Some(platform_runtime::ApiConnection { server_url }),
    }
    .api_url()?
    .trim_end_matches('/')
    .to_string();
    check_api(&state.http, &url).await
}

async fn check_api(http: &reqwest::Client, url: &str) -> Result<ConnectionStatus, String> {
    let response = http
        .get(format!("{url}/api/v1/health"))
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(%error, "API health check failed");
            "Could not reach the API server.".to_string()
        })?;
    if !response.status().is_success() {
        return Err(format!("API server returned HTTP {}.", response.status()));
    }
    Ok(ConnectionStatus {
        connected: true,
        mode: "api".to_string(),
        message: "API server is reachable.".to_string(),
    })
}

#[tauri::command]
pub async fn platform_connection_connect(
    state: State<'_, PlatformState>,
) -> Result<ConnectionStatus, String> {
    let config = state.config().await?;
    if config.mode == ConnectionMode::Api {
        let server_url = config.api_url()?.to_string();
        return check_api(&state.http, &server_url).await;
    }
    state.connect().await?;
    Ok(ConnectionStatus {
        connected: true,
        mode: match config.mode {
            ConnectionMode::Local => "local",
            ConnectionMode::Direct => "direct",
            ConnectionMode::Api => "api",
        }
        .to_string(),
        message: "Database connection is ready and migrations are applied.".to_string(),
    })
}

#[tauri::command]
pub async fn platform_auth_get_setup_status(
    state: State<'_, PlatformState>,
) -> Result<PlatformSetupStatus, String> {
    let config = state.config().await?;
    if config.mode == ConnectionMode::Api {
        return Err("API authentication is provided by the configured server.".to_string());
    }
    let current_user = restore_local_session(&state).await?;
    let needs_admin = local_identity(&state)
        .await?
        .setup_status()
        .await
        .map_err(application_error)?;
    Ok(PlatformSetupStatus {
        needs_admin,
        current_user,
    })
}

#[tauri::command]
pub async fn platform_auth_bootstrap_admin(
    state: State<'_, PlatformState>,
    organization_name: String,
    name: String,
    email: String,
    password: String,
) -> Result<UserAccount, String> {
    if state.config().await?.mode == ConnectionMode::Api {
        return Err("API authentication is provided by the configured server.".to_string());
    }
    let user = local_identity(&state)
        .await?
        .bootstrap_admin(OrganizationBootstrap {
            organization_name: &organization_name,
            name: &name,
            email: &email,
            password: &password,
        })
        .await
        .map_err(application_error)?
        .1;
    persist_local_session(&state, &user).await?;
    Ok(user)
}

#[tauri::command]
pub async fn platform_auth_login(
    state: State<'_, PlatformState>,
    email: String,
    password: String,
) -> Result<UserAccount, String> {
    if state.config().await?.mode == ConnectionMode::Api {
        return Err("API authentication is provided by the configured server.".to_string());
    }
    let user = local_identity(&state)
        .await?
        .login(&email, &password)
        .await
        .map_err(application_error)?;
    persist_local_session(&state, &user).await?;
    Ok(user)
}

async fn persist_local_session(state: &PlatformState, user: &UserAccount) -> Result<(), String> {
    let config = state
        .config
        .read()
        .await
        .clone()
        .ok_or_else(|| "Choose and save a database connection first.".to_string())?;
    platform_runtime::save_session(&SavedSession {
        user_id: user.id.to_string(),
        api_token: None,
        connection_key: config_key(&config)?,
    })?;
    *state.current_user.write().await = Some(user.clone());
    Ok(())
}

#[tauri::command]
pub async fn platform_auth_logout(state: State<'_, PlatformState>) -> Result<(), String> {
    *state.current_user.write().await = None;
    *state.api_token.write().await = None;
    platform_runtime::clear_session()
}

#[tauri::command]
pub async fn platform_organization_get_name(
    state: State<'_, PlatformState>,
) -> Result<String, String> {
    let actor = active_actor(&state).await?;
    local_identity(&state)
        .await?
        .organization_name(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_users_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<UserAccount>, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    let identities = local_identity(&state).await?;
    identities
        .list_users(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_role_permissions_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<RolePermissionSet>, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    local_identity(&state)
        .await?
        .list_role_permissions(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_role_permissions_update(
    state: State<'_, PlatformState>,
    role_id: String,
    permissions: Vec<Permission>,
) -> Result<RolePermissionSet, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    local_identity(&state)
        .await?
        .update_role_permissions(actor, &role_id, permissions)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_roles_create(
    state: State<'_, PlatformState>,
    name: String,
    permissions: Vec<Permission>,
) -> Result<RolePermissionSet, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    local_identity(&state)
        .await?
        .create_role(actor, &name, permissions)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_roles_delete(
    state: State<'_, PlatformState>,
    role_id: String,
) -> Result<(), String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    local_identity(&state)
        .await?
        .delete_role(actor, &role_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_users_create(
    state: State<'_, PlatformState>,
    name: String,
    email: String,
    password: String,
    role_id: String,
) -> Result<UserAccount, String> {
    let actor = active_actor(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "user_management")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    let identities = local_identity(&state).await?;
    let current_count = identities
        .list_users(actor)
        .await
        .map_err(application_error)?
        .into_iter()
        .filter(|user| user.is_active)
        .count() as u64;
    state
        .license
        .check_limit_for_organization(&organization_id, "users.max", current_count, 1)
        .await?;
    identities
        .create_user(
            actor,
            NewUser {
                name: &name,
                email: &email,
                password: &password,
                role_id: &role_id,
            },
        )
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_users_update(
    state: State<'_, PlatformState>,
    user_id: String,
    name: String,
    email: String,
    role_id: String,
    password: Option<String>,
) -> Result<UserAccount, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    let user_id = UserId::parse(&user_id)
        .map_err(|_| "The selected user identifier is invalid.".to_string())?;
    local_identity(&state)
        .await?
        .update_user(
            actor,
            user_id,
            UserUpdate {
                name: &name,
                email: &email,
                password: password.as_deref(),
                role_id: &role_id,
            },
        )
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_users_set_active(
    state: State<'_, PlatformState>,
    user_id: String,
    is_active: bool,
) -> Result<(), String> {
    let actor = active_actor(&state).await?;
    let user_id = UserId::parse(&user_id)
        .map_err(|_| "The selected user identifier is invalid.".to_string())?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "user_management")
        .await?;
    let identities = local_identity(&state).await?;
    let _quota_guard = if is_active {
        Some(state.license_quota_lock.lock().await)
    } else {
        None
    };
    if is_active {
        let users = identities
            .list_users(actor)
            .await
            .map_err(application_error)?;
        if users
            .iter()
            .any(|user| user.id == user_id && !user.is_active)
        {
            let active_count = users.iter().filter(|user| user.is_active).count() as u64;
            state
                .license
                .check_limit_for_organization(
                    &actor.organization_id().to_string(),
                    "users.max",
                    active_count,
                    1,
                )
                .await?;
        }
    }
    identities
        .set_user_active(actor, user_id, is_active)
        .await
        .map_err(application_error)?;
    Ok(())
}

#[tauri::command]
pub async fn platform_properties_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<PropertyView>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    let properties = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)?;
    properties.into_iter().map(PropertyView::try_from).collect()
}

#[tauri::command]
pub async fn platform_properties_create(
    state: State<'_, PlatformState>,
    input: PropertyInput,
) -> Result<PropertyView, String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "properties")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    let catalog = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database));
    let current_count = catalog.list(actor).await.map_err(application_error)?.len() as u64;
    state
        .license
        .check_limit_for_organization(&organization_id, "properties.max", current_count, 1)
        .await?;
    let property = catalog
        .create(actor, input)
        .await
        .map_err(application_error)?;
    PropertyView::try_from(property)
}

#[tauri::command]
pub async fn platform_properties_update(
    state: State<'_, PlatformState>,
    property_id: String,
    input: PropertyInput,
) -> Result<PropertyView, String> {
    let property_id = PropertyId::parse(&property_id)
        .map_err(|_| "The selected property identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    let property = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database))
        .update(actor, property_id, input)
        .await
        .map_err(application_error)?;
    PropertyView::try_from(property)
}

#[tauri::command]
pub async fn platform_properties_archive(
    state: State<'_, PlatformState>,
    property_id: String,
) -> Result<(), String> {
    let property_id = PropertyId::parse(&property_id)
        .map_err(|_| "The selected property identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    PropertyCatalog::new(SeaOrmPropertyRepository::new(&database))
        .archive(actor, property_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_units_list(
    state: State<'_, PlatformState>,
    property_id: Option<String>,
) -> Result<Vec<UnitView>, String> {
    let property_id = property_id
        .map(|value| {
            PropertyId::parse(&value)
                .map_err(|_| "The selected property identifier is invalid.".to_string())
        })
        .transpose()?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "units")
        .await?;
    let units = UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .list(actor, property_id)
        .await
        .map_err(application_error)?;
    units.into_iter().map(UnitView::try_from).collect()
}

#[tauri::command]
pub async fn platform_units_create(
    state: State<'_, PlatformState>,
    input: PropertyUnitInput,
) -> Result<UnitView, String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "units")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    let catalog = UnitCatalog::new(SeaOrmUnitRepository::new(&database));
    let current_count = catalog
        .list(actor, None)
        .await
        .map_err(application_error)?
        .len() as u64;
    state
        .license
        .check_limit_for_organization(&organization_id, "units.max", current_count, 1)
        .await?;
    let unit = catalog
        .create(actor, input)
        .await
        .map_err(application_error)?;
    UnitView::try_from(unit)
}

#[tauri::command]
pub async fn platform_units_update(
    state: State<'_, PlatformState>,
    unit_id: String,
    input: PropertyUnitInput,
) -> Result<UnitView, String> {
    let unit_id = UnitId::parse(&unit_id)
        .map_err(|_| "The selected unit identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "units")
        .await?;
    let unit = UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .update(actor, unit_id, input)
        .await
        .map_err(application_error)?;
    UnitView::try_from(unit)
}

#[tauri::command]
pub async fn platform_units_archive(
    state: State<'_, PlatformState>,
    unit_id: String,
) -> Result<(), String> {
    let unit_id = UnitId::parse(&unit_id)
        .map_err(|_| "The selected unit identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "units")
        .await?;
    UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .archive(actor, unit_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_owners_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<OwnerView>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    OwnerDirectory::new(SeaOrmOwnerRepository::new(&database))
        .list(actor)
        .await
        .map(|owners| owners.into_iter().map(OwnerView::from).collect())
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_owners_create(
    state: State<'_, PlatformState>,
    input: OwnerInput,
) -> Result<OwnerView, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    OwnerDirectory::new(SeaOrmOwnerRepository::new(&database))
        .create(actor, input)
        .await
        .map(OwnerView::from)
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_owners_update(
    state: State<'_, PlatformState>,
    owner_id: String,
    input: OwnerInput,
) -> Result<OwnerView, String> {
    let owner_id = OwnerId::parse(&owner_id)
        .map_err(|_| "The selected owner identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "properties")
        .await?;
    OwnerDirectory::new(SeaOrmOwnerRepository::new(&database))
        .update(actor, owner_id, input)
        .await
        .map(OwnerView::from)
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_tenants_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<TenantView>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "tenants")
        .await?;
    TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .list(actor)
        .await
        .map(|tenants| tenants.into_iter().map(TenantView::from).collect())
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_tenants_create(
    state: State<'_, PlatformState>,
    input: TenantInput,
) -> Result<TenantView, String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "tenants")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    let directory = TenantDirectory::new(SeaOrmTenantRepository::new(&database));
    let current_count = directory
        .list(actor)
        .await
        .map_err(application_error)?
        .len() as u64;
    state
        .license
        .check_limit_for_organization(&organization_id, "tenants.max", current_count, 1)
        .await?;
    directory
        .create(actor, input)
        .await
        .map(TenantView::from)
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_tenants_update(
    state: State<'_, PlatformState>,
    tenant_id: String,
    input: TenantInput,
) -> Result<TenantView, String> {
    let tenant_id = TenantId::parse(&tenant_id)
        .map_err(|_| "The selected tenant identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "tenants")
        .await?;
    TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .update(actor, tenant_id, input)
        .await
        .map(TenantView::from)
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_tenants_archive(
    state: State<'_, PlatformState>,
    tenant_id: String,
) -> Result<(), String> {
    let tenant_id = TenantId::parse(&tenant_id)
        .map_err(|_| "The selected tenant identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "tenants")
        .await?;
    TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .archive(actor, tenant_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_payments_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<PaymentView>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "payments")
        .await?;
    let payments = PaymentLedger::new(SeaOrmPaymentRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)?;
    let tenant_ids = payments
        .iter()
        .map(|payment| payment.tenant_id)
        .collect::<Vec<_>>();
    let tenant_names = TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .names(actor, &tenant_ids)
        .await
        .map_err(application_error)?;
    payments
        .into_iter()
        .map(|payment| {
            let tenant_name = tenant_names
                .get(&payment.tenant_id)
                .cloned()
                .ok_or_else(|| {
                    tracing::error!(
                        tenant_id = %payment.tenant_id,
                        "Payment has no tenant record in its organization"
                    );
                    "One or more payment records reference unavailable tenants.".to_string()
                })?;
            Ok::<PaymentView, String>(PaymentView::from((payment, tenant_name)))
        })
        .collect::<Result<Vec<_>, String>>()
}

#[tauri::command]
pub async fn platform_payments_record(
    state: State<'_, PlatformState>,
    input: PaymentInput,
) -> Result<PaymentView, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "payments")
        .await?;
    let organization_settings =
        OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database))
            .get(actor)
            .await
            .map_err(application_error)?;
    if input.currency_code.as_str() != organization_settings.default_currency {
        let organization_id = actor.organization_id().to_string();
        state
            .license
            .require_feature(&organization_id, "multi_currency")
            .await?;
        let enabled_currencies = aqardesk_backend::application::CurrencyCatalog::new(
            aqardesk_backend::infrastructure::SeaOrmCurrencyRepository::new(&database),
        )
        .list(actor)
        .await
        .map_err(application_error)?
        .into_iter()
        .filter(|currency| currency.is_enabled_for_organization)
        .count() as u64;
        state
            .license
            .check_limit_for_organization(&organization_id, "currencies.max", enabled_currencies, 0)
            .await?;
    }
    let tenant_names = TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .names(actor, &[input.tenant_id])
        .await
        .map_err(application_error)?;
    let tenant_name = tenant_names
        .get(&input.tenant_id)
        .cloned()
        .ok_or_else(|| "The selected tenant is not available.".to_string())?;
    let payment = PaymentLedger::new(SeaOrmPaymentRepository::new(&database))
        .record(actor, input)
        .await
        .map_err(application_error)?;
    Ok(PaymentView::from((payment, tenant_name)))
}

#[tauri::command]
pub async fn platform_payments_contracts(
    state: State<'_, PlatformState>,
    tenant_id: String,
) -> Result<Vec<PaymentContractView>, String> {
    let tenant_id = TenantId::parse(&tenant_id)
        .map_err(|_| "The selected tenant identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "payments")
        .await?;
    PaymentLedger::new(SeaOrmPaymentRepository::new(&database))
        .contracts(actor, tenant_id)
        .await
        .map(|contracts| {
            contracts
                .into_iter()
                .map(PaymentContractView::from)
                .collect()
        })
        .map_err(application_error)
}

#[tauri::command]
pub async fn platform_payments_void(
    state: State<'_, PlatformState>,
    payment_id: String,
) -> Result<(), String> {
    let payment_id = aqardesk_backend::domain::PaymentId::parse(&payment_id)
        .map_err(|_| "The selected payment identifier is invalid.".to_string())?;
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "payments")
        .await?;
    PaymentLedger::new(SeaOrmPaymentRepository::new(&database))
        .void(actor, payment_id)
        .await
        .map_err(application_error)
}
