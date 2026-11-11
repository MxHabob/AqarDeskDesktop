use aqardesk_platform::infrastructure::Database;
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use url::Url;

const DATABASE_CREDENTIAL_SERVICE: &str = "com.aqardesk.desktop.database";
const SESSION_CREDENTIAL_SERVICE: &str = "com.aqardesk.desktop.session";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    Local,
    Direct,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalConnection {
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectConnection {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub ssl_mode: Option<String>,
    pub credential_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiConnection {
    pub server_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub mode: ConnectionMode,
    pub local: Option<LocalConnection>,
    pub direct: Option<DirectConnection>,
    pub api: Option<ApiConnection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionView {
    pub mode: ConnectionMode,
    pub local: Option<LocalConnection>,
    pub direct: Option<DirectConnectionView>,
    pub api: Option<ApiConnection>,
}

#[derive(Debug, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub mode: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectConnectionView {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub ssl_mode: Option<String>,
}

impl From<&ConnectionConfig> for ConnectionView {
    fn from(config: &ConnectionConfig) -> Self {
        Self {
            mode: config.mode,
            local: config.local.clone(),
            direct: config.direct.as_ref().map(|direct| DirectConnectionView {
                host: direct.host.clone(),
                port: direct.port,
                database_name: direct.database_name.clone(),
                username: direct.username.clone(),
                ssl_mode: direct.ssl_mode.clone(),
            }),
            api: config.api.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub mode: ConnectionMode,
    pub database_path: Option<String>,
    pub server_url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub user_id: String,
    pub api_token: Option<String>,
    pub connection_key: String,
}

pub struct PlatformState {
    pub app_data_dir: PathBuf,
    pub config: RwLock<Option<ConnectionConfig>>,
    pub database: RwLock<Option<Arc<Database>>>,
    pub current_user: RwLock<Option<aqardesk_platform::domain::UserAccount>>,
    pub api_token: RwLock<Option<String>>,
    pub http: reqwest::Client,
}

impl PlatformState {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| {
                tracing::error!(%error, "Could not initialize the desktop HTTP client");
                "The network client could not be initialized.".to_string()
            })?;
        Ok(Self {
            config: RwLock::new(load_config(&app_data_dir)?),
            app_data_dir,
            database: RwLock::new(None),
            current_user: RwLock::new(None),
            api_token: RwLock::new(None),
            http,
        })
    }

    pub async fn config(&self) -> Result<ConnectionConfig, String> {
        self.config
            .read()
            .await
            .clone()
            .ok_or_else(|| "Choose and save a database connection first.".to_string())
    }

    pub async fn database(&self) -> Result<Arc<Database>, String> {
        self.database
            .read()
            .await
            .clone()
            .ok_or_else(|| "Connect to a database before continuing.".to_string())
    }

    pub async fn connect(&self) -> Result<String, String> {
        let config = self.config().await?;
        match config.mode {
            ConnectionMode::Api => config.api_url().map(str::to_string),
            ConnectionMode::Local | ConnectionMode::Direct => {
                let url = config.database_url(&self.app_data_dir)?;
                let database = Database::connect(&url)
                    .await
                    .map_err(|error| format!("Could not connect to database: {error}"))?;
                aqardesk_platform::migration::initialize(database.connection())
                    .await
                    .map_err(|error| {
                        tracing::error!(%error, "Platform database initialization failed");
                        "Could not initialize the database schema.".to_string()
                    })?;
                *self.database.write().await = Some(Arc::new(database));
                Ok("local".to_string())
            }
        }
    }
}

impl ConnectionConfig {
    fn local_database_path(&self, app_data_dir: &Path) -> Result<PathBuf, String> {
        let local = self
            .local
            .as_ref()
            .ok_or_else(|| "Local database settings are missing.".to_string())?;
        let path = PathBuf::from(&local.database_path);
        Ok(if path.is_absolute() {
            path
        } else {
            app_data_dir.join(path)
        })
    }

    pub fn database_url(&self, app_data_dir: &Path) -> Result<String, String> {
        match self.mode {
            ConnectionMode::Local => {
                let path = self.local_database_path(app_data_dir)?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| {
                        tracing::error!(%error, "Could not create database directory");
                        "Could not prepare the local database folder.".to_string()
                    })?;
                }
                Ok(format!(
                    "sqlite://{}?mode=rwc",
                    path.to_string_lossy().replace('\\', "/")
                ))
            }
            ConnectionMode::Direct => {
                let direct = self
                    .direct
                    .as_ref()
                    .ok_or_else(|| "PostgreSQL connection settings are missing.".to_string())?;
                let password = Entry::new(DATABASE_CREDENTIAL_SERVICE, &direct.credential_id)
                    .map_err(|error| {
                        tracing::error!(%error, "Could not access PostgreSQL credential");
                        "Could not access the operating-system credential store.".to_string()
                    })?
                    .get_password()
                    .map_err(|error| match error {
                        KeyringError::NoEntry => {
                            "PostgreSQL password is not present in the credential store."
                                .to_string()
                        }
                        error => {
                            tracing::error!(%error, "Could not read PostgreSQL credential");
                            "Could not read the PostgreSQL password.".to_string()
                        }
                    })?;
                let mut url = Url::parse("postgres://localhost")
                    .map_err(|_| "Could not prepare PostgreSQL connection.".to_string())?;
                url.set_host(Some(&direct.host))
                    .map_err(|_| "PostgreSQL host is invalid.".to_string())?;
                url.set_port(Some(direct.port))
                    .map_err(|_| "PostgreSQL port is invalid.".to_string())?;
                url.set_username(&direct.username)
                    .map_err(|_| "PostgreSQL username is invalid.".to_string())?;
                url.set_password(Some(&password))
                    .map_err(|_| "PostgreSQL password is invalid.".to_string())?;
                url.set_path(&format!("/{}", direct.database_name));
                if let Some(ssl_mode) = direct.ssl_mode.as_deref() {
                    if !matches!(
                        ssl_mode,
                        "disable" | "prefer" | "require" | "verify-ca" | "verify-full"
                    ) {
                        return Err("PostgreSQL SSL mode is invalid.".to_string());
                    }
                    url.query_pairs_mut().append_pair("sslmode", ssl_mode);
                }
                Ok(url.to_string())
            }
            ConnectionMode::Api => Err("The selected connection uses an API server.".to_string()),
        }
    }

    pub fn api_url(&self) -> Result<&str, String> {
        if self.mode != ConnectionMode::Api {
            return Err("The selected connection is not an API server.".to_string());
        }
        let url = self
            .api
            .as_ref()
            .ok_or_else(|| "API server URL is missing.".to_string())?
            .server_url
            .trim();
        let parsed = Url::parse(url).map_err(|_| "API server URL is invalid.".to_string())?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err("API server URL must be an HTTP or HTTPS address.".to_string());
        }
        Ok(url)
    }
}

pub fn make_config(
    input: ConnectionInput,
    existing: Option<&ConnectionConfig>,
    app_data_dir: &Path,
) -> Result<ConnectionConfig, String> {
    match input.mode {
        ConnectionMode::Local => {
            let database_path = input
                .database_path
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "aqardesk.sqlite".to_string());
            Ok(ConnectionConfig {
                mode: ConnectionMode::Local,
                local: Some(LocalConnection { database_path }),
                direct: None,
                api: None,
            })
        }
        ConnectionMode::Direct => {
            let host = required(input.host, "PostgreSQL host")?;
            let database_name = required(input.database_name, "Database name")?;
            let username = required(input.username, "Database username")?;
            if database_name.contains('/') || database_name.contains('\\') {
                return Err("Database name must not contain path separators.".to_string());
            }
            let port = input.port.unwrap_or(5432);
            if port == 0 {
                return Err("PostgreSQL port must be between 1 and 65535.".to_string());
            }
            let credential_id = existing
                .and_then(|config| config.direct.as_ref())
                .map(|config| config.credential_id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            if let Some(password) = input
                .password
                .filter(|value| !value.is_empty() && value != "••••••••" && value != "********")
            {
                Entry::new(DATABASE_CREDENTIAL_SERVICE, &credential_id)
                    .map_err(|error| {
                        tracing::error!(%error, "Could not access database credential store");
                        "Could not access the operating-system credential store.".to_string()
                    })?
                    .set_password(&password)
                    .map_err(|error| {
                        tracing::error!(%error, "Could not save database credential");
                        "Could not save the database password securely.".to_string()
                    })?;
            }
            let config = ConnectionConfig {
                mode: ConnectionMode::Direct,
                local: None,
                direct: Some(DirectConnection {
                    host,
                    port,
                    database_name,
                    username,
                    ssl_mode: input.ssl_mode,
                    credential_id,
                }),
                api: None,
            };
            config.database_url(app_data_dir)?;
            Ok(config)
        }
        ConnectionMode::Api => {
            let config = ConnectionConfig {
                mode: ConnectionMode::Api,
                local: None,
                direct: None,
                api: Some(ApiConnection {
                    server_url: required(input.server_url, "API server URL")?,
                }),
            };
            config.api_url()?;
            Ok(config)
        }
    }
}

pub fn save_config(app_data_dir: &Path, config: &ConnectionConfig) -> Result<(), String> {
    let path = app_data_dir.join("connection.json");
    let contents = serde_json::to_vec_pretty(config).map_err(|error| {
        tracing::error!(%error, "Could not serialize connection configuration");
        "Could not save connection settings.".to_string()
    })?;
    std::fs::create_dir_all(app_data_dir).map_err(|error| {
        tracing::error!(%error, "Could not create application data directory");
        "Could not prepare application settings storage.".to_string()
    })?;
    std::fs::write(path, contents).map_err(|error| {
        tracing::error!(%error, "Could not save connection configuration");
        "Could not save connection settings.".to_string()
    })
}

pub fn load_config(app_data_dir: &Path) -> Result<Option<ConnectionConfig>, String> {
    let path = app_data_dir.join("connection.json");
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read(path).map_err(|error| {
        tracing::error!(%error, "Could not read connection configuration");
        "Could not read connection settings.".to_string()
    })?;
    serde_json::from_slice(&contents)
        .map(Some)
        .map_err(|error| {
            tracing::error!(%error, "Connection configuration is invalid");
            "Connection settings are invalid.".to_string()
        })
}

pub fn load_session() -> Result<Option<SavedSession>, String> {
    let entry = Entry::new(SESSION_CREDENTIAL_SERVICE, "active").map_err(|error| {
        tracing::error!(%error, "Could not access session credential");
        "Could not access the operating-system credential store.".to_string()
    })?;
    match entry.get_password() {
        Ok(value) => serde_json::from_str(&value).map(Some).map_err(|error| {
            tracing::error!(%error, "Saved session is invalid");
            "The saved session is invalid.".to_string()
        }),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => {
            tracing::error!(%error, "Could not read saved session");
            Err("Could not read the saved session.".to_string())
        }
    }
}

pub fn save_session(session: &SavedSession) -> Result<(), String> {
    let value = serde_json::to_string(session).map_err(|error| {
        tracing::error!(%error, "Could not serialize saved session");
        "Could not save the session securely.".to_string()
    })?;
    Entry::new(SESSION_CREDENTIAL_SERVICE, "active")
        .and_then(|entry| entry.set_password(&value))
        .map_err(|error| {
            tracing::error!(%error, "Could not persist session credential");
            "Could not save the session securely.".to_string()
        })
}

pub fn clear_session() -> Result<(), String> {
    let entry = Entry::new(SESSION_CREDENTIAL_SERVICE, "active").map_err(|error| {
        tracing::error!(%error, "Could not access saved session");
        "Could not access the operating-system credential store.".to_string()
    })?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => {
            tracing::error!(%error, "Could not delete saved session");
            Err("Could not clear the saved session.".to_string())
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_connection_settings_remain_readable_without_exposing_credentials() {
        let config: ConnectionConfig = serde_json::from_str(
            r#"{
                "mode": "direct",
                "local": null,
                "direct": {
                    "host": "db.example.test",
                    "port": 5432,
                    "database_name": "aqardesk",
                    "username": "desktop",
                    "ssl_mode": "require",
                    "credential_id": "credential-ref"
                },
                "api": null
            }"#,
        )
        .expect("deserialize existing connection config");

        assert_eq!(config.mode, ConnectionMode::Direct);
        let view = serde_json::to_value(ConnectionView::from(&config))
            .expect("serialize frontend-safe connection view");
        assert!(view["direct"].get("credential_id").is_none());
        assert!(
            serde_json::to_string(&config)
                .expect("serialize persisted connection settings")
                .contains("credential-ref")
        );
        assert!(
            !serde_json::to_string(&config)
                .expect("serialize persisted connection settings")
                .contains("password")
        );
    }

    #[test]
    fn api_connection_requires_a_valid_http_endpoint() {
        let config = ConnectionConfig {
            mode: ConnectionMode::Api,
            local: None,
            direct: None,
            api: Some(ApiConnection {
                server_url: "file:///etc/passwd".to_string(),
            }),
        };
        assert!(config.api_url().is_err());
    }

    #[test]
    fn relative_local_database_paths_resolve_inside_application_data() {
        let config = ConnectionConfig {
            mode: ConnectionMode::Local,
            local: Some(LocalConnection {
                database_path: "db/aqardesk.sqlite".to_string(),
            }),
            direct: None,
            api: None,
        };
        let app_data_dir = Path::new("C:\\AqarDesk\\data");
        assert_eq!(
            config
                .local_database_path(app_data_dir)
                .expect("resolve local path"),
            PathBuf::from("C:\\AqarDesk\\data\\db\\aqardesk.sqlite")
        );
    }
}
