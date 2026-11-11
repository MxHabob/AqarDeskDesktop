use aqardesk_infrastructure::{
    connect_database, run_migrations,
    settings::{ConnectionConfig, ConnectionMode},
    users::{self, UserSummary},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct ApiState {
    database: DatabaseConnection,
    sessions: Arc<RwLock<HashMap<String, UserSummary>>>,
}

#[derive(Debug, Serialize)]
struct HealthPayload {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
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

async fn health() -> Json<HealthPayload> {
    Json(HealthPayload {
        status: "ok",
        service: "aqardesk-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

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
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Sign in is required."))?;
    let user = state
        .sessions
        .read()
        .await
        .get(token)
        .cloned()
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Session expired. Sign in again."))?;
    if !user.is_active || user.role != "admin" {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Administrator access is required.",
        ));
    }
    Ok(user)
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
                    args.next()
                        .ok_or("--connection-config requires a path")?,
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
        .route("/api/v1/ready", get(readiness))
        .route("/api/v1/auth/setup", get(setup_status))
        .route("/api/v1/auth/bootstrap", post(bootstrap_admin))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/{user_id}", patch(update_user))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("AqarDesk API listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}
