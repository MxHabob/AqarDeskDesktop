use aqardesk_backend::{
    application::{ContractDocumentInput, ContractDocumentService, ContractEndInput},
    infrastructure::SeaOrmContractDocumentRepository,
};
use tauri::State;

use crate::{
    platform_commands::{application_error, local_business_state},
    platform_runtime::PlatformState,
};

#[tauri::command]
pub async fn contract_documents_list(
    state: State<'_, PlatformState>,
    property_id: Option<String>,
) -> Result<Vec<aqardesk_backend::application::ContractDocument>, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .list(actor, property_id.as_deref())
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_documents_list_page(
    state: State<'_, PlatformState>,
    property_id: Option<String>,
    limit: u64,
    offset: u64,
) -> Result<
    aqardesk_backend::application::Page<aqardesk_backend::application::ContractDocument>,
    String,
> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    let page = aqardesk_backend::application::PageRequest::new(limit, offset)
        .map_err(application_error)?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .list_page(actor, property_id.as_deref(), page)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_documents_create(
    state: State<'_, PlatformState>,
    input: ContractDocumentInput,
) -> Result<aqardesk_backend::application::ContractDocument, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .create(actor, input)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_documents_delete_draft(
    state: State<'_, PlatformState>,
    contract_id: String,
) -> Result<(), String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .delete_draft(actor, &contract_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_documents_issue_draft(
    state: State<'_, PlatformState>,
    contract_id: String,
) -> Result<aqardesk_backend::application::ContractDocument, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .issue_draft(actor, &contract_id)
        .await
        .map_err(application_error)
}

#[tauri::command]
pub async fn contract_documents_end(
    state: State<'_, PlatformState>,
    contract_id: String,
    input: ContractDocumentEndInput,
) -> Result<aqardesk_backend::application::ContractDocument, String> {
    let (actor, database) = local_business_state(&state).await?;
    state
        .license
        .require_feature(&actor.organization_id().to_string(), "contract_templates")
        .await?;
    ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
        .end(actor, &contract_id, input.into())
        .await
        .map_err(application_error)
}

#[derive(Debug, serde::Deserialize)]
pub struct ContractDocumentEndInput {
    pub effective_date: String,
    pub reason: String,
}

impl From<ContractDocumentEndInput> for ContractEndInput {
    fn from(value: ContractDocumentEndInput) -> Self {
        Self {
            effective_date: value.effective_date,
            reason: value.reason,
        }
    }
}
