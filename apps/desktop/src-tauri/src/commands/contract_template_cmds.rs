use aqardesk_backend::{
    application::{ContractTemplateInput, ContractTemplateService},
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
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .list(actor)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_templates_create(
    state: State<'_, PlatformState>,
    input: ContractTemplateInput,
) -> Result<aqardesk_backend::application::ContractTemplate, String> {
    let (actor, database) = local_business_state(&state).await?;
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
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
    ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database))
        .set_active(actor, &template_id, is_active)
        .await
        .map_err(application_error)
}
