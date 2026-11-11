use aqardesk_infrastructure::{
    connect_database, run_migrations,
    services::{properties, tenants, units},
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
    .map_err(|message| error(StatusCode::BAD_REQUEST, message))?;
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
    require_admin(&state, &headers).await?;
    let users = users::list_users(&state.database)
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
    require_admin(&state, &headers).await?;
    let user = users::create_user(
        &state.database,
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
    let user = users::set_user_active(&state.database, &user_id, request.is_active)
        .await
        .map_err(|message| error(StatusCode::NOT_FOUND, message))?;
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
    let items = properties::list_properties(&state.database, &user.organization_id)
        .await
        .map_err(|message| business_error(message))?;
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
    let item = properties::create_property(
        &state.database,
        &user.organization_id,
        &user.id,
        input.into(),
    )
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
    let item = properties::update_property(
        &state.database,
        &user.organization_id,
        &user.id,
        &property_id,
        input.into(),
    )
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
    properties::archive_property(&state.database, &user.organization_id, &property_id)
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
    let items = units::list_units(&state.database, &user.organization_id, None)
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
    let item = units::create_unit(&state.database, &user.organization_id, input.into())
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
    let item = units::update_unit(
        &state.database,
        &user.organization_id,
        &unit_id,
        input.into(),
    )
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
    units::archive_unit(&state.database, &user.organization_id, &unit_id)
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
    let items = tenants::list_tenants(&state.database, &user.organization_id)
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
    let item = tenants::create_tenant(&state.database, &user.organization_id, input.into())
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
    let item = tenants::update_tenant(
        &state.database,
        &user.organization_id,
        &tenant_id,
        input.into(),
    )
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
    tenants::archive_tenant(&state.database, &user.organization_id, &tenant_id)
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

fn business_error(message: String) -> (StatusCode, Json<serde_json::Value>) {
    if message.contains("not found") {
        error(StatusCode::NOT_FOUND, message)
    } else if message.starts_with("Failed to") {
        error(StatusCode::INTERNAL_SERVER_ERROR, message)
    } else {
        error(StatusCode::BAD_REQUEST, message)
    }
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
        assert!(paths.contains_key("/api/v1/units"));
        assert!(paths.contains_key("/api/v1/tenants"));
        assert_eq!(
            paths["/api/v1/properties"]["post"]["operationId"],
            "createProperty"
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

        let restored = get_session(State(state.clone()), headers.clone())
            .await
            .expect("restore active session")
            .0;
        assert_eq!(restored.id, user.id);
        assert_eq!(restored.organization_id, user.organization_id);

        users::set_user_active(&state.database, &user.id, false)
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
}
