use aqardesk_backend::{
    application::{
        CurrencyCatalog, CurrencyInput, NewCurrencyInput, OrganizationSettingsInput,
        OrganizationSettingsService, Page, PageRequest,
    },
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
    state
        .license
        .require_operational(&actor.organization_id().to_string())
        .await?;
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
    state
        .license
        .require_operational(&actor.organization_id().to_string())
        .await?;
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn currencies_list_page(
    state: State<'_, PlatformState>,
    limit: u64,
    offset: u64,
) -> Result<Page<aqardesk_backend::application::CurrencySettings>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_operational(&actor.organization_id().to_string())
        .await?;
    let page = PageRequest::new(limit, offset).map_err(application_error)?;
    let currencies = CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)?;
    let items = currencies
        .into_iter()
        .skip(usize::try_from(page.offset()).unwrap_or(usize::MAX))
        .take(usize::try_from(page.fetch_limit()).unwrap_or(usize::MAX))
        .collect();
    Ok(Page::from_items(items, page.limit()))
}

#[tauri::command]
pub async fn currencies_create(
    state: State<'_, PlatformState>,
    input: NewCurrencyInput,
) -> Result<(), String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "multi_currency")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    if input.enable_for_organization {
        let currencies = CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
            .list(actor)
            .await
            .map_err(application_error)?;
        let enabled_count = currencies
            .iter()
            .filter(|currency| currency.is_enabled_for_organization)
            .count() as u64;
        state
            .license
            .check_limit_for_organization(&organization_id, "currencies.max", enabled_count, 1)
            .await?;
    }
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .create(actor, input)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn currencies_update(
    state: State<'_, PlatformState>,
    currency_code: String,
    input: CurrencyInput,
) -> Result<(), String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "multi_currency")
        .await?;
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .update(actor, &currency_code, input)
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
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "multi_currency")
        .await?;
    let _quota_guard = if is_enabled {
        Some(state.license_quota_lock.lock().await)
    } else {
        None
    };
    if is_enabled {
        let currencies = CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
            .list(actor)
            .await
            .map_err(application_error)?;
        if currencies
            .iter()
            .any(|currency| currency.code == currency_code && currency.is_enabled_for_organization)
        {
            return CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
                .set_enabled(actor, &currency_code, is_enabled)
                .await
                .map_err(application_error);
        }
        let enabled_count = currencies
            .iter()
            .filter(|currency| currency.is_enabled_for_organization)
            .count() as u64;
        state
            .license
            .check_limit_for_organization(&organization_id, "currencies.max", enabled_count, 1)
            .await?;
    }
    CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database))
        .set_enabled(actor, &currency_code, is_enabled)
        .await
        .map_err(application_error)
}
