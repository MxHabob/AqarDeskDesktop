pub mod connection_cmds;
pub mod update_cmds;
pub mod user_cmds;

pub fn handlers() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        connection_cmds::connection_get_config,
        connection_cmds::connection_set_config,
        connection_cmds::connection_check_api,
        connection_cmds::connection_connect,
        user_cmds::auth_get_setup_status,
        user_cmds::auth_bootstrap_admin,
        user_cmds::auth_login,
        user_cmds::auth_logout,
        user_cmds::users_list,
        user_cmds::users_create,
        user_cmds::users_set_active,
        update_cmds::system_check_for_updates,
        update_cmds::system_download_update,
        update_cmds::system_install_update
    ]
}
