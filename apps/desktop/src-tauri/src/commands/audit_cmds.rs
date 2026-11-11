use aqardesk_backend::{application::AuditLog, infrastructure::SeaOrmAuditLogRepository};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn audit_list(
    state: State<'_, PlatformState>,
    limit: u64,
    offset: u64,
) -> Result<Vec<aqardesk_backend::application::AuditEntry>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "audit_log")
        .await?;
    AuditLog::new(SeaOrmAuditLogRepository::new(&database))
        .list(actor, limit, offset)
        .await
        .map_err(application_error)
}
