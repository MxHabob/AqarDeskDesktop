use crate::{
    licensing::LicenseStateDto,
    platform_commands::{active_actor, local_business_state},
    platform_runtime::PlatformState,
};
use aqardesk_backend::{
    application::OrganizationSettingsService, infrastructure::SeaOrmOrganizationSettingsRepository,
};
use tauri::State;

#[tauri::command]
pub async fn license_get_state(
    state: State<'_, PlatformState>,
    organization_id: Option<String>,
) -> Result<LicenseStateDto, String> {
    if let Some(organization_id) = &organization_id {
        uuid::Uuid::parse_str(organization_id)
            .map_err(|_| "The organization identifier is invalid.".to_string())?;
    }
    Ok(state.license.state(organization_id.as_deref()).await)
}

#[tauri::command]
pub async fn license_activate_key(
    state: State<'_, PlatformState>,
    product_key: String,
) -> Result<LicenseStateDto, String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization =
        OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database))
            .get(actor)
            .await
            .map_err(|error| {
                tracing::error!(%error, "Could not read the organization for license activation");
                "Could not read organization details for license activation.".to_string()
            })?;
    state
        .license
        .activate(&product_key, &organization.id, &organization.name)
        .await
}

#[tauri::command]
pub async fn license_start_trial(
    state: State<'_, PlatformState>,
) -> Result<LicenseStateDto, String> {
    let (actor, database) = local_business_state(&state).await?;
    let settings =
        OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database))
            .get(actor)
            .await
            .map_err(|error| {
                tracing::error!(%error, "Could not read the organization for trial registration");
                "Could not read organization details for trial registration.".to_string()
            })?;
    state
        .license
        .start_trial(&settings.id, &settings.name)
        .await
}

#[tauri::command]
pub async fn license_refresh(state: State<'_, PlatformState>) -> Result<LicenseStateDto, String> {
    let actor = active_actor(&state).await?;
    state
        .license
        .refresh(Some(&actor.organization_id().to_string()))
        .await
}

#[tauri::command]
pub async fn license_deactivate(
    state: State<'_, PlatformState>,
) -> Result<LicenseStateDto, String> {
    let actor = active_actor(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .deactivate_for_organization(&organization_id)
        .await
}
