use crate::{
    commands::user_cmds::{self, api_request},
    connection::{AuthState, ConnectionState, DatabaseState},
};
use aqardesk_app::services::settings::{
    AuditEntry, ContractTemplate, ContractTemplateInput, CurrencySettings, OrganizationSettings,
    OrganizationSettingsInput, ProfileSettings, ProfileSettingsInput, SettingsService,
};
use aqardesk_infrastructure::{
    repositories::settings::SeaOrmSettingsRepository, settings::ConnectionMode,
};
use reqwest::Method;
use tauri::State;

#[tauri::command]
pub async fn organization_get_settings(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<OrganizationSettings, String> {
    let user = user_cmds::require_active_user(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_request(
            &config,
            Method::GET,
            "/api/v1/organizations/current",
            auth.api_token.read().await.clone(),
            None,
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .get_organization(&user.organization_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn organization_update_settings(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: OrganizationSettingsInput,
) -> Result<OrganizationSettings, String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let body = serde_json::to_value(input)
            .map_err(|error| format!("Could not encode organization settings: {error}"))?;
        return api_request(
            &config,
            Method::PUT,
            "/api/v1/organizations/current",
            auth.api_token.read().await.clone(),
            Some(body),
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .update_organization(&user.organization_id, &user.id, input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn currencies_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<CurrencySettings>, String> {
    let user = user_cmds::require_active_user(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_request(
            &config,
            Method::GET,
            "/api/v1/currencies",
            auth.api_token.read().await.clone(),
            None,
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .list_currencies(&user.organization_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn currencies_set_organization_enabled(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    currency_code: String,
    is_enabled: bool,
) -> Result<(), String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let currency_code = currency_code.trim().to_uppercase();
    if currency_code.len() != 3 || !currency_code.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err("Currency must be a three-letter currency code.".to_string());
    }
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let _: serde_json::Value = api_request(
            &config,
            Method::PUT,
            &format!("/api/v1/currencies/{}/organization-state", currency_code),
            auth.api_token.read().await.clone(),
            Some(serde_json::json!({ "is_enabled": is_enabled })),
        )
        .await?;
        return Ok(());
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .set_currency_enabled(&user.organization_id, &user.id, &currency_code, is_enabled)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn profile_get_settings(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<ProfileSettings, String> {
    let user = user_cmds::require_active_user(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_request(
            &config,
            Method::GET,
            "/api/v1/users/me",
            auth.api_token.read().await.clone(),
            None,
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .get_profile(&user.organization_id, &user.id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn profile_update_settings(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: ProfileSettingsInput,
) -> Result<ProfileSettings, String> {
    let user = user_cmds::require_active_user(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let body = serde_json::to_value(input)
            .map_err(|error| format!("Could not encode profile settings: {error}"))?;
        return api_request(
            &config,
            Method::PUT,
            "/api/v1/users/me",
            auth.api_token.read().await.clone(),
            Some(body),
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .update_profile(&user.organization_id, &user.id, input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn audit_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<AuditEntry>, String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    if config.mode == ConnectionMode::Api {
        return api_request(
            &config,
            Method::GET,
            &format!("/api/v1/audit-events?limit={limit}&offset={offset}"),
            auth.api_token.read().await.clone(),
            None,
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .list_audit(&user.organization_id, limit, offset)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn contract_templates_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<ContractTemplate>, String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_request(
            &config,
            Method::GET,
            "/api/v1/contract-templates",
            auth.api_token.read().await.clone(),
            None,
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .list_templates(&user.organization_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn contract_templates_create(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: ContractTemplateInput,
) -> Result<ContractTemplate, String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let body = serde_json::to_value(input)
            .map_err(|error| format!("Could not encode contract template: {error}"))?;
        return api_request(
            &config,
            Method::POST,
            "/api/v1/contract-templates",
            auth.api_token.read().await.clone(),
            Some(body),
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .create_template(&user.organization_id, &user.id, input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn contract_templates_update(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    template_id: String,
    input: ContractTemplateInput,
) -> Result<ContractTemplate, String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let body = serde_json::to_value(input)
            .map_err(|error| format!("Could not encode contract template: {error}"))?;
        return api_request(
            &config,
            Method::PUT,
            &format!("/api/v1/contract-templates/{template_id}"),
            auth.api_token.read().await.clone(),
            Some(body),
        )
        .await;
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .update_template(&user.organization_id, &user.id, &template_id, input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn contract_templates_set_active(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    template_id: String,
    is_active: bool,
) -> Result<(), String> {
    let user = user_cmds::require_admin(&connection, &database, &auth).await?;
    let config = user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let _: serde_json::Value = api_request(
            &config,
            Method::PATCH,
            &format!("/api/v1/contract-templates/{template_id}/state"),
            auth.api_token.read().await.clone(),
            Some(serde_json::json!({ "is_active": is_active })),
        )
        .await?;
        return Ok(());
    }
    let database = database.get().await?;
    let service = SettingsService::new(SeaOrmSettingsRepository::new(&database));
    service
        .set_template_active(&user.organization_id, &user.id, &template_id, is_active)
        .await
        .map_err(|error| error.to_string())
}
