use aqardesk_backend::{
    application::{DashboardService, DashboardSummary},
    infrastructure::SeaOrmDashboardRepository,
};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn dashboard_get_summary(
    state: State<'_, PlatformState>,
    months: u8,
) -> Result<DashboardSummary, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_operational(&actor.organization_id().to_string())
        .await?;
    DashboardService::new(SeaOrmDashboardRepository::new(&database))
        .summary(actor, months)
        .await
        .map_err(application_error)
}
