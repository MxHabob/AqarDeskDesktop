use aqardesk_backend::{
    application::{ProfileService, ProfileSettingsInput},
    infrastructure::SeaOrmProfileRepository,
};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn profile_get_settings(
    state: State<'_, PlatformState>,
) -> Result<aqardesk_backend::application::ProfileSettings, String> {
    let (actor, database) = local_business_state(&state).await?;
    ProfileService::new(SeaOrmProfileRepository::new(&database))
        .get(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn profile_update_settings(
    state: State<'_, PlatformState>,
    input: ProfileSettingsInput,
) -> Result<aqardesk_backend::application::ProfileSettings, String> {
    let (actor, database) = local_business_state(&state).await?;
    ProfileService::new(SeaOrmProfileRepository::new(&database))
        .update(actor, input)
        .await
        .map_err(application_error)
}
