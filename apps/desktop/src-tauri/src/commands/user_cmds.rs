use crate::connection::{AuthState, ConnectionState, DatabaseState, LoggedInUser};
use aqardesk_infrastructure::{
    settings::{self, ConnectionConfig, ConnectionMode},
    users::{self, UserSummary},
};
use reqwest::{Method, StatusCode};
use sea_orm::EntityTrait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct AuthSetupStatus {
    pub needs_admin: bool,
    pub current_user: Option<LoggedInUser>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedSession {
    user_id: String,
    api_token: Option<String>,
    connection_key: String,
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
    let config = get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let response: ApiSetupResponse =
            api_request(&config, Method::GET, "/api/v1/auth/setup", None, None).await?;
        restore_session(&config, &database, &auth).await?;
        return Ok(AuthSetupStatus {
            needs_admin: response.needs_admin,
            current_user: auth.current_user.read().await.clone(),
        });
    }

    let database_connection = database.get().await?;
    restore_session(&config, &database, &auth).await?;
    Ok(AuthSetupStatus {
        needs_admin: !users::has_users(&database_connection).await?,
        current_user: auth.current_user.read().await.clone(),
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
            users::create_initial_admin(&database, &organization_name, &name, &email, &password)
                .await?,
            None,
        )
    };
    establish_session(&auth, &config, user, token).await
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
        (
            users::authenticate(&database, &email, &password).await?,
            None,
        )
    };
    establish_session(&auth, &config, user, token).await
}

#[tauri::command]
pub async fn auth_logout(
    connection: State<'_, ConnectionState>,
    auth: State<'_, AuthState>,
) -> Result<(), String> {
    if let (Ok(config), Some(token)) =
        (get_config(&connection), auth.api_token.read().await.clone())
    {
        if config.mode == ConnectionMode::Api {
            if let Err(error) = api_request::<serde_json::Value>(
                &config,
                Method::POST,
                "/api/v1/auth/logout",
                Some(token),
                None,
            )
            .await
            {
                log::warn!("Could not revoke the remote API session during logout: {error}");
            }
        }
    }
    *auth.current_user.write().await = None;
    *auth.api_token.write().await = None;
    settings::remove_auth_session()?;
    Ok(())
}

#[tauri::command]
pub async fn users_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<UserSummary>, String> {
    require_admin(&connection, &database, &auth).await?;
    let config = get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let token = auth.api_token.read().await.clone();
        return api_request(&config, Method::GET, "/api/v1/users", token, None).await;
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
    require_admin(&connection, &database, &auth).await?;
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
    let current_user = require_admin(&connection, &database, &auth).await?;
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
        if target.access_role == "admin" && users::active_admin_count(&database).await? <= 1 {
            return Err("The last active administrator cannot be deactivated.".to_string());
        }
    }
    users::set_user_active(&database, &user_id, is_active).await
}

async fn establish_session(
    state: &AuthState,
    config: &ConnectionConfig,
    user: UserSummary,
    token: Option<String>,
) -> Result<LoggedInUser, String> {
    let user = to_logged_in_user(user);
    let session = PersistedSession {
        user_id: user.id.clone(),
        api_token: token.clone(),
        connection_key: connection_key(config)?,
    };
    let serialized = serde_json::to_string(&session)
        .map_err(|error| format!("Could not encode the session securely: {error}"))?;
    if let Err(error) = settings::store_auth_session(&serialized) {
        if config.mode == ConnectionMode::Api {
            if let Some(token) = token.as_ref() {
                if let Err(revoke_error) = api_request::<serde_json::Value>(
                    config,
                    Method::POST,
                    "/api/v1/auth/logout",
                    Some(token.clone()),
                    None,
                )
                .await
                {
                    log::warn!(
                        "Could not revoke the API session after secure persistence failed: {revoke_error}"
                    );
                }
            }
        }
        return Err(error);
    }
    *state.current_user.write().await = Some(user.clone());
    *state.api_token.write().await = token;
    Ok(user)
}

async fn restore_session(
    config: &ConnectionConfig,
    database: &DatabaseState,
    auth: &AuthState,
) -> Result<(), String> {
    let Some(stored) = settings::load_auth_session()? else {
        clear_auth_state(auth).await;
        return Ok(());
    };
    let session: PersistedSession = serde_json::from_str(&stored)
        .map_err(|error| format!("The saved session is invalid: {error}"))?;
    if session.connection_key != connection_key(config)? {
        discard_saved_session(auth).await?;
        return Ok(());
    }

    let user = if config.mode == ConnectionMode::Api {
        let Some(token) = session.api_token.as_deref() else {
            discard_saved_session(auth).await?;
            return Ok(());
        };
        match api_request::<UserSummary>(
            config,
            Method::GET,
            "/api/v1/auth/session",
            Some(token.to_string()),
            None,
        )
        .await
        {
            Ok(user) if user.id == session.user_id && user.is_active => user,
            Ok(_) => {
                discard_saved_session(auth).await?;
                return Ok(());
            }
            Err(error) if error == "The API session has expired. Sign in again." => {
                discard_saved_session(auth).await?;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    } else {
        let database = database.get().await?;
        let Some(user) = users::get_active_user(&database, &session.user_id).await? else {
            discard_saved_session(auth).await?;
            return Ok(());
        };
        user
    };

    let logged_in = to_logged_in_user(user);
    *auth.current_user.write().await = Some(logged_in);
    *auth.api_token.write().await = session.api_token;
    Ok(())
}

fn to_logged_in_user(user: UserSummary) -> LoggedInUser {
    LoggedInUser {
        id: user.id,
        organization_id: user.organization_id,
        name: user.name,
        email: user.email,
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at,
    }
}

async fn discard_saved_session(auth: &AuthState) -> Result<(), String> {
    clear_auth_state(auth).await;
    settings::remove_auth_session()
}

async fn clear_auth_state(auth: &AuthState) {
    *auth.current_user.write().await = None;
    *auth.api_token.write().await = None;
}

fn connection_key(config: &ConnectionConfig) -> Result<String, String> {
    serde_json::to_string(config)
        .map_err(|error| format!("Could not identify the active connection: {error}"))
}

pub(super) async fn require_active_user(
    connection: &ConnectionState,
    database: &DatabaseState,
    state: &AuthState,
) -> Result<LoggedInUser, String> {
    let mut user = state
        .current_user
        .read()
        .await
        .clone()
        .ok_or_else(|| "Sign in before managing users.".to_string())?;
    if connection_mode(connection)? != ConnectionMode::Api {
        let database = database.get().await?;
        let Some(active_user) = users::get_active_user(&database, &user.id).await? else {
            discard_saved_session(state).await?;
            return Err("The session is no longer valid. Sign in again.".to_string());
        };
        user = to_logged_in_user(active_user);
        *state.current_user.write().await = Some(user.clone());
    }
    if !user.is_active {
        return Err("The session is no longer valid. Sign in again.".to_string());
    }
    Ok(user)
}

async fn require_admin(
    connection: &ConnectionState,
    database: &DatabaseState,
    state: &AuthState,
) -> Result<LoggedInUser, String> {
    let user = require_active_user(connection, database, state).await?;
    if user.role != "admin" {
        return Err("Administrator access is required.".to_string());
    }
    Ok(user)
}

pub(super) fn connection_mode(state: &ConnectionState) -> Result<ConnectionMode, String> {
    Ok(get_config(state)?.mode)
}

pub(super) fn get_config(
    state: &ConnectionState,
) -> Result<aqardesk_infrastructure::settings::ConnectionConfig, String> {
    state
        .config
        .read()
        .map_err(|error| format!("Connection settings lock failed: {error}"))?
        .clone()
        .ok_or_else(|| "Choose and save a database connection first.".to_string())
}

pub(super) async fn api_request<T: DeserializeOwned>(
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
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.as_str())
                    .map(str::to_owned)
            })
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
