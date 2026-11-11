pub mod business_cmds;
pub mod connection_cmds;
pub mod dashboard_cmds;
pub mod settings_cmds;
pub mod update_cmds;
pub mod user_cmds;

pub fn handlers() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        connection_cmds::connection_get_config,
        connection_cmds::connection_set_config,
        connection_cmds::connection_check_api,
        connection_cmds::connection_connect,
        dashboard_cmds::dashboard_get_summary,
        user_cmds::auth_get_setup_status,
        user_cmds::auth_bootstrap_admin,
        user_cmds::auth_login,
        user_cmds::organization_get_name,
        user_cmds::auth_logout,
        user_cmds::users_list,
        user_cmds::users_create,
        user_cmds::users_set_active,
        settings_cmds::organization_get_settings,
        settings_cmds::organization_update_settings,
        settings_cmds::currencies_list,
        settings_cmds::currencies_set_organization_enabled,
        settings_cmds::profile_get_settings,
        settings_cmds::profile_update_settings,
        settings_cmds::audit_list,
        settings_cmds::contract_templates_list,
        settings_cmds::contract_templates_create,
        settings_cmds::contract_templates_update,
        settings_cmds::contract_templates_set_active,
        business_cmds::properties_list,
        business_cmds::properties_create,
        business_cmds::properties_update,
        business_cmds::properties_archive,
        business_cmds::units_list,
        business_cmds::units_create,
        business_cmds::units_update,
        business_cmds::units_archive,
        business_cmds::tenants_list,
        business_cmds::tenants_create,
        business_cmds::tenants_update,
        business_cmds::tenants_archive,
        business_cmds::payments_list,
        business_cmds::payments_record,
        business_cmds::payments_void,
        update_cmds::system_check_for_updates,
        update_cmds::system_download_update,
        update_cmds::system_install_update
    ]
}
