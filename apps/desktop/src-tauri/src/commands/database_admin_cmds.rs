use crate::{
    platform_commands,
    platform_runtime::{self, ConnectionMode, PlatformState},
};
use aqardesk_backend::infrastructure::Database;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;
use uuid::Uuid;

const DATABASE_BACKUP_DIR: &str = "database-backups";

#[derive(Debug, Serialize)]
pub struct DatabaseAdminStatus {
    pub mode: ConnectionMode,
    pub database_path: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub modified_at_unix: Option<u64>,
    pub supports_file_management: bool,
    pub remote_database: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseBackup {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at_unix: u64,
}

#[derive(Debug, Serialize)]
pub struct DatabaseOperationResult {
    pub path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn database_admin_get_status(
    state: State<'_, PlatformState>,
) -> Result<DatabaseAdminStatus, String> {
    let config = state.config().await?;
    if config.mode == ConnectionMode::Api {
        return Ok(DatabaseAdminStatus {
            mode: config.mode,
            database_path: None,
            file_size_bytes: None,
            modified_at_unix: None,
            supports_file_management: false,
            remote_database: config
                .api
                .as_ref()
                .map(|connection| connection.server_url.clone()),
        });
    }
    require_admin(&state).await?;
    if config.mode != ConnectionMode::Local {
        let remote_database = match config.mode {
            ConnectionMode::Direct => config
                .direct
                .as_ref()
                .map(|connection| format!("{} · {}", connection.host, connection.database_name)),
            ConnectionMode::Api => config
                .api
                .as_ref()
                .map(|connection| connection.server_url.clone()),
            ConnectionMode::Local => None,
        };
        return Ok(DatabaseAdminStatus {
            mode: config.mode,
            database_path: None,
            file_size_bytes: None,
            modified_at_unix: None,
            supports_file_management: false,
            remote_database,
        });
    }

    let path = config.local_database_path(&state.app_data_dir)?;
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            tracing::error!(%error, path = %path.display(), "Could not inspect local database");
            return Err("Could not read the local database file information.".to_string());
        }
    };
    let modified_at_unix = match metadata.as_ref() {
        Some(metadata) => Some(
            metadata
                .modified()
                .map_err(|error| {
                    tracing::error!(%error, path = %path.display(), "Could not read local database timestamp");
                    "Could not read the local database modification time.".to_string()
                })?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| {
                    tracing::error!(%error, path = %path.display(), "Local database timestamp is invalid");
                    "The local database modification time is invalid.".to_string()
                })?
                .as_secs(),
        ),
        None => None,
    };
    Ok(DatabaseAdminStatus {
        mode: config.mode,
        database_path: Some(path.to_string_lossy().into_owned()),
        file_size_bytes: metadata.as_ref().map(fs::Metadata::len),
        modified_at_unix,
        supports_file_management: true,
        remote_database: None,
    })
}

#[tauri::command]
pub async fn database_admin_list_backups(
    state: State<'_, PlatformState>,
) -> Result<Vec<DatabaseBackup>, String> {
    require_local_admin(&state).await?;
    let directory = state.app_data_dir.join(DATABASE_BACKUP_DIR);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            tracing::error!(%error, "Could not read the database backup directory");
            return Err("Could not list database backups.".to_string());
        }
    };
    let mut backups = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            tracing::error!(%error, "Could not read a database backup directory entry");
            "Could not list database backups.".to_string()
        })?;
        let path = entry.path();
        let is_sqlite = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "sqlite" | "sqlite3" | "db"
                )
            });
        if !is_sqlite {
            continue;
        }
        let metadata = entry.metadata().map_err(|error| {
            tracing::error!(%error, path = %path.display(), "Could not inspect a database backup");
            "Could not inspect a database backup.".to_string()
        })?;
        if !metadata.is_file() {
            continue;
        }
        let modified_at_unix = metadata
                .modified()
                .map_err(|error| {
                    tracing::error!(%error, path = %path.display(), "Could not read a database backup timestamp");
                    "Could not inspect a database backup timestamp.".to_string()
                })?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| {
                    tracing::error!(%error, path = %path.display(), "Database backup timestamp is invalid");
                    "A database backup timestamp is invalid.".to_string()
                })?
                .as_secs();
        backups.push(DatabaseBackup {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: path.to_string_lossy().into_owned(),
            size_bytes: metadata.len(),
            modified_at_unix,
        });
    }
    backups.sort_by(|left, right| right.modified_at_unix.cmp(&left.modified_at_unix));
    Ok(backups)
}

#[tauri::command]
pub async fn database_admin_backup(
    state: State<'_, PlatformState>,
    destination_path: String,
) -> Result<DatabaseOperationResult, String> {
    require_local_admin(&state).await?;
    let destination = validated_sqlite_path(&destination_path)?;
    if destination.exists() {
        return Err("A file already exists at the selected backup location.".to_string());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "Choose a valid backup location.".to_string())?;
    if !parent.is_dir() {
        return Err("The selected backup folder does not exist.".to_string());
    }
    let source = local_database_path(&state).await?;
    if same_path(&source, &destination) {
        return Err("The backup location must be different from the active database.".to_string());
    }

    let database = state.database().await?;
    vacuum_into(&database, &destination).await?;
    match validate_sqlite_database(&destination).await {
        Ok(()) => operation_result(destination),
        Err(error) => {
            remove_file_if_present(&destination);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn database_admin_check_integrity(
    state: State<'_, PlatformState>,
) -> Result<String, String> {
    require_local_admin(&state).await?;
    let database = state.database().await?;
    let integrity = integrity_check(database.connection()).await?;
    aqardesk_backend::migration::verify_schema(database.connection())
        .await
        .map_err(|error| {
            tracing::error!(%error, "Database schema validation failed");
            "The database is missing required tables or has an invalid schema.".to_string()
        })?;
    if integrity != "ok" {
        return Err(format!(
            "SQLite reported a database integrity problem: {integrity}"
        ));
    }
    Ok(integrity)
}

#[tauri::command]
pub async fn database_admin_restore(
    state: State<'_, PlatformState>,
    source_path: String,
) -> Result<DatabaseOperationResult, String> {
    require_local_admin(&state).await?;
    let source = validated_sqlite_path(&source_path)?;
    if !source.is_file() {
        return Err("The selected backup file does not exist.".to_string());
    }
    let active_path = local_database_path(&state).await?;
    if same_path(&source, &active_path) {
        return Err("Select a backup file other than the active database.".to_string());
    }
    validate_sqlite_database(&source).await?;

    let backup_dir = state.app_data_dir.join(DATABASE_BACKUP_DIR);
    fs::create_dir_all(&backup_dir).map_err(|error| {
        tracing::error!(%error, "Could not create the database backup directory");
        "Could not prepare a safety backup before restoring.".to_string()
    })?;
    let safety_path = backup_dir.join(format!(
        "before-restore-{}-{}.sqlite",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        Uuid::new_v4()
    ));
    let staged_path = sibling_temp_path(&active_path, "restore");
    if let Err(error) = copy_file(&source, &staged_path).await {
        remove_file_if_present(&staged_path);
        return Err(error);
    }
    if let Err(error) = validate_sqlite_database(&staged_path).await {
        remove_file_if_present(&staged_path);
        return Err(error);
    }

    let previous_path = sibling_temp_path(&active_path, "previous");
    let database = state.database.write().await.take().ok_or_else(|| {
        remove_file_if_present(&staged_path);
        "The active database is not connected.".to_string()
    })?;
    if let Err(error) = database.connection().clone().close().await {
        *state.database.write().await = Some(database);
        remove_file_if_present(&staged_path);
        tracing::error!(%error, "Could not close the active database before restore");
        return Err("Could not safely close the active database for restoration.".to_string());
    }
    drop(database);

    let active_config = state.config().await?;
    let safety_database_url = match active_config.database_url(&state.app_data_dir) {
        Ok(url) => url,
        Err(error) => {
            remove_file_if_present(&staged_path);
            reconnect_after_operation(&state).await?;
            return Err(error);
        }
    };
    let safety_database = match Database::connect(&safety_database_url).await {
        Ok(database) => database,
        Err(error) => {
            remove_file_if_present(&staged_path);
            reconnect_after_operation(&state).await?;
            return Err(format!(
                "Could not reopen the active database to create a safety backup: {error}"
            ));
        }
    };
    if let Err(error) = checkpoint(safety_database.connection()).await {
        let _ = safety_database.connection().clone().close().await;
        remove_file_if_present(&staged_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    if let Err(error) = vacuum_into(&safety_database, &safety_path).await {
        let _ = safety_database.connection().clone().close().await;
        remove_file_if_present(&staged_path);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    if let Err(error) = safety_database.connection().clone().close().await {
        remove_file_if_present(&staged_path);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        tracing::error!(%error, "Could not close the temporary safety-backup database connection");
        return Err("Could not safely finish the pre-restore safety backup.".to_string());
    }
    if let Err(error) = validate_sqlite_database(&safety_path).await {
        remove_file_if_present(&staged_path);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    remove_sqlite_sidecars(&active_path);

    if let Err(error) = fs::rename(&active_path, &previous_path) {
        remove_file_if_present(&staged_path);
        reconnect_after_operation(&state).await?;
        tracing::error!(%error, "Could not stage the active database before restore");
        return Err("Could not prepare the active database for restoration.".to_string());
    }
    if let Err(error) = fs::rename(&staged_path, &active_path) {
        let rollback = fs::rename(&previous_path, &active_path);
        tracing::error!(%error, "Could not install the selected database backup");
        return match rollback {
            Ok(()) => {
                reconnect_after_operation(&state).await?;
                Err("Could not install the selected database backup. The original database was restored.".to_string())
            }
            Err(rollback_error) => {
                tracing::error!(%rollback_error, "Could not restore the original database file");
                Err(format!(
                    "Could not install the selected backup ({error}) or restore the original database ({rollback_error}). The safety backup is saved at {}.",
                    safety_path.display()
                ))
            }
        };
    }

    if let Err(error) = reconnect_after_operation(&state).await {
        remove_sqlite_sidecars(&active_path);
        remove_file_if_present(&active_path);
        let rollback = fs::rename(&previous_path, &active_path);
        if let Err(rollback_error) = rollback {
            tracing::error!(%rollback_error, "Could not roll back a failed database restore");
            return Err(format!(
                "The restored database could not be opened ({error}) and the original could not be put back ({rollback_error}). Safety backup: {}.",
                safety_path.display()
            ));
        }
        reconnect_after_operation(&state).await?;
        return Err(format!(
            "The selected backup could not be opened. The original database has been restored. Details: {error}"
        ));
    }

    if let Err(error) = platform_runtime::clear_session() {
        tracing::error!(%error, "Could not clear the session after database restore");
        let restored_database = state.database.write().await.take();
        if let Some(database) = restored_database {
            let _ = database.connection().clone().close().await;
        }
        remove_sqlite_sidecars(&active_path);
        remove_file_if_present(&active_path);
        if let Err(rollback_error) = fs::rename(&previous_path, &active_path) {
            tracing::error!(%rollback_error, "Could not roll back restore after session cleanup failed");
            return Err(format!(
                "Could not clear the saved sign-in ({error}) or restore the original database ({rollback_error}). Safety backup: {}.",
                safety_path.display()
            ));
        }
        reconnect_after_operation(&state).await?;
        return Err("Could not clear the saved sign-in, so the restore was cancelled and the original database was restored.".to_string());
    }
    *state.current_user.write().await = None;
    *state.api_token.write().await = None;
    remove_file_if_present(&previous_path);
    operation_result(active_path)
}

#[tauri::command]
pub async fn database_admin_change_location(
    state: State<'_, PlatformState>,
    destination_path: String,
) -> Result<DatabaseOperationResult, String> {
    require_local_admin(&state).await?;
    let destination = validated_sqlite_path(&destination_path)?;
    if destination.exists() {
        return Err("A file already exists at the selected database location.".to_string());
    }
    if !destination.parent().is_some_and(|parent| parent.is_dir()) {
        return Err("The selected database folder does not exist.".to_string());
    }
    let source = local_database_path(&state).await?;
    if same_path(&source, &destination) {
        return Err("The selected location is already in use.".to_string());
    }
    let original_config = state.config().await?;
    let database = state
        .database
        .write()
        .await
        .take()
        .ok_or_else(|| "The active database is not connected.".to_string())?;
    if let Err(error) = database.connection().clone().close().await {
        *state.database.write().await = Some(database);
        tracing::error!(%error, "Could not close the active database before moving it");
        return Err("Could not safely close the active database.".to_string());
    }
    drop(database);

    let database_url = match original_config.database_url(&state.app_data_dir) {
        Ok(url) => url,
        Err(error) => {
            reconnect_after_operation(&state).await?;
            return Err(error);
        }
    };
    let source_database = match Database::connect(&database_url).await {
        Ok(database) => database,
        Err(error) => {
            reconnect_after_operation(&state).await?;
            return Err(format!(
                "Could not reopen the current database for moving: {error}"
            ));
        }
    };
    if let Err(error) = checkpoint(source_database.connection()).await {
        let _ = source_database.connection().clone().close().await;
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    let backup_dir = state.app_data_dir.join(DATABASE_BACKUP_DIR);
    if let Err(error) = fs::create_dir_all(&backup_dir) {
        let _ = source_database.connection().clone().close().await;
        reconnect_after_operation(&state).await?;
        tracing::error!(%error, "Could not create database safety backup directory");
        return Err("Could not prepare a safety backup before moving the database.".to_string());
    }
    let safety_path = backup_dir.join(format!(
        "before-move-{}-{}.sqlite",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        Uuid::new_v4()
    ));
    let backup_result = async {
        vacuum_into(&source_database, &destination).await?;
        vacuum_into(&source_database, &safety_path).await
    }
    .await;
    let close_result = source_database.connection().clone().close().await;
    if let Err(error) = backup_result {
        remove_file_if_present(&destination);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    if let Err(error) = close_result {
        remove_file_if_present(&destination);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        tracing::error!(%error, "Could not close the temporary source database connection");
        return Err("Could not safely finish moving the database.".to_string());
    }
    if let Err(error) = validate_sqlite_database(&destination).await {
        remove_file_if_present(&destination);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    if let Err(error) = validate_sqlite_database(&safety_path).await {
        remove_file_if_present(&destination);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }

    let mut next_config = original_config.clone();
    let Some(local_config) = next_config.local.as_mut() else {
        remove_file_if_present(&destination);
        reconnect_after_operation(&state).await?;
        return Err("Local database settings are missing.".to_string());
    };
    local_config.database_path = destination.to_string_lossy().into_owned();
    if let Err(error) = platform_runtime::save_config(&state.app_data_dir, &next_config) {
        if let Err(rollback_error) =
            platform_runtime::save_config(&state.app_data_dir, &original_config)
        {
            tracing::error!(%rollback_error, "Could not restore the original database configuration after a failed move");
        }
        remove_file_if_present(&destination);
        remove_file_if_present(&safety_path);
        reconnect_after_operation(&state).await?;
        return Err(error);
    }
    *state.config.write().await = Some(next_config.clone());
    if let Err(error) = state.connect().await {
        *state.config.write().await = Some(original_config.clone());
        if let Err(rollback_error) =
            platform_runtime::save_config(&state.app_data_dir, &original_config)
        {
            tracing::error!(%rollback_error, "Could not restore the original database configuration");
        }
        remove_sqlite_sidecars(&destination);
        remove_file_if_present(&destination);
        reconnect_after_operation(&state).await?;
        return Err(format!(
            "Could not connect to the database at the new location. The previous location remains active. Details: {error}"
        ));
    }

    if let Err(error) = platform_commands::rebind_current_session(&state, &next_config).await {
        if let Some(database) = state.database.write().await.take() {
            if let Err(close_error) = database.connection().clone().close().await {
                tracing::error!(%close_error, "Could not close the new database connection during rollback");
            }
        }
        if let Err(session_error) =
            platform_commands::rebind_current_session(&state, &original_config).await
        {
            tracing::error!(%session_error, "Could not restore the original saved session after moving failed");
        }
        *state.config.write().await = Some(original_config.clone());
        if let Err(config_error) =
            platform_runtime::save_config(&state.app_data_dir, &original_config)
        {
            tracing::error!(%config_error, "Could not restore the original database configuration");
        }
        remove_sqlite_sidecars(&destination);
        remove_file_if_present(&destination);
        reconnect_after_operation(&state).await?;
        return Err(format!(
            "Could not preserve the current sign-in while moving the database: {error}"
        ));
    }

    remove_sqlite_sidecars(&source);
    remove_file_if_present(&source);
    operation_result(destination)
}

async fn require_local_admin(state: &PlatformState) -> Result<(), String> {
    require_admin(state).await?;
    if state.config().await?.mode != ConnectionMode::Local {
        return Err("Database file operations are available only for a local SQLite database. Use the database server's backup tools for remote connections.".to_string());
    }
    Ok(())
}

async fn require_admin(state: &PlatformState) -> Result<(), String> {
    let actor = platform_commands::active_actor(state).await?;
    if actor.role() != aqardesk_backend::domain::AccessRole::Admin {
        return Err("Administrator access is required for database management.".to_string());
    }
    Ok(())
}

async fn local_database_path(state: &PlatformState) -> Result<PathBuf, String> {
    state
        .config()
        .await?
        .local_database_path(&state.app_data_dir)
}

fn validated_sqlite_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value.trim());
    if !path.is_absolute() {
        return Err("Choose an absolute file location.".to_string());
    }
    let has_sqlite_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "sqlite" | "sqlite3" | "db"
            )
        });
    if !has_sqlite_extension {
        return Err("Database files must use a .sqlite, .sqlite3, or .db extension.".to_string());
    }
    Ok(path)
}

async fn vacuum_into(database: &Database, destination: &Path) -> Result<(), String> {
    database
        .connection()
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "VACUUM INTO ?",
            [destination.to_string_lossy().to_string().into()],
        ))
        .await
        .map_err(|error| {
            tracing::error!(%error, path = %destination.display(), "Could not create a consistent SQLite snapshot");
            format!("Could not create a database snapshot: {error}")
        })?;
    Ok(())
}

async fn checkpoint(database: &sea_orm::DatabaseConnection) -> Result<(), String> {
    let result = database
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "PRAGMA wal_checkpoint(TRUNCATE)",
        ))
        .await
        .map_err(|error| {
            tracing::error!(%error, "Could not checkpoint the active SQLite database");
            "Could not safely flush pending database changes.".to_string()
        })?;
    if let Some(result) = result {
        let busy = result
            .try_get_by::<i32, _>(0)
            .map_err(|error| format!("Could not verify the SQLite checkpoint result: {error}"))?;
        if busy != 0 {
            return Err(
                "The database is busy and could not safely flush pending changes.".to_string(),
            );
        }
    }
    Ok(())
}

async fn validate_sqlite_database(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("The selected file is not a database file.".to_string());
    }
    let url = sqlite_read_only_url(path);
    let database = Database::connect(&url)
        .await
        .map_err(|error| format!("Could not open the selected SQLite database: {error}"))?;
    let integrity = integrity_check(database.connection()).await?;
    if integrity != "ok" {
        return Err(format!(
            "The selected database failed its integrity check: {integrity}"
        ));
    }
    aqardesk_backend::migration::verify_schema(database.connection())
        .await
        .map_err(|error| {
            tracing::error!(%error, path = %path.display(), "Selected database schema is invalid");
            "The selected file is missing required AqarDesk database tables.".to_string()
        })?;
    database
        .connection()
        .clone()
        .close()
        .await
        .map_err(|error| format!("Could not close the validation connection: {error}"))?;
    Ok(())
}

async fn integrity_check(database: &sea_orm::DatabaseConnection) -> Result<String, String> {
    let result = database
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "PRAGMA integrity_check",
        ))
        .await
        .map_err(|error| {
            tracing::error!(%error, "SQLite integrity check failed to run");
            "Could not run the SQLite integrity check.".to_string()
        })?
        .ok_or_else(|| "SQLite did not return an integrity check result.".to_string())?;
    result
        .try_get_by::<String, _>(0)
        .map_err(|error| format!("Could not read the SQLite integrity check result: {error}"))
}

fn sqlite_read_only_url(path: &Path) -> String {
    let mut escaped = String::new();
    for byte in path.to_string_lossy().replace('\\', "/").bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'.' | b'_' | b'-') {
            escaped.push(byte as char);
        } else {
            escaped.push_str(&format!("%{byte:02X}"));
        }
    }
    format!("sqlite://{escaped}?mode=ro")
}

async fn reconnect_after_operation(state: &PlatformState) -> Result<(), String> {
    state.connect().await.map(|_| ())
}

async fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    let source = source.to_path_buf();
    let destination = destination.to_path_buf();
    tokio::task::spawn_blocking(move || fs::copy(source, destination))
        .await
        .map_err(|error| format!("Database file operation failed: {error}"))?
        .map(|_| ())
        .map_err(|error| {
            tracing::error!(%error, "Could not copy a database file");
            "Could not copy the selected database file.".to_string()
        })
}

fn sibling_temp_path(path: &Path, purpose: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("aqardesk.sqlite");
    path.with_file_name(format!(".{name}.{purpose}-{}.tmp", Uuid::new_v4()))
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn remove_sqlite_sidecars(path: &Path) {
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", path.to_string_lossy()));
        remove_file_if_present(&sidecar);
    }
}

fn remove_file_if_present(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(%error, path = %path.display(), "Could not remove a temporary database file");
        }
    }
}

fn operation_result(path: PathBuf) -> Result<DatabaseOperationResult, String> {
    let metadata = fs::metadata(&path).map_err(|error| {
        tracing::error!(%error, path = %path.display(), "Could not inspect completed database operation");
        "The database operation completed, but its result could not be verified.".to_string()
    })?;
    Ok(DatabaseOperationResult {
        path: path.to_string_lossy().into_owned(),
        size_bytes: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_runtime::{ConnectionConfig, LocalConnection};
    use sea_orm::DbBackend;

    #[tokio::test]
    async fn sqlite_snapshot_with_spaces_is_valid_and_keeps_the_schema() {
        let directory = std::env::temp_dir().join(format!("aqardesk-db-admin-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create temporary database directory");
        let config = ConnectionConfig {
            mode: ConnectionMode::Local,
            local: Some(LocalConnection {
                database_path: "source.sqlite".to_string(),
            }),
            direct: None,
            api: None,
        };
        let database = Database::connect(
            &config
                .database_url(&directory)
                .expect("build source database URL"),
        )
        .await
        .expect("connect source SQLite database");
        aqardesk_backend::migration::initialize(database.connection())
            .await
            .expect("initialize source schema");
        database
            .connection()
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "INSERT INTO organizations (id, name, default_currency, timezone) \
                 VALUES ('snapshot-org', 'Snapshot Organization', 'SAR', 'Asia/Riyadh')",
            ))
            .await
            .expect("insert organization");

        checkpoint(database.connection())
            .await
            .expect("checkpoint source database");
        let destination = directory.join("verified backup.sqlite");
        vacuum_into(&database, &destination)
            .await
            .expect("create database snapshot");
        database
            .connection()
            .clone()
            .close()
            .await
            .expect("close source database");
        validate_sqlite_database(&destination)
            .await
            .expect("validate database snapshot");
        let backup_database = Database::connect(&sqlite_read_only_url(&destination))
            .await
            .expect("open database snapshot");
        let organization_count = backup_database
            .connection()
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS count FROM organizations WHERE id = 'snapshot-org'",
            ))
            .await
            .expect("query database snapshot")
            .expect("snapshot query result")
            .try_get_by::<i64, _>("count")
            .expect("read snapshot query result");
        assert_eq!(organization_count, 1);
        backup_database
            .connection()
            .clone()
            .close()
            .await
            .expect("close database snapshot");

        fs::remove_dir_all(&directory).expect("remove temporary database directory");
    }
}
