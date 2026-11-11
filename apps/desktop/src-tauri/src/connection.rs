use aqardesk_infrastructure::settings::{
    ApiConfig, ConnectionConfig, ConnectionMode, DirectConfig, LocalConfig,
};
use serde::Serialize;
use sea_orm::DatabaseConnection;
use std::{path::PathBuf, sync::RwLock};
use tokio::sync::RwLock as AsyncRwLock;

pub struct ConnectionState {
    pub app_data_dir: PathBuf,
    pub config: RwLock<Option<ConnectionConfig>>,
}

pub struct DatabaseState {
    pub connection: AsyncRwLock<Option<DatabaseConnection>>,
}

impl DatabaseState {
    pub async fn get(&self) -> Result<DatabaseConnection, String> {
        self.connection
            .read()
            .await
            .clone()
            .ok_or_else(|| "Connect to a database before continuing.".to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoggedInUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

pub struct AuthState {
    pub current_user: AsyncRwLock<Option<LoggedInUser>>,
    pub api_token: AsyncRwLock<Option<String>>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            current_user: AsyncRwLock::new(None),
            api_token: AsyncRwLock::new(None),
        }
    }
}

pub fn config_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("connection.json")
}

pub fn load_config(app_data_dir: &std::path::Path) -> Result<Option<ConnectionConfig>, String> {
    let path = config_path(app_data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read database settings: {error}"))?;
    serde_json::from_str(&contents)
        .map(Some)
        .map_err(|error| format!("Database settings are invalid: {error}"))
}

pub fn save_config(app_data_dir: &std::path::Path, config: &ConnectionConfig) -> Result<(), String> {
    std::fs::create_dir_all(app_data_dir)
        .map_err(|error| format!("Could not create application data folder: {error}"))?;
    let path = config_path(app_data_dir);
    let serialized = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Could not encode database settings: {error}"))?;
    std::fs::write(&path, serialized)
        .map_err(|error| format!("Could not save database settings: {error}"))?;
    Ok(())
}

pub fn make_config(
    app_data_dir: &std::path::Path,
    mode: &str,
    database_path: Option<String>,
    server_url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ssl_mode: Option<String>,
    existing: Option<&ConnectionConfig>,
) -> Result<ConnectionConfig, String> {
    match mode {
        "local" => {
            let path = database_path
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "aqardesk.sqlite".to_string());
            Ok(ConnectionConfig {
                mode: ConnectionMode::Local,
                local: Some(LocalConfig {
                    database_path: path,
                }),
                direct: None,
                api: None,
            })
        }
        "direct" => {
            let host = required(host, "PostgreSQL host")?;
            let database_name = required(database_name, "Database name")?;
            let username = required(username, "Database username")?;
            let port = port.unwrap_or(5432);
            if port == 0 {
                return Err("PostgreSQL port must be between 1 and 65535.".to_string());
            }
            if database_name.contains('/') || database_name.contains('\\') {
                return Err("Database name must not contain path separators.".to_string());
            }
            let credential_id = existing
                .and_then(|config| config.direct.as_ref())
                .map(|config| config.credential_id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            if let Some(password) = password.filter(|value| {
                !value.is_empty() && value != "••••••••" && value != "********"
            }) {
                aqardesk_infrastructure::settings::store_postgres_password(
                    &credential_id,
                    &password,
                )?;
            }
            let config = ConnectionConfig {
                mode: ConnectionMode::Direct,
                local: None,
                direct: Some(DirectConfig {
                    host,
                    port,
                    database_name,
                    username,
                    ssl_mode,
                    credential_id,
                }),
                api: None,
            };
            config.resolve_database_url(app_data_dir)?;
            Ok(config)
        }
        "api" => {
            let server_url = required(server_url, "API server URL")?;
            let config = ConnectionConfig {
                mode: ConnectionMode::Api,
                local: None,
                direct: None,
                api: Some(ApiConfig { server_url }),
            };
            config.api_url()?;
            Ok(config)
        }
        other => Err(format!("Unsupported connection mode: {other}")),
    }
}

fn required(value: Option<String>, label: &str) -> Result<String, String> {
    let value = value.unwrap_or_default();
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} is required."));
    }
    Ok(value.to_string())
}
