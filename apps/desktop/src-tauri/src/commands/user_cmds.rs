use crate::connection::{AuthState, ConnectionState, DatabaseState, LoggedInUser};
use aqardesk_infrastructure::{
    settings::ConnectionMode,
    users::{self, UserSummary},
};
use reqwest::{Method, StatusCode};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct AuthSetupStatus {
    pub needs_admin: bool,
    pub current_user: Option<LoggedInUser>,
}

#[derive(Debug, Deserialize)]
struct ApiLoginResponse {
    user: UserSummary,
    token: String,
}

#[derive(Debug, Deserialize)]
struct ApiSetupResponse {
    needs_admin: bool,
}

#[tauri::command]
pub async fn auth_get_setup_status(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<AuthSetupStatus, String> {
    let current_user = auth.current_user.read().await.clone();
    if connection_mode(&connection)? == ConnectionMode::Api {
        let config = get_config(&connection)?;
        let response: ApiSetupResponse = api_request(
            &config,
            Method::GET,
            "/api/v1/auth/setup",
            None,
            None,
        )
        .await?;
        return Ok(AuthSetupStatus {
            needs_admin: response.needs_admin,
            current_user,
        });
    }

    let database = database.get().await?;
    Ok(AuthSetupStatus {
        needs_admin: !users::has_users(&database).await?,
        current_user,
    })
}

#[tauri::command]
pub async fn auth_bootstrap_admin(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    organization_name: String,
    name: String,
    email: String,
    password: String,
) -> Result<LoggedInUser, String> {
    let config = get_config(&connection)?;
    let (user, token) = if config.mode == ConnectionMode::Api {
        let response: ApiLoginResponse = api_request(
            &config,
            Method::POST,
            "/api/v1/auth/bootstrap",
            None,
            Some(serde_json::json!({
                "organization_name": organization_name,
                "name": name,
                "email": email,
                "password": password
            })),
        )
        .await?;
        (response.user, Some(response.token))
    } else {
        let database = database.get().await?;
        (
            users::create_initial_admin(
                &database,
                &organization_name,
                &name,
                &email,
                &password,
            )
            .await?,
            None,
        )
    };
    establish_session(&auth, user, token).await
}

#[tauri::command]
pub async fn auth_login(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    email: String,
    password: String,
) -> Result<LoggedInUser, String> {
    let config = get_config(&connection)?;
    let (user, token) = if config.mode == ConnectionMode::Api {
        let response: ApiLoginResponse = api_request(
            &config,
            Method::POST,
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "email": email, "password": password })),
        )
        .await?;
        (response.user, Some(response.token))
    } else {
        let database = database.get().await?;
        (users::authenticate(&database, &email, &password).await?, None)
    };
    establish_session(&auth, user, token).await
}

#[tauri::command]
pub async fn auth_logout(auth: State<'_, AuthState>) -> Result<(), String> {
    *auth.current_user.write().await = None;
    *auth.api_token.write().await = None;
    Ok(())
}

#[tauri::command]
pub async fn users_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<UserSummary>, String> {
    require_admin(&auth).await?;
    let config = get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let token = auth.api_token.read().await.clone();
        return api_request(
            &config,
            Method::GET,
            "/api/v1/users",
            token,
            None,
        )
        .await;
    }
    users::list_users(&database.get().await?).await
}

#[tauri::command]
pub async fn users_create(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    name: String,
    email: String,
    password: String,
    role: String,
) -> Result<UserSummary, String> {
    require_admin(&auth).await?;
    let config = get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let token = auth.api_token.read().await.clone();
        return api_request(
            &config,
            Method::POST,
            "/api/v1/users",
            token,
            Some(serde_json::json!({ "name": name, "email": email, "password": password, "role": role })),
        )
        .await;
    }
    users::create_user(&database.get().await?, &name, &email, &password, &role).await
}

#[tauri::command]
pub async fn users_set_active(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    user_id: String,
    is_active: bool,
) -> Result<UserSummary, String> {
    let current_user = require_admin(&auth).await?;
    if current_user.id == user_id && !is_active {
        return Err("You cannot deactivate the account currently in use.".to_string());
    }

    let config = get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let token = auth.api_token.read().await.clone();
        return api_request(
            &config,
            Method::PATCH,
            &format!("/api/v1/users/{user_id}"),
            token,
            Some(serde_json::json!({ "is_active": is_active })),
        )
        .await;
    }
    let database = database.get().await?;
    if !is_active {
        let target = aqardesk_infrastructure::models::user::Entity::find_by_id(&user_id)
            .one(&database)
            .await
            .map_err(|error| format!("Could not inspect user account: {error}"))?
            .ok_or_else(|| "User was not found.".to_string())?;
        if target.role == "admin" && users::active_admin_count(&database).await? <= 1 {
            return Err("The last active administrator cannot be deactivated.".to_string());
        }
    }
    users::set_user_active(&database, &user_id, is_active).await
}

async fn establish_session(
    state: &AuthState,
    user: UserSummary,
    token: Option<String>,
) -> Result<LoggedInUser, String> {
    let user = LoggedInUser {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at,
    };
    *state.current_user.write().await = Some(user.clone());
    *state.api_token.write().await = token;
    Ok(user)
}

async fn require_admin(state: &AuthState) -> Result<LoggedInUser, String> {
    let user = state
        .current_user
        .read()
        .await
        .clone()
        .ok_or_else(|| "Sign in before managing users.".to_string())?;
    if !user.is_active || user.role != "admin" {
        return Err("Administrator access is required.".to_string());
    }
    Ok(user)
}

fn get_config(state: &ConnectionState) -> Result<aqardesk_infrastructure::settings::ConnectionConfig, String> {
    state
        .config
        .read()
        .map_err(|error| format!("Connection settings lock failed: {error}"))?
        .clone()
        .ok_or_else(|| "Choose and save a database connection first.".to_string())
}

fn connection_mode(state: &ConnectionState) -> Result<ConnectionMode, String> {
    Ok(get_config(state)?.mode)
}

async fn api_request<T: DeserializeOwned>(
    config: &aqardesk_infrastructure::settings::ConnectionConfig,
    method: Method,
    path: &str,
    token: Option<String>,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let base = config.api_url()?.trim_end_matches('/');
    let mut request = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| format!("Could not prepare API request: {error}"))?
        .request(method, format!("{base}{path}"));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("API request failed: {error}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let detail = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.get("error").and_then(|error| error.as_str()).map(str::to_owned))
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(if status == StatusCode::UNAUTHORIZED {
            "The API session has expired. Sign in again.".to_string()
        } else {
            detail
        });
    }
    response
        .json()
        .await
        .map_err(|error| format!("API returned an invalid response: {error}"))
}
