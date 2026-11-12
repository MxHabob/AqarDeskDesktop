use aqardesk_backend::{
    application::{ContractTemplateInput, ContractTemplateService, Page, PageRequest},
    infrastructure::SeaOrmContractTemplateRepository,
};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn contract_templates_list(
    state: State<'_, PlatformState>,
) -> Result<Vec<aqardesk_backend::application::ContractTemplate>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_templates_list_page(
    state: State<'_, PlatformState>,
    limit: u64,
    offset: u64,
) -> Result<Page<aqardesk_backend::application::ContractTemplate>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    let page = PageRequest::new(limit, offset).map_err(application_error)?;
    let templates = ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)?;
    let items = templates
        .into_iter()
        .skip(usize::try_from(page.offset()).unwrap_or(usize::MAX))
        .take(usize::try_from(page.fetch_limit()).unwrap_or(usize::MAX))
        .collect();
    Ok(Page::from_items(items, page.limit()))
}

#[tauri::command]
pub async fn contract_templates_available(
    state: State<'_, PlatformState>,
) -> Result<Vec<aqardesk_backend::application::ContractTemplate>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .list_active(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_templates_create(
    state: State<'_, PlatformState>,
    input: ContractTemplateInput,
) -> Result<aqardesk_backend::application::ContractTemplate, String> {
    let (actor, database) = local_business_state(&state).await?;
    let organization_id = actor.organization_id().to_string();
    state
        .license
        .require_feature(&organization_id, "contract_templates")
        .await?;
    let _quota_guard = state.license_quota_lock.lock().await;
    let service = ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database));
    let current_count = service.list(actor).await.map_err(application_error)?.len() as u64;
    state
        .license
        .check_limit_for_organization(&organization_id, "contract_templates.max", current_count, 1)
        .await?;
    service
        .create(actor, input)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_templates_update(
    state: State<'_, PlatformState>,
    template_id: String,
    input: ContractTemplateInput,
) -> Result<aqardesk_backend::application::ContractTemplate, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .update(actor, &template_id, input)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_templates_set_active(
    state: State<'_, PlatformState>,
    template_id: String,
    is_active: bool,
) -> Result<(), String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .set_active(actor, &template_id, is_active)
        .await
        .map_err(application_error)
}
