use crate::commands::user_cmds::{api_request, get_config, require_active_user};
use crate::connection::{AuthState, ConnectionState, DatabaseState};
use aqardesk_infrastructure::{dashboard, settings::ConnectionMode};
use reqwest::Method;
use tauri::State;

#[tauri::command]
pub async fn dashboard_get_summary(
    connection: State<'_, ConnectionState>,
    database: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    months: Option<u32>,
) -> Result<dashboard::DashboardSummary, String> {
    let user = require_active_user(&connection, &database, &auth).await?;
    let config = get_config(&connection)?;
    let months = months.unwrap_or(6);
    if !matches!(months, 3 | 6 | 12) {
        return Err("Dashboard period must be 3, 6, or 12 months.".to_string());
    }

    if config.mode == ConnectionMode::Api {
        let token = auth.api_token.read().await.clone();
        return api_request(
            &config,
            Method::GET,
            &format!("/api/v1/dashboard/summary?months={months}"),
            token,
            None,
        )
        .await;
    }

    dashboard::get_summary(&database.get().await?, &user.organization_id, months).await
}
