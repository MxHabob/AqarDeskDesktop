use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use url::Url;

const SECRET_SERVICE: &str = "com.aqardesk.desktop.database";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    Local,
    Direct,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub mode: ConnectionMode,
    #[serde(default)]
    pub local: Option<LocalConfig>,
    #[serde(default)]
    pub direct: Option<DirectConfig>,
    #[serde(default)]
    pub api: Option<ApiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub ssl_mode: Option<String>,
    pub credential_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub server_url: String,
}

impl ConnectionConfig {
    pub fn resolve_database_url(&self, config_dir: &Path) -> Result<Option<String>, String> {
        match self.mode {
            ConnectionMode::Local => {
                let local = self
                    .local
                    .as_ref()
                    .ok_or_else(|| "Local database settings are missing.".to_string())?;
                let path = PathBuf::from(&local.database_path);
                let path = if path.is_absolute() {
                    path
                } else {
                    config_dir.join(path)
                };
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| {
                        format!("Could not create the local database folder: {error}")
                    })?;
                }
                let path = path.to_string_lossy().replace('\\', "/");
                Ok(Some(format!("sqlite://{path}?mode=rwc")))
            }
            ConnectionMode::Direct => {
                let direct = self
                    .direct
                    .as_ref()
                    .ok_or_else(|| "PostgreSQL connection settings are missing.".to_string())?;
                let password = Entry::new(SECRET_SERVICE, &direct.credential_id)
                    .map_err(|error| format!("Could not access the OS credential store: {error}"))?
                    .get_password()
                    .map_err(|error| match error {
                        KeyringError::NoEntry => {
                            "PostgreSQL password is not present in the OS credential store."
                                .to_string()
                        }
                        other => format!("Could not read the PostgreSQL password: {other}"),
                    })?;
                let mut url = Url::parse("postgres://localhost")
                    .map_err(|error| format!("Could not construct PostgreSQL URL: {error}"))?;
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
                    if !matches!(ssl_mode, "disable" | "prefer" | "require" | "verify-ca" | "verify-full") {
                        return Err("PostgreSQL SSL mode is invalid.".to_string());
                    }
                    url.query_pairs_mut().append_pair("sslmode", ssl_mode);
                }
                Ok(Some(url.to_string()))
            }
            ConnectionMode::Api => Ok(None),
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

pub fn store_postgres_password(credential_id: &str, password: &str) -> Result<(), String> {
    Entry::new(SECRET_SERVICE, credential_id)
        .map_err(|error| format!("Could not access the OS credential store: {error}"))?
        .set_password(password)
        .map_err(|error| format!("Could not save PostgreSQL password securely: {error}"))
}

pub fn remove_postgres_password(credential_id: &str) -> Result<(), String> {
    let entry = Entry::new(SECRET_SERVICE, credential_id)
        .map_err(|error| format!("Could not access the OS credential store: {error}"))?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => Err(format!("Could not remove PostgreSQL password securely: {error}")),
    }
}
