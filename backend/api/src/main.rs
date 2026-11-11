use aqardesk_app::{
    AppError,
    services::{
        payments::PaymentService, properties::PropertyService, settings::SettingsService,
        tenants::TenantService, units::UnitService,
    },
};
use aqardesk_infrastructure::{
    connect_database,
    repositories::{
        payments::SeaOrmPaymentRepository, properties::SeaOrmPropertyRepository,
        settings::SeaOrmSettingsRepository, tenants::SeaOrmTenantRepository,
        units::SeaOrmUnitRepository,
    },
    run_migrations,
    settings::{ConnectionConfig, ConnectionMode},
    users::{self, UserSummary},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post, put},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

mod dto;
mod openapi;

#[derive(Clone)]
struct ApiState {
    database: DatabaseConnection,
    sessions: Arc<RwLock<HashMap<String, UserSummary>>>,
}

#[derive(Debug, Serialize, ToSchema)]
struct HealthPayload {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
struct ReadinessPayload {
    ready: bool,
    database: bool,
}

#[derive(Debug, Deserialize)]
struct BootstrapRequest {
    organization_name: String,
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    is_active: bool,
}

#[derive(Debug, Deserialize)]
struct DashboardQuery {
    months: Option<u32>,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    user: UserSummary,
    token: String,
}

#[derive(Debug, Serialize)]
struct SetupResponse {
    needs_admin: bool,
}

fn error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

#[utoipa::path(
    get,
    operation_id = "getHealth",
    path = "/api/v1/health",
    tag = "System",
    responses((status = 200, description = "API health", body = HealthPayload))
)]
async fn health() -> Json<HealthPayload> {
    Json(HealthPayload {
        status: "ok",
        service: "aqardesk-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi::ApiDoc::openapi())
}

#[utoipa::path(
    get,
    operation_id = "getReadiness",
    path = "/api/v1/ready",
    tag = "System",
    responses((status = 200, description = "API and database readiness", body = ReadinessPayload))
)]
async fn readiness(State(state): State<ApiState>) -> Json<ReadinessPayload> {
    let database = state.database.ping().await.is_ok();
    Json(ReadinessPayload {
        ready: database,
        database,
    })
}

async fn setup_status(
    State(state): State<ApiState>,
) -> Result<Json<SetupResponse>, (StatusCode, Json<serde_json::Value>)> {
    let has_users = users::has_users(&state.database)
        .await
        .map_err(|message| error(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    Ok(Json(SetupResponse {
        needs_admin: !has_users,
    }))
}

async fn bootstrap_admin(
    State(state): State<ApiState>,
    Json(request): Json<BootstrapRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = users::create_initial_admin(
        &state.database,
        &request.organization_name,
        &request.name,
        &request.email,
        &request.password,
    )
    .await
    .map_err(user_operation_error)?;
    Ok(Json(create_session(&state, user).await))
}

async fn login(
    State(state): State<ApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = users::authenticate(&state.database, &request.email, &request.password)
        .await
        .map_err(|message| error(StatusCode::UNAUTHORIZED, message))?;
    Ok(Json(create_session(&state, user).await))
}

async fn get_session(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<UserSummary>, (StatusCode, Json<serde_json::Value>)> {
    let token = session_token(&headers)?;
    let user = state
        .sessions
        .read()
        .await
        .get(token)
        .cloned()
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session expired. Sign in again."))?;
    let user = users::get_active_user(&state.database, &user.id)
        .await
        .map_err(|message| error(StatusCode::INTERNAL_SERVER_ERROR, message))?
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session expired. Sign in again."))?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    operation_id = "getCurrentOrganization",
    path = "/api/v1/organizations/current",
    tag = "Organization",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current organization settings", body = dto::OrganizationSettings),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Organization not found", body = dto::ErrorResponse)
    )
)]
async fn get_current_organization(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<dto::OrganizationSettings>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .get_organization(&user.organization_id)
        .await
        .map(Into::into)
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    put,
    operation_id = "updateCurrentOrganization",
    path = "/api/v1/organizations/current",
    tag = "Organization",
    request_body = dto::OrganizationSettingsInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated organization settings", body = dto::OrganizationSettings),
        (status = 400, description = "Invalid organization settings", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse)
    )
)]
async fn update_current_organization(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::OrganizationSettingsInput>,
) -> Result<Json<dto::OrganizationSettings>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .update_organization(&user.organization_id, &user.id, input.into())
        .await
        .map(Into::into)
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    get,
    operation_id = "listCurrencies",
    path = "/api/v1/currencies",
    tag = "Settings",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Currencies and their organization availability", body = [dto::CurrencySettings]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn list_currencies(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::CurrencySettings>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .list_currencies(&user.organization_id)
        .await
        .map(|currencies| currencies.into_iter().map(Into::into).collect())
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    put,
    operation_id = "setOrganizationCurrencyState",
    path = "/api/v1/currencies/{currency_code}/organization-state",
    tag = "Settings",
    request_body = dto::OrganizationCurrencyStateInput,
    params(("currency_code" = String, Path, description = "ISO currency code")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated organization currency availability", body = dto::OperationResult),
        (status = 400, description = "Currency cannot be disabled or request is invalid", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse),
        (status = 404, description = "Currency not found", body = dto::ErrorResponse)
    )
)]
async fn set_organization_currency_state(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(currency_code): Path<String>,
    Json(input): Json<dto::OrganizationCurrencyStateInput>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .set_currency_enabled(
            &user.organization_id,
            &user.id,
            &currency_code,
            input.is_enabled,
        )
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

#[utoipa::path(
    get,
    operation_id = "getMyProfile",
    path = "/api/v1/users/me",
    tag = "Settings",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = dto::ProfileSettings),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn get_my_profile(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<dto::ProfileSettings>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .get_profile(&user.organization_id, &user.id)
        .await
        .map(Into::into)
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    put,
    operation_id = "updateMyProfile",
    path = "/api/v1/users/me",
    tag = "Settings",
    request_body = dto::ProfileSettingsInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated user profile", body = dto::ProfileSettings),
        (status = 400, description = "Invalid profile settings", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn update_my_profile(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::ProfileSettingsInput>,
) -> Result<Json<dto::ProfileSettings>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .update_profile(&user.organization_id, &user.id, input.into())
        .await
        .map(Into::into)
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    get,
    operation_id = "listAuditEvents",
    path = "/api/v1/audit-events",
    tag = "Settings",
    params(
        ("limit" = Option<u64>, Query, description = "Maximum entries, 1 to 100"),
        ("offset" = Option<u64>, Query, description = "Number of entries to skip")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization audit events", body = [dto::AuditEntry]),
        (status = 400, description = "Invalid paging options", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse)
    )
)]
async fn list_audit_events(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<dto::AuditQuery>,
) -> Result<Json<Vec<dto::AuditEntry>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .list_audit(
            &user.organization_id,
            query.limit.unwrap_or(50),
            query.offset.unwrap_or(0),
        )
        .await
        .map(|entries| entries.into_iter().map(Into::into).collect())
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    get,
    operation_id = "listContractTemplates",
    path = "/api/v1/contract-templates",
    tag = "Settings",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization contract templates", body = [dto::ContractTemplate]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse)
    )
)]
async fn list_contract_templates(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::ContractTemplate>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .list_templates(&user.organization_id)
        .await
        .map(|templates| templates.into_iter().map(Into::into).collect())
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    post,
    operation_id = "createContractTemplate",
    path = "/api/v1/contract-templates",
    tag = "Settings",
    request_body = dto::ContractTemplateInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Created contract template", body = dto::ContractTemplate),
        (status = 400, description = "Invalid contract template", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse)
    )
)]
async fn create_contract_template(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::ContractTemplateInput>,
) -> Result<(StatusCode, Json<dto::ContractTemplate>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    let template = service
        .create_template(&user.organization_id, &user.id, input.into())
        .await
        .map_err(business_error)?;
    Ok((StatusCode::CREATED, Json(template.into())))
}

#[utoipa::path(
    put,
    operation_id = "createContractTemplateVersion",
    path = "/api/v1/contract-templates/{template_id}",
    tag = "Settings",
    request_body = dto::ContractTemplateInput,
    params(("template_id" = String, Path, description = "Contract template identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated template with a new immutable version", body = dto::ContractTemplate),
        (status = 400, description = "Invalid contract template", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse),
        (status = 404, description = "Template not found", body = dto::ErrorResponse)
    )
)]
async fn update_contract_template(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(template_id): Path<String>,
    Json(input): Json<dto::ContractTemplateInput>,
) -> Result<Json<dto::ContractTemplate>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .update_template(&user.organization_id, &user.id, &template_id, input.into())
        .await
        .map(Into::into)
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    patch,
    operation_id = "setContractTemplateState",
    path = "/api/v1/contract-templates/{template_id}/state",
    tag = "Settings",
    request_body = dto::ContractTemplateStateInput,
    params(("template_id" = String, Path, description = "Contract template identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated contract template state", body = dto::OperationResult),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 403, description = "Administrator access is required", body = dto::ErrorResponse),
        (status = 404, description = "Template not found", body = dto::ErrorResponse)
    )
)]
async fn set_contract_template_state(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(template_id): Path<String>,
    Json(input): Json<dto::ContractTemplateStateInput>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_admin(&state, &headers).await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&state.database));
    service
        .set_template_active(
            &user.organization_id,
            &user.id,
            &template_id,
            input.is_active,
        )
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

async fn logout(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token = session_token(&headers)?;
    state.sessions.write().await.remove(token);
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn list_users(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserSummary>>, (StatusCode, Json<serde_json::Value>)> {
    let current_user = require_admin(&state, &headers).await?;
    let users = users::list_users(&state.database, &current_user.organization_id)
        .await
        .map_err(|message| error(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    Ok(Json(users))
}

#[utoipa::path(
    get,
    operation_id = "getDashboardSummary",
    path = "/api/v1/dashboard/summary",
    tag = "Dashboard",
    params(("months" = Option<u32>, Query, description = "Dashboard period: 3, 6, or 12 months")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization dashboard summary", body = dto::DashboardSummary),
        (status = 400, description = "Invalid dashboard period", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn dashboard_summary(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<dto::DashboardSummary>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let months = query.months.unwrap_or(6);
    if !matches!(months, 3 | 6 | 12) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Dashboard period must be 3, 6, or 12 months.",
        ));
    }
    let summary = aqardesk_infrastructure::dashboard::get_summary(
        &state.database,
        &user.organization_id,
        months,
    )
    .await
    .map_err(|message| error(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    Ok(Json(summary.into()))
}

async fn create_user(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserSummary>, (StatusCode, Json<serde_json::Value>)> {
    let current_user = require_admin(&state, &headers).await?;
    let user = users::create_user(
        &state.database,
        &current_user.organization_id,
        &current_user.id,
        &request.name,
        &request.email,
        &request.password,
        &request.role,
    )
    .await
    .map_err(|message| error(StatusCode::BAD_REQUEST, message))?;
    Ok(Json(user))
}

async fn update_user(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserSummary>, (StatusCode, Json<serde_json::Value>)> {
    let current_user = require_admin(&state, &headers).await?;
    if current_user.id == user_id && !request.is_active {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "You cannot deactivate the account currently in use.",
        ));
    }
    let user = users::set_user_active(
        &state.database,
        &current_user.organization_id,
        &current_user.id,
        &user_id,
        request.is_active,
    )
    .await
    .map_err(user_operation_error)?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    operation_id = "listProperties",
    path = "/api/v1/properties",
    tag = "Properties",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization properties", body = [dto::Property]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn list_properties(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::Property>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = PropertyService::new(SeaOrmPropertyRepository::new(&state.database));
    let items = service
        .list(&user.organization_id)
        .await
        .map_err(business_error)?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    operation_id = "createProperty",
    path = "/api/v1/properties",
    tag = "Properties",
    request_body = dto::PropertyInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Created property", body = dto::Property),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn create_property(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::PropertyInput>,
) -> Result<(StatusCode, Json<dto::Property>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = PropertyService::new(SeaOrmPropertyRepository::new(&state.database));
    let item = service
        .create(&user.organization_id, &user.id, input.into())
        .await
        .map_err(business_error)?;
    Ok((StatusCode::CREATED, Json(item.into())))
}

#[utoipa::path(
    put,
    operation_id = "updateProperty",
    path = "/api/v1/properties/{property_id}",
    tag = "Properties",
    request_body = dto::PropertyInput,
    params(("property_id" = String, Path, description = "Property identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated property", body = dto::Property),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Property not found", body = dto::ErrorResponse)
    )
)]
async fn update_property(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(property_id): Path<String>,
    Json(input): Json<dto::PropertyInput>,
) -> Result<Json<dto::Property>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = PropertyService::new(SeaOrmPropertyRepository::new(&state.database));
    let item = service
        .update(&user.organization_id, &user.id, &property_id, input.into())
        .await
        .map_err(business_error)?;
    Ok(Json(item.into()))
}

#[utoipa::path(
    delete,
    operation_id = "archiveProperty",
    path = "/api/v1/properties/{property_id}",
    tag = "Properties",
    params(("property_id" = String, Path, description = "Property identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Property archived without deleting history", body = dto::OperationResult),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Property not found", body = dto::ErrorResponse)
    )
)]
async fn archive_property(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(property_id): Path<String>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = PropertyService::new(SeaOrmPropertyRepository::new(&state.database));
    service
        .archive(&user.organization_id, &user.id, &property_id)
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

#[utoipa::path(
    get,
    operation_id = "listUnits",
    path = "/api/v1/units",
    tag = "Units",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization units", body = [dto::PropertyUnit]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn list_units(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::PropertyUnit>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = UnitService::new(SeaOrmUnitRepository::new(&state.database));
    let items = service
        .list(&user.organization_id, None)
        .await
        .map_err(business_error)?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    operation_id = "createUnit",
    path = "/api/v1/units",
    tag = "Units",
    request_body = dto::PropertyUnitInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Created unit", body = dto::PropertyUnit),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn create_unit(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::PropertyUnitInput>,
) -> Result<(StatusCode, Json<dto::PropertyUnit>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = UnitService::new(SeaOrmUnitRepository::new(&state.database));
    let item = service
        .create(&user.organization_id, input.into())
        .await
        .map_err(business_error)?;
    Ok((StatusCode::CREATED, Json(item.into())))
}

#[utoipa::path(
    put,
    operation_id = "updateUnit",
    path = "/api/v1/units/{unit_id}",
    tag = "Units",
    request_body = dto::PropertyUnitInput,
    params(("unit_id" = String, Path, description = "Unit identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated unit", body = dto::PropertyUnit),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Unit not found", body = dto::ErrorResponse)
    )
)]
async fn update_unit(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(unit_id): Path<String>,
    Json(input): Json<dto::PropertyUnitInput>,
) -> Result<Json<dto::PropertyUnit>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = UnitService::new(SeaOrmUnitRepository::new(&state.database));
    let item = service
        .update(&user.organization_id, &unit_id, input.into())
        .await
        .map_err(business_error)?;
    Ok(Json(item.into()))
}

#[utoipa::path(
    delete,
    operation_id = "archiveUnit",
    path = "/api/v1/units/{unit_id}",
    tag = "Units",
    params(("unit_id" = String, Path, description = "Unit identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Unit archived without deleting lease history", body = dto::OperationResult),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Unit not found", body = dto::ErrorResponse)
    )
)]
async fn archive_unit(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(unit_id): Path<String>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = UnitService::new(SeaOrmUnitRepository::new(&state.database));
    service
        .archive(&user.organization_id, &unit_id)
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

#[utoipa::path(
    get,
    operation_id = "listTenants",
    path = "/api/v1/tenants",
    tag = "Tenants",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization tenants", body = [dto::Tenant]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn list_tenants(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::Tenant>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = TenantService::new(SeaOrmTenantRepository::new(&state.database));
    let items = service
        .list(&user.organization_id)
        .await
        .map_err(business_error)?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    operation_id = "createTenant",
    path = "/api/v1/tenants",
    tag = "Tenants",
    request_body = dto::TenantInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Created tenant", body = dto::Tenant),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn create_tenant(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::TenantInput>,
) -> Result<(StatusCode, Json<dto::Tenant>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = TenantService::new(SeaOrmTenantRepository::new(&state.database));
    let item = service
        .create(&user.organization_id, input.into())
        .await
        .map_err(business_error)?;
    Ok((StatusCode::CREATED, Json(item.into())))
}

#[utoipa::path(
    put,
    operation_id = "updateTenant",
    path = "/api/v1/tenants/{tenant_id}",
    tag = "Tenants",
    request_body = dto::TenantInput,
    params(("tenant_id" = String, Path, description = "Tenant identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Updated tenant", body = dto::Tenant),
        (status = 400, description = "Invalid request", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Tenant not found", body = dto::ErrorResponse)
    )
)]
async fn update_tenant(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    Json(input): Json<dto::TenantInput>,
) -> Result<Json<dto::Tenant>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = TenantService::new(SeaOrmTenantRepository::new(&state.database));
    let item = service
        .update(&user.organization_id, &tenant_id, input.into())
        .await
        .map_err(business_error)?;
    Ok(Json(item.into()))
}

#[utoipa::path(
    delete,
    operation_id = "archiveTenant",
    path = "/api/v1/tenants/{tenant_id}",
    tag = "Tenants",
    params(("tenant_id" = String, Path, description = "Tenant identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Tenant archived without deleting history", body = dto::OperationResult),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Tenant not found", body = dto::ErrorResponse)
    )
)]
async fn archive_tenant(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = TenantService::new(SeaOrmTenantRepository::new(&state.database));
    service
        .archive(&user.organization_id, &tenant_id)
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

#[utoipa::path(
    get,
    operation_id = "listPayments",
    path = "/api/v1/payments",
    tag = "Payments",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization payment receipts", body = [dto::Payment]),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn list_payments(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<dto::Payment>>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(&state, &headers).await?;
    let service = PaymentService::new(SeaOrmPaymentRepository::new(&state.database));
    service
        .list(&user.organization_id)
        .await
        .map(|items| items.into_iter().map(Into::into).collect())
        .map(Json)
        .map_err(business_error)
}

#[utoipa::path(
    post,
    operation_id = "recordPayment",
    path = "/api/v1/payments",
    tag = "Payments",
    request_body = dto::PaymentInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Recorded payment receipt", body = dto::Payment),
        (status = 400, description = "Invalid payment", body = dto::ErrorResponse),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse)
    )
)]
async fn record_payment(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<dto::PaymentInput>,
) -> Result<(StatusCode, Json<dto::Payment>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = PaymentService::new(SeaOrmPaymentRepository::new(&state.database));
    let payment = service
        .record(&user.organization_id, &user.id, input.into())
        .await
        .map_err(business_error)?;
    Ok((StatusCode::CREATED, Json(payment.into())))
}

#[utoipa::path(
    patch,
    operation_id = "voidPayment",
    path = "/api/v1/payments/{payment_id}/void",
    tag = "Payments",
    params(("payment_id" = String, Path, description = "Payment identifier")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Voided payment receipt", body = dto::OperationResult),
        (status = 401, description = "Authentication required", body = dto::ErrorResponse),
        (status = 404, description = "Payment not found", body = dto::ErrorResponse)
    )
)]
async fn void_payment(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(payment_id): Path<String>,
) -> Result<Json<dto::OperationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_business_writer(&state, &headers).await?;
    let service = PaymentService::new(SeaOrmPaymentRepository::new(&state.database));
    service
        .void(&user.organization_id, &user.id, &payment_id)
        .await
        .map_err(business_error)?;
    Ok(Json(dto::OperationResult { ok: true }))
}

async fn create_session(state: &ApiState, user: UserSummary) -> LoginResponse {
    let token = Uuid::new_v4().to_string();
    state
        .sessions
        .write()
        .await
        .insert(token.clone(), user.clone());
    LoginResponse { user, token }
}

async fn require_admin(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<UserSummary, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(state, headers).await?;
    if user.role != "admin" {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Administrator access is required.",
        ));
    }
    Ok(user)
}

async fn require_business_writer(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<UserSummary, (StatusCode, Json<serde_json::Value>)> {
    let user = require_user(state, headers).await?;
    if user.role == "readonly" {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Read-only users cannot change business records.",
        ));
    }
    Ok(user)
}

async fn require_user(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<UserSummary, (StatusCode, Json<serde_json::Value>)> {
    let token = session_token(headers)?;
    let user = state
        .sessions
        .read()
        .await
        .get(token)
        .cloned()
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session expired. Sign in again."))?;
    users::get_active_user(&state.database, &user.id)
        .await
        .map_err(|message| error(StatusCode::INTERNAL_SERVER_ERROR, message))?
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session expired. Sign in again."))
}

fn business_error(value: AppError) -> (StatusCode, Json<serde_json::Value>) {
    match value {
        AppError::Validation(message) => error(StatusCode::BAD_REQUEST, message),
        AppError::NotFound(message) => error(StatusCode::NOT_FOUND, message),
        AppError::Persistence(message) => {
            tracing::error!(%message, "Business data persistence operation failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database operation failed.",
            )
        }
        AppError::OrganizationNotFound(id) => error(
            StatusCode::NOT_FOUND,
            format!("Organization {id:?} was not found."),
        ),
        AppError::UserNotFound(id) => {
            error(StatusCode::NOT_FOUND, format!("User {id:?} was not found."))
        }
    }
}

fn user_operation_error(message: String) -> (StatusCode, Json<serde_json::Value>) {
    if message.starts_with("Failed to") {
        tracing::error!(%message, "User operation failed");
        return error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A user operation failed.",
        );
    }
    let status = match message.as_str() {
        "User was not found." => StatusCode::NOT_FOUND,
        "Administrator access is required." => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_REQUEST,
    };
    error(status, message)
}

fn session_token(headers: &HeaderMap) -> Result<&str, (StatusCode, Json<serde_json::Value>)> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Sign in is required."))
}

fn read_connection_config(path: &PathBuf) -> Result<ConnectionConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("Could not read connection settings at {path:?}: {error}"))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("Connection settings are invalid: {error}"))
}

fn parse_args() -> Result<(PathBuf, SocketAddr), String> {
    let mut args = std::env::args().skip(1);
    let mut config_path = None;
    let mut bind = "127.0.0.1:3000".parse().expect("static socket address");
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--connection-config" => {
                config_path = Some(PathBuf::from(
                    args.next().ok_or("--connection-config requires a path")?,
                ));
            }
            "--bind" => {
                bind = args
                    .next()
                    .ok_or("--bind requires a socket address")?
                    .parse()
                    .map_err(|error| format!("Invalid --bind address: {error}"))?;
            }
            "--help" | "-h" => {
                println!("Usage: aqardesk-api --connection-config <path> [--bind <ip:port>]");
                std::process::exit(0);
            }
            other => return Err(format!("Unknown argument: {other}")),
        }
    }
    let path = config_path.ok_or(
        "Database settings are required. Pass --connection-config with the settings file created by the desktop app.",
    )?;
    Ok((path, bind))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("--export-openapi") {
        let output = args.next().ok_or("--export-openapi requires a path")?;
        std::fs::write(output, openapi::ApiDoc::openapi().to_pretty_json()?)?;
        return Ok(());
    }

    let (config_path, bind) = parse_args().map_err(std::io::Error::other)?;
    let config = read_connection_config(&config_path).map_err(std::io::Error::other)?;
    if config.mode == ConnectionMode::Api {
        return Err("The API server needs a local SQLite or direct PostgreSQL connection.".into());
    }
    let config_dir = config_path
        .parent()
        .ok_or("Connection settings path has no parent directory")?;
    let database_url = config
        .resolve_database_url(config_dir)
        .map_err(std::io::Error::other)?
        .ok_or("Database URL is missing")?;
    let database = connect_database(&database_url).await?;
    run_migrations(&database).await?;
    let state = ApiState {
        database,
        sessions: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/openapi.json", get(openapi_spec))
        .route("/api/v1/ready", get(readiness))
        .route("/api/v1/auth/setup", get(setup_status))
        .route("/api/v1/auth/bootstrap", post(bootstrap_admin))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/session", get(get_session))
        .route("/api/v1/auth/logout", post(logout))
        .route(
            "/api/v1/organizations/current",
            get(get_current_organization).put(update_current_organization),
        )
        .route("/api/v1/currencies", get(list_currencies))
        .route(
            "/api/v1/currencies/{currency_code}/organization-state",
            put(set_organization_currency_state),
        )
        .route(
            "/api/v1/users/me",
            get(get_my_profile).put(update_my_profile),
        )
        .route("/api/v1/audit-events", get(list_audit_events))
        .route("/api/v1/payments", get(list_payments).post(record_payment))
        .route("/api/v1/payments/{payment_id}/void", patch(void_payment))
        .route(
            "/api/v1/contract-templates",
            get(list_contract_templates).post(create_contract_template),
        )
        .route(
            "/api/v1/contract-templates/{template_id}",
            put(update_contract_template),
        )
        .route(
            "/api/v1/contract-templates/{template_id}/state",
            patch(set_contract_template_state),
        )
        .route("/api/v1/dashboard/summary", get(dashboard_summary))
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/{user_id}", patch(update_user))
        .route(
            "/api/v1/properties",
            get(list_properties).post(create_property),
        )
        .route(
            "/api/v1/properties/{property_id}",
            put(update_property).delete(archive_property),
        )
        .route("/api/v1/units", get(list_units).post(create_unit))
        .route(
            "/api/v1/units/{unit_id}",
            put(update_unit).delete(archive_unit),
        )
        .route("/api/v1/tenants", get(list_tenants).post(create_tenant))
        .route(
            "/api/v1/tenants/{tenant_id}",
            put(update_tenant).delete(archive_tenant),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("AqarDesk API listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use sea_orm::Database;

    #[test]
    fn generated_openapi_describes_business_routes_and_authentication() {
        let spec = serde_json::to_value(openapi::ApiDoc::openapi())
            .expect("serialize generated API specification");
        let paths = spec["paths"].as_object().expect("OpenAPI paths");

        assert!(paths.contains_key("/api/v1/properties"));
        assert!(paths.contains_key("/api/v1/properties/{property_id}"));
        assert!(paths.contains_key("/api/v1/organizations/current"));
        assert!(paths.contains_key("/api/v1/users/me"));
        assert!(paths.contains_key("/api/v1/audit-events"));
        assert!(paths.contains_key("/api/v1/contract-templates"));
        assert!(paths.contains_key("/api/v1/contract-templates/{template_id}"));
        assert_eq!(
            paths["/api/v1/organizations/current"]["put"]["operationId"],
            "updateCurrentOrganization"
        );
        assert_eq!(
            paths["/api/v1/users/me"]["put"]["operationId"],
            "updateMyProfile"
        );
        assert!(paths.contains_key("/api/v1/units"));
        assert!(paths.contains_key("/api/v1/tenants"));
        assert_eq!(
            paths["/api/v1/properties"]["post"]["operationId"],
            "createProperty"
        );
        assert_eq!(
            spec["components"]["schemas"]["PropertyType"]["enum"],
            serde_json::json!([
                "residential",
                "commercial",
                "mixed_use",
                "land",
                "other",
                "building",
                "house_villa",
                "investment",
                "warehouse",
                "yard",
                "apartment"
            ])
        );
        assert!(
            spec["components"]["securitySchemes"]["bearer_auth"].is_object(),
            "business endpoints must publish their bearer authentication scheme"
        );
        assert!(
            spec["components"]["schemas"]["PropertyInput"]["properties"]["organization_id"]
                .is_null(),
            "clients must not be able to set the organization scope"
        );
    }

    #[tokio::test]
    async fn session_endpoint_revalidates_and_logout_revokes_session() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory database");
        aqardesk_infrastructure::run_migrations(&database)
            .await
            .expect("apply migrations");
        let user = users::create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create initial administrator");
        let state = ApiState {
            database,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };
        let token = create_session(&state, user.clone()).await.token;
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("valid auth header"),
        );

        let organization = get_current_organization(State(state.clone()), headers.clone())
            .await
            .expect("load current organization")
            .0;
        assert_eq!(organization.name, "AqarDesk");

        let restored = get_session(State(state.clone()), headers.clone())
            .await
            .expect("restore active session")
            .0;
        assert_eq!(restored.id, user.id);
        assert_eq!(restored.organization_id, user.organization_id);

        let secondary_admin = users::create_user(
            &state.database,
            &user.organization_id,
            &user.id,
            "Second Admin",
            "second-admin@example.com",
            "another-secure-password",
            "admin",
        )
        .await
        .expect("create a second administrator");
        users::set_user_active(
            &state.database,
            &user.organization_id,
            &secondary_admin.id,
            &user.id,
            false,
        )
        .await
        .expect("deactivate administrator");
        assert!(
            get_session(State(state.clone()), headers.clone())
                .await
                .is_err()
        );

        let _response = logout(State(state.clone()), headers.clone())
            .await
            .expect("revoke session");
        assert!(get_session(State(state), headers).await.is_err());
    }

    #[tokio::test]
    async fn settings_endpoints_scope_profiles_and_restrict_administrative_operations() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory database");
        aqardesk_infrastructure::run_migrations(&database)
            .await
            .expect("apply migrations");
        let admin = users::create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create initial administrator");
        let staff = users::create_user(
            &database,
            &admin.organization_id,
            &admin.id,
            "Staff",
            "staff@example.com",
            "another-secure-password",
            "staff",
        )
        .await
        .expect("create staff user");
        let state = ApiState {
            database,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };
        let token = create_session(&state, staff.clone()).await.token;
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("valid auth header"),
        );

        let profile = get_my_profile(State(state.clone()), headers.clone())
            .await
            .expect("staff may read own profile")
            .0;
        assert_eq!(profile.id, staff.id);
        assert_eq!(profile.role, "staff");

        let update = update_my_profile(
            State(state.clone()),
            headers.clone(),
            Json(dto::ProfileSettingsInput {
                name: "Updated Staff".to_string(),
                email: "updated-staff@example.com".to_string(),
                current_password: None,
                new_password: None,
            }),
        )
        .await
        .expect("staff may update own profile")
        .0;
        assert_eq!(update.name, "Updated Staff");
        assert_eq!(update.role, "staff");

        let organization_update = update_current_organization(
            State(state.clone()),
            headers.clone(),
            Json(dto::OrganizationSettingsInput {
                name: "Not Allowed".to_string(),
                legal_name: None,
                default_currency: "SAR".to_string(),
                timezone: "Asia/Riyadh".to_string(),
            }),
        )
        .await
        .expect_err("staff cannot update organization settings");
        assert_eq!(organization_update.0, StatusCode::FORBIDDEN);

        let audit = list_audit_events(
            State(state),
            headers,
            Query(dto::AuditQuery {
                limit: Some(50),
                offset: Some(0),
            }),
        )
        .await
        .expect_err("staff cannot read organization audit events");
        assert_eq!(audit.0, StatusCode::FORBIDDEN);
    }
}
