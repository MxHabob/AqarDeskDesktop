pub mod audit_cmds;
pub mod contract_template_cmds;
pub mod dashboard_cmds;
pub mod organization_settings_cmds;
pub mod profile_cmds;
pub mod update_cmds;

pub fn handlers() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        audit_cmds::audit_list,
        contract_template_cmds::contract_templates_list,
        contract_template_cmds::contract_templates_create,
        contract_template_cmds::contract_templates_update,
        contract_template_cmds::contract_templates_set_active,
        dashboard_cmds::dashboard_get_summary,
        organization_settings_cmds::organization_get_settings,
        organization_settings_cmds::organization_update_settings,
        organization_settings_cmds::currencies_list,
        organization_settings_cmds::currencies_set_organization_enabled,
        profile_cmds::profile_get_settings,
        profile_cmds::profile_update_settings,
        update_cmds::system_check_for_updates,
        update_cmds::system_download_update,
        update_cmds::system_install_update,
        crate::platform_commands::platform_connection_get_config,
        crate::platform_commands::platform_connection_set_config,
        crate::platform_commands::platform_connection_check_api,
        crate::platform_commands::platform_connection_connect,
        crate::platform_commands::platform_organization_get_name,
        crate::platform_commands::platform_auth_get_setup_status,
        crate::platform_commands::platform_auth_bootstrap_admin,
        crate::platform_commands::platform_auth_login,
        crate::platform_commands::platform_auth_logout,
        crate::platform_commands::platform_users_list,
        crate::platform_commands::platform_users_create,
        crate::platform_commands::platform_users_set_active,
        crate::platform_commands::platform_properties_list,
        crate::platform_commands::platform_properties_create,
        crate::platform_commands::platform_properties_update,
        crate::platform_commands::platform_properties_archive,
        crate::platform_commands::platform_units_list,
        crate::platform_commands::platform_units_create,
        crate::platform_commands::platform_units_update,
        crate::platform_commands::platform_units_archive,
        crate::platform_commands::platform_tenants_list,
        crate::platform_commands::platform_tenants_create,
        crate::platform_commands::platform_tenants_update,
        crate::platform_commands::platform_tenants_archive,
        crate::platform_commands::platform_payments_list,
        crate::platform_commands::platform_payments_record,
        crate::platform_commands::platform_payments_void
    ]
}
