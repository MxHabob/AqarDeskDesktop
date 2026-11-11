mod commands;
mod connection;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(commands::handlers())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let app_data_dir = app.path().app_data_dir().map_err(std::io::Error::other)?;
            std::fs::create_dir_all(&app_data_dir)?;
            let config = connection::load_config(&app_data_dir).map_err(std::io::Error::other)?;
            app.manage(connection::ConnectionState {
                app_data_dir,
                config: std::sync::RwLock::new(config),
            });
            app.manage(connection::DatabaseState {
                connection: tokio::sync::RwLock::new(None),
            });
            app.manage(connection::AuthState::default());
            app.manage(commands::update_cmds::PendingUpdateState::default());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
