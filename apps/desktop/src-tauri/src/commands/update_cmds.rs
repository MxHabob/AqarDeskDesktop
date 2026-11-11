use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateDownloadProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

pub struct PendingUpdateState {
    operation_lock: tokio::sync::Mutex<()>,
    pending: tokio::sync::Mutex<Option<PendingUpdate>>,
}

struct PendingUpdate {
    update: tauri_plugin_updater::Update,
    bytes: Vec<u8>,
}

impl Default for PendingUpdateState {
    fn default() -> Self {
        Self {
            operation_lock: tokio::sync::Mutex::new(()),
            pending: tokio::sync::Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn system_check_for_updates(app: AppHandle) -> Result<UpdateCheckResult, String> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(|error| {
            format!("Desktop updates are not configured. Configure a signing key and release endpoint: {error}")
        })?
        .check()
        .await
        .map_err(|error| format!("Could not check for desktop updates: {error}"))?;

    Ok(match update {
        Some(update) => UpdateCheckResult {
            current_version,
            latest_version: update.version.to_string(),
            has_update: true,
            release_notes: update.body.clone(),
        },
        None => UpdateCheckResult {
            current_version: current_version.clone(),
            latest_version: current_version,
            has_update: false,
            release_notes: None,
        },
    })
}

#[tauri::command]
pub async fn system_download_update(
    app: AppHandle,
    pending: State<'_, PendingUpdateState>,
) -> Result<String, String> {
    let _operation = pending.operation_lock.lock().await;
    let update = app
        .updater()
        .map_err(|error| format!("Could not initialize updater: {error}"))?
        .check()
        .await
        .map_err(|error| format!("Could not check for desktop updates: {error}"))?
        .ok_or_else(|| "No desktop update is currently available.".to_string())?;
    let version = update.version.to_string();
    let app_for_progress = app.clone();
    let mut downloaded_bytes = 0_u64;
    let bytes = update
        .download(
            move |chunk_length, total_bytes| {
                downloaded_bytes = downloaded_bytes.saturating_add(chunk_length as u64);
                if let Err(error) = app_for_progress.emit(
                    "desktop-update-progress",
                    UpdateDownloadProgress {
                        downloaded_bytes,
                        total_bytes,
                    },
                ) {
                    tracing::warn!(%error, "Could not report update download progress");
                }
            },
            || tracing::info!("Desktop update download and signature verification completed"),
        )
        .await
        .map_err(|error| format!("Could not download or verify desktop update: {error}"))?;
    *pending.pending.lock().await = Some(PendingUpdate { update, bytes });
    Ok(version)
}

#[tauri::command]
pub async fn system_install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdateState>,
) -> Result<(), String> {
    let update = pending
        .pending
        .lock()
        .await
        .take()
        .ok_or_else(|| "Download an update before installing it.".to_string())?;
    if let Err(error) = update.update.install(&update.bytes) {
        *pending.pending.lock().await = Some(update);
        return Err(format!("Could not install desktop update: {error}"));
    }
    app.restart();
}
