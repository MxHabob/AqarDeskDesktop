use crate::connection::{
    AuthState, ConnectionState, DatabaseState, make_config, save_config,
};
use aqardesk_infrastructure::{
    connect_database, run_migrations,
    settings::{ConnectionConfig, ConnectionMode},
};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub mode: String,
    pub message: String,
}

#[tauri::command]
pub async fn connection_get_config(
    state: State<'_, ConnectionState>,
) -> Result<Option<ConnectionConfig>, String> {
    state
        .config
        .read()
        .map_err(|error| format!("Connection settings lock failed: {error}"))
        .map(|config| config.clone())
}

#[tauri::command]
pub async fn connection_set_config(
    state: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    mode: String,
    database_path: Option<String>,
    server_url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ssl_mode: Option<String>,
) -> Result<ConnectionConfig, String> {
    let existing = state
        .config
        .read()
        .map_err(|error| format!("Connection settings lock failed: {error}"))?
        .clone();
    let config = make_config(
        &state.app_data_dir,
        &mode,
        database_path,
        server_url,
        host,
        port,
        database_name,
        username,
        password,
        ssl_mode,
        existing.as_ref(),
    )?;
    save_config(&state.app_data_dir, &config)?;
    if let Some(old_credential_id) = existing
        .as_ref()
        .and_then(|existing| existing.direct.as_ref())
        .map(|direct| direct.credential_id.as_str())
    {
        let new_credential_id = config
            .direct
            .as_ref()
            .map(|direct| direct.credential_id.as_str());
        if new_credential_id != Some(old_credential_id) {
            aqardesk_infrastructure::settings::remove_postgres_password(old_credential_id)?;
        }
    }

    *state
        .config
        .write()
        .map_err(|error| format!("Connection settings lock failed: {error}"))? =
        Some(config.clone());
    *database.connection.write().await = None;
    *auth.current_user.write().await = None;
    *auth.api_token.write().await = None;
    Ok(config)
}

#[tauri::command]
pub async fn connection_check_api(server_url: String) -> Result<ConnectionStatus, String> {
    let base = server_url.trim().trim_end_matches('/');
    let parsed = url::Url::parse(base).map_err(|_| "API server URL is invalid.".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("API server URL must be an HTTP or HTTPS address.".to_string());
    }

    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|error| format!("Could not prepare API health check: {error}"))?
        .get(format!("{base}/api/v1/health"))
        .send()
        .await
        .map_err(|error| format!("Could not reach API server: {error}"))?;
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
pub async fn connection_connect(
    state: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<ConnectionStatus, String> {
    let config = state
        .config
        .read()
        .map_err(|error| format!("Connection settings lock failed: {error}"))?
        .clone()
        .ok_or_else(|| "Choose and save a database connection first.".to_string())?;

    match config.mode {
        ConnectionMode::Api => {
            let url = config.api_url()?.trim_end_matches('/').to_string();
            let result = connection_check_api(url).await?;
            *database.connection.write().await = None;
            *auth.current_user.write().await = None;
            *auth.api_token.write().await = None;
            Ok(result)
        }
        ConnectionMode::Local | ConnectionMode::Direct => {
            let url = config
                .resolve_database_url(&state.app_data_dir)?
                .ok_or_else(|| "Database connection settings are missing.".to_string())?;
            let connection = connect_database(&url)
                .await
                .map_err(|error| format!("Could not connect to database: {error}"))?;
            run_migrations(&connection)
                .await
                .map_err(|error| format!("Could not prepare database schema: {error}"))?;
            *database.connection.write().await = Some(connection);
            *auth.current_user.write().await = None;
            *auth.api_token.write().await = None;
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
    }
}
