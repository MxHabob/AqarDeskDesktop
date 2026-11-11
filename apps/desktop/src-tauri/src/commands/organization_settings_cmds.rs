use aqardesk_backend::{
    application::{CurrencyCatalog, OrganizationSettingsInput, OrganizationSettingsService},
    infrastructure::{SeaOrmCurrencyRepository, SeaOrmOrganizationSettingsRepository},
};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn organization_get_settings(
    state: State<'_, PlatformState>,
) -> Result<aqardesk_backend::application::OrganizationSettings, String> {
    let (actor, database) = local_business_state(&state).await?;
    OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database))
        .get(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn organization_update_settings(
    state: State<'_, PlatformState>,
    input: OrganizationSettingsInput,
) -> Result<aqardesk_backend::application::OrganizationSettings, String> {
    let (actor, database) = local_business_state(&state).await?;
    OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database))
        .update(actor, input)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn currencies_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<aqardesk_backend::application::CurrencySettings>, String> {
    let (actor, database) = local_business_state(&state).await?;
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn currencies_set_organization_enabled(
    state: State<'_, PlatformState>,
    currency_code: String,
    is_enabled: bool,
) -> Result<(), String> {
    let (actor, database) = local_business_state(&state).await?;
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .set_enabled(actor, &currency_code, is_enabled)
        .await
        .map_err(application_error)
}
