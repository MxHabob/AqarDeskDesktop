use crate::connection::{AuthState, ConnectionState, DatabaseState, LoggedInUser};
use aqardesk_infrastructure::{
    models::{property, tenant, unit},
    services::{
        properties::{self, PropertyInput},
        tenants::{self, TenantInput},
        units::{self, UnitInput},
    },
    settings::ConnectionMode,
    users,
};
use reqwest::Method;
use serde::Serialize;
use tauri::State;

#[tauri::command]
pub async fn properties_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<property::Model>, String> {
    let user = require_user(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call(
            &connection,
            &auth,
            Method::GET,
            "/api/v1/properties",
            None::<serde_json::Value>,
        )
        .await;
    }
    properties::list_properties(&database.get().await?, &user.organization_id).await
}

#[tauri::command]
pub async fn properties_create(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: PropertyInput,
) -> Result<property::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call::<property::Model, _>(
            &connection,
            &auth,
            Method::POST,
            "/api/v1/properties",
            Some(input),
        )
        .await;
    }
    properties::create_property(
        &database.get().await?,
        &user.organization_id,
        &user.id,
        input,
    )
    .await
}

#[tauri::command]
pub async fn properties_update(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    property_id: String,
    input: PropertyInput,
) -> Result<property::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/properties/{}", encode_segment(&property_id));
        return api_call(&connection, &auth, Method::PUT, &path, Some(input)).await;
    }
    properties::update_property(
        &database.get().await?,
        &user.organization_id,
        &user.id,
        &property_id,
        input,
    )
    .await
}

#[tauri::command]
pub async fn properties_archive(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    property_id: String,
) -> Result<(), String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/properties/{}", encode_segment(&property_id));
        return api_call(
            &connection,
            &auth,
            Method::DELETE,
            &path,
            None::<serde_json::Value>,
        )
        .await;
    }
    properties::archive_property(&database.get().await?, &user.organization_id, &property_id).await
}

#[tauri::command]
pub async fn units_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<unit::Model>, String> {
    let user = require_user(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call(
            &connection,
            &auth,
            Method::GET,
            "/api/v1/units",
            None::<serde_json::Value>,
        )
        .await;
    }
    units::list_units(&database.get().await?, &user.organization_id, None).await
}

#[tauri::command]
pub async fn units_create(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: UnitInput,
) -> Result<unit::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call(
            &connection,
            &auth,
            Method::POST,
            "/api/v1/units",
            Some(input),
        )
        .await;
    }
    units::create_unit(&database.get().await?, &user.organization_id, input).await
}

#[tauri::command]
pub async fn units_update(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    unit_id: String,
    input: UnitInput,
) -> Result<unit::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/units/{}", encode_segment(&unit_id));
        return api_call(&connection, &auth, Method::PUT, &path, Some(input)).await;
    }
    units::update_unit(
        &database.get().await?,
        &user.organization_id,
        &unit_id,
        input,
    )
    .await
}

#[tauri::command]
pub async fn units_archive(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    unit_id: String,
) -> Result<(), String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/units/{}", encode_segment(&unit_id));
        return api_call(
            &connection,
            &auth,
            Method::DELETE,
            &path,
            None::<serde_json::Value>,
        )
        .await;
    }
    units::archive_unit(&database.get().await?, &user.organization_id, &unit_id).await
}

#[tauri::command]
pub async fn tenants_list(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
) -> Result<Vec<tenant::Model>, String> {
    let user = require_user(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call(
            &connection,
            &auth,
            Method::GET,
            "/api/v1/tenants",
            None::<serde_json::Value>,
        )
        .await;
    }
    tenants::list_tenants(&database.get().await?, &user.organization_id).await
}

#[tauri::command]
pub async fn tenants_create(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: TenantInput,
) -> Result<tenant::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        return api_call(
            &connection,
            &auth,
            Method::POST,
            "/api/v1/tenants",
            Some(input),
        )
        .await;
    }
    tenants::create_tenant(&database.get().await?, &user.organization_id, input).await
}

#[tauri::command]
pub async fn tenants_update(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    tenant_id: String,
    input: TenantInput,
) -> Result<tenant::Model, String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/tenants/{}", encode_segment(&tenant_id));
        return api_call(&connection, &auth, Method::PUT, &path, Some(input)).await;
    }
    tenants::update_tenant(
        &database.get().await?,
        &user.organization_id,
        &tenant_id,
        input,
    )
    .await
}

#[tauri::command]
pub async fn tenants_archive(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    tenant_id: String,
) -> Result<(), String> {
    let user = require_writer(&connection, &database, &auth).await?;
    let config = super::user_cmds::get_config(&connection)?;
    if config.mode == ConnectionMode::Api {
        let path = format!("/api/v1/tenants/{}", encode_segment(&tenant_id));
        return api_call(
            &connection,
            &auth,
            Method::DELETE,
            &path,
            None::<serde_json::Value>,
        )
        .await;
    }
    tenants::archive_tenant(&database.get().await?, &user.organization_id, &tenant_id).await
}

async fn require_user(
    connection: &ConnectionState,
    database: &DatabaseState,
    auth: &AuthState,
) -> Result<LoggedInUser, String> {
    let mut user = auth
        .current_user
        .read()
        .await
        .clone()
        .ok_or_else(|| "Sign in before managing business records.".to_string())?;
    if !user.is_active {
        return Err("The session is no longer valid. Sign in again.".to_string());
    }
    if super::user_cmds::get_config(connection)?.mode != ConnectionMode::Api {
        let Some(active_user) = users::get_active_user(&database.get().await?, &user.id).await?
        else {
            return Err("The session is no longer valid. Sign in again.".to_string());
        };
        user.organization_id = active_user.organization_id;
        user.role = active_user.role;
    }
    Ok(user)
}

async fn require_writer(
    connection: &ConnectionState,
    database: &DatabaseState,
    auth: &AuthState,
) -> Result<LoggedInUser, String> {
    let user = require_user(connection, database, auth).await?;
    if user.role == "readonly" {
        return Err("Read-only users cannot change business records.".to_string());
    }
    Ok(user)
}

async fn api_call<T, B>(
    connection: &ConnectionState,
    auth: &AuthState,
    method: Method,
    path: &str,
    body: Option<B>,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
    B: Serialize,
{
    let config = super::user_cmds::get_config(connection)?;
    let token = auth.api_token.read().await.clone();
    let body = body
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| format!("Could not encode business request: {error}"))?;
    super::user_cmds::api_request(&config, method, path, token, body).await
}

fn encode_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}
