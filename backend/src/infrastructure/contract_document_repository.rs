use crate::{
    application::{
        ApplicationError, ContractDocument, ContractDocumentInput, ContractDocumentRepository,
        ContractDocumentStyle, Page, PageRequest,
    },
    infrastructure::{
        audit,
        models::{organization, organization_currency, tenant, unit},
    },
};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

mod property {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "properties")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub code: String,
        pub name: String,
        pub address_line_1: String,
        pub address_line_2: Option<String>,
        pub city: String,
        pub region: Option<String>,
        pub status: String,
        pub deleted_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod currency {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "currencies")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub code: String,
        pub name: String,
        pub name_ar: Option<String>,
        pub symbol: String,
        pub symbol_position: String,
        pub decimal_digits: i16,
        pub is_active: bool,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod template {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "contract_templates")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub template_key: String,
        pub name: String,
        pub description: Option<String>,
        pub lease_type: String,
        pub locale: String,
        pub is_active: bool,
        pub created_by: Option<String>,
        pub updated_by: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod template_version {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "contract_template_versions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub template_id: String,
        pub version_number: i32,
        pub content_format: String,
        pub content_body: Option<String>,
        pub placeholders_json: String,
        pub content_sha256: String,
        pub change_note: Option<String>,
        pub created_by: Option<String>,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contract_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: String,
    organization_id: String,
    contract_number: String,
    template_id: String,
    template_version_id: String,
    template_name: String,
    template_locale: String,
    property_id: String,
    property_name: String,
    property_code: String,
    unit_id: String,
    unit_number: String,
    tenant_id: String,
    tenant_name: String,
    start_date: String,
    end_date: String,
    currency_code: String,
    currency_decimal_digits: i16,
    rent_amount_minor: i64,
    deposit_amount_minor: i64,
    document_body: String,
    docforge_document_json: Option<String>,
    docforge_context_json: Option<String>,
    document_style_json: String,
    created_by: String,
    created_at: DateTimeUtc,
    status: String,
    status_reason: Option<String>,
    status_effective_date: Option<String>,
    parent_contract_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct SeaOrmContractDocumentRepository {
    database: DatabaseConnection,
}

impl SeaOrmContractDocumentRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ContractDocumentRepository for SeaOrmContractDocumentRepository {
    async fn list(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
    ) -> Result<Vec<ContractDocument>, ApplicationError> {
        let mut query = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .order_by_desc(Column::CreatedAt);
        if let Some(property_id) = property_id {
            query = query.filter(Column::PropertyId.eq(property_id));
        }
        query
            .all(&self.database)
            .await
            .map_err(persistence_error)?
            .into_iter()
            .map(to_contract_document)
            .collect()
    }

    async fn list_page(
        &self,
        organization_id: &str,
        property_id: Option<&str>,
        page: PageRequest,
    ) -> Result<Page<ContractDocument>, ApplicationError> {
        let mut query = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id);
        if let Some(property_id) = property_id {
            query = query.filter(Column::PropertyId.eq(property_id));
        }
        let documents = query
            .limit(page.fetch_limit())
            .offset(page.offset())
            .all(&self.database)
            .await
            .map_err(persistence_error)?
            .into_iter()
            .map(to_contract_document)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::from_items(documents, page.limit()))
    }

    async fn create(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: ContractDocumentInput,
    ) -> Result<ContractDocument, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing_draft = if let Some(draft_id) = input.draft_id.as_deref() {
            Some(
                Entity::find_by_id(draft_id)
                    .filter(Column::OrganizationId.eq(organization_id))
                    .filter(Column::Status.eq("draft"))
                    .one(&transaction)
                    .await
                    .map_err(persistence_error)?
                    .ok_or(ApplicationError::NotFound)?,
            )
        } else {
            None
        };
        let parent_contract = if let Some(parent_id) = input.parent_contract_id.as_deref() {
            Some(
                Entity::find_by_id(parent_id)
                    .filter(Column::OrganizationId.eq(organization_id))
                    .filter(Column::Status.eq("active"))
                    .one(&transaction)
                    .await
                    .map_err(persistence_error)?
                    .ok_or_else(|| validation("Only an active contract can be amended."))?,
            )
        } else {
            None
        };
        let property = property::Entity::find_by_id(&input.property_id)
            .filter(property::Column::OrganizationId.eq(organization_id))
            .filter(property::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .filter(|property| property.status == "active")
            .ok_or(ApplicationError::NotFound)?;
        let unit = unit::Entity::find_by_id(&input.unit_id)
            .filter(unit::Column::OrganizationId.eq(organization_id))
            .filter(unit::Column::PropertyId.eq(&input.property_id))
            .filter(unit::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .filter(|unit| unit.status != "inactive")
            .ok_or(ApplicationError::NotFound)?;
        let tenant = tenant::Entity::find_by_id(&input.tenant_id)
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let template = template::Entity::find_by_id(&input.template_id)
            .filter(template::Column::OrganizationId.eq(organization_id))
            .filter(template::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .filter(|template| template.lease_type != "receipt")
            .ok_or(ApplicationError::NotFound)?;
        let template_version = template_version::Entity::find()
            .filter(template_version::Column::OrganizationId.eq(organization_id))
            .filter(template_version::Column::TemplateId.eq(&template.id))
            .order_by_desc(template_version::Column::VersionNumber)
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let content = template_version
            .content_body
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| validation("The selected contract template has no content."))?;
        let placeholders: serde_json::Value =
            serde_json::from_str(&template_version.placeholders_json).map_err(|error| {
                tracing::error!(%error, "Contract template settings are invalid");
                validation("Contract template settings are invalid.")
            })?;
        let organization = organization::Entity::find_by_id(organization_id)
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut document_style: ContractDocumentStyle = serde_json::from_value(
            placeholders
                .get("design")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        )
        .map_err(|error| {
            tracing::error!(%error, "Contract document design could not be decoded");
            validation("Contract document design settings are invalid.")
        })?;
        document_style.organization_logo_data_url = organization.logo_data_url.clone();
        document_style.validate()?;
        let document_style_json = serde_json::to_string(&document_style).map_err(|error| {
            tracing::error!(%error, "Contract document design could not be encoded");
            ApplicationError::Persistence
        })?;
        let currency = currency::Entity::find_by_id(&input.currency_code)
            .filter(currency::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or_else(|| validation("The selected currency is not active."))?;
        let organization_currency = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization_id))
            .filter(organization_currency::Column::CurrencyCode.eq(&input.currency_code))
            .filter(organization_currency::Column::IsEnabled.eq(true))
            .one(&transaction)
            .await
            .map_err(persistence_error)?;
        if organization_currency.is_none() {
            return Err(validation(
                "The selected currency is not enabled for this organization.",
            ));
        }

        let timezone = organization.timezone.parse::<chrono_tz::Tz>().map_err(|error| {
            tracing::error!(%error, "Organization timezone is invalid while preparing a contract");
            ApplicationError::Persistence
        })?;
        let now = Utc::now();
        let amendment_effective_date = input.start_date.clone();
        let status = input.status.clone();
        let is_draft_update = input.draft_id.is_some();
        let today = now.with_timezone(&timezone).date_naive().to_string();
        let contract_number = existing_draft
            .as_ref()
            .map(|draft| draft.contract_number.clone())
            .unwrap_or_else(|| {
                format!(
                    "AQD-{}-{}",
                    today.replace('-', ""),
                    Uuid::new_v4()
                        .simple()
                        .to_string()
                        .chars()
                        .take(8)
                        .collect::<String>()
                        .to_uppercase()
                )
            });
        let property_address = [
            Some(property.address_line_1.as_str()),
            property.address_line_2.as_deref(),
            property.region.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("، ");
        let values = HashMap::from([
            ("tenant.full_name", tenant.full_name.clone()),
            ("tenant.name", tenant.full_name.clone()),
            ("tenant.national_id", String::new()),
            (
                "tenant.email",
                tenant.email.unwrap_or_else(|| "—".to_string()),
            ),
            (
                "tenant.phone",
                tenant.phone.unwrap_or_else(|| "—".to_string()),
            ),
            (
                "tenant.address",
                tenant.mailing_address.unwrap_or_else(|| "—".to_string()),
            ),
            ("property.name", property.name.clone()),
            ("property.code", property.code.clone()),
            (
                "property.address",
                if property_address.is_empty() {
                    "—".to_string()
                } else {
                    property_address
                },
            ),
            ("property.city", property.city.clone()),
            ("unit.number", unit.unit_number.clone()),
            ("unit.type", unit.unit_type.clone()),
            (
                "unit.floor",
                unit.floor_number
                    .map_or_else(|| "—".to_string(), |floor| floor.to_string()),
            ),
            (
                "unit.area",
                unit.area_square_meters
                    .map_or_else(|| "—".to_string(), |area| format!("{area} م²")),
            ),
            ("unit.bedrooms", unit.bedrooms.to_string()),
            ("unit.bathrooms", unit.bathrooms.to_string()),
            (
                "unit.rent",
                format_amount_value(input.rent_amount_minor, currency.decimal_digits),
            ),
            ("unit.currency", currency.code.clone()),
            ("contract.number", contract_number.clone()),
            ("contract.date", today),
            ("contract.start_date", input.start_date.clone()),
            ("contract.end_date", input.end_date.clone()),
            (
                "contract.rent",
                format_amount_value(input.rent_amount_minor, currency.decimal_digits),
            ),
            (
                "contract.deposit",
                if input.deposit_amount_minor == 0 {
                    "—".to_string()
                } else {
                    format_amount(
                        input.deposit_amount_minor,
                        currency.decimal_digits,
                        &currency.code,
                    )
                },
            ),
            ("organization.name_ar", String::new()),
            ("organization.phone", String::new()),
            ("organization.website", String::new()),
            ("organization.address", String::new()),
            ("organization.logo_url", String::new()),
            ("organization.name", organization.name),
        ]);
        let document_body = resolve_template(&content, &values)?;
        if document_body.trim().is_empty() || document_body.len() > 500_000 {
            return Err(validation("The generated contract document is invalid."));
        }
        let docforge_document_json = docforge_document_snapshot(&placeholders)?;
        let docforge_context_json = docforge_document_json
            .as_ref()
            .map(|_| serialize_docforge_context(&values))
            .transpose()?;

        let id = existing_draft
            .as_ref()
            .map(|draft| draft.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let active_model = ActiveModel {
            id: Set(id.clone()),
            organization_id: Set(organization_id.to_string()),
            contract_number: Set(contract_number),
            template_id: Set(template.id),
            template_version_id: Set(template_version.id),
            template_name: Set(template.name),
            template_locale: Set(template.locale),
            property_id: Set(property.id),
            property_name: Set(property.name),
            property_code: Set(property.code),
            unit_id: Set(unit.id),
            unit_number: Set(unit.unit_number),
            tenant_id: Set(tenant.id),
            tenant_name: Set(tenant.full_name),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            currency_code: Set(currency.code),
            currency_decimal_digits: Set(currency.decimal_digits),
            rent_amount_minor: Set(input.rent_amount_minor),
            deposit_amount_minor: Set(input.deposit_amount_minor),
            document_body: Set(document_body),
            docforge_document_json: Set(docforge_document_json),
            docforge_context_json: Set(docforge_context_json),
            document_style_json: Set(document_style_json),
            created_by: Set(existing_draft
                .as_ref()
                .map(|draft| draft.created_by.clone())
                .unwrap_or_else(|| actor_id.to_string())),
            created_at: Set(existing_draft
                .as_ref()
                .map_or(now, |draft| draft.created_at)),
            status: Set(status.clone()),
            status_reason: Set(None),
            status_effective_date: Set(None),
            parent_contract_id: Set(input.parent_contract_id.clone()),
        };
        let model = if existing_draft.is_some() {
            active_model
                .update(&transaction)
                .await
                .map_err(persistence_error)?
        } else {
            active_model
                .insert(&transaction)
                .await
                .map_err(persistence_error)?
        };

        audit::record(
            &transaction,
            organization_id,
            actor_id,
            if is_draft_update {
                "draft_updated"
            } else if status == "draft" {
                "draft_created"
            } else {
                "created"
            },
            "contract_document",
            &model.id,
            serde_json::json!({
                "contract_number": model.contract_number,
                "property_id": model.property_id,
                "tenant_id": model.tenant_id,
                "status": model.status,
                "parent_contract_id": model.parent_contract_id
            }),
        )
        .await?;
        if status == "active" {
            if let Some(parent) = parent_contract {
                let mut parent = parent.into_active_model();
                parent.status = Set("amended".to_string());
                parent.status_reason = Set(Some(format!("Amended by {}", model.contract_number)));
                parent.status_effective_date = Set(Some(amendment_effective_date));
                parent
                    .update(&transaction)
                    .await
                    .map_err(persistence_error)?;
                audit::record(
                    &transaction,
                    organization_id,
                    actor_id,
                    "amended",
                    "contract_document",
                    &model
                        .parent_contract_id
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                    serde_json::json!({ "amended_by": model.id }),
                )
                .await?;
            }
        }
        transaction.commit().await.map_err(persistence_error)?;
        to_contract_document(model)
    }

    async fn delete_draft(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let contract = Entity::find_by_id(contract_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::Status.eq("draft"))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "draft_deleted",
            "contract_document",
            contract_id,
            serde_json::json!({ "contract_number": contract.contract_number }),
        )
        .await?;
        Entity::delete_by_id(contract_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;
        transaction.commit().await.map_err(persistence_error)
    }

    async fn issue_draft(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
    ) -> Result<ContractDocument, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let contract = Entity::find_by_id(contract_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::Status.eq("draft"))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if let Some(parent_id) = contract.parent_contract_id.as_deref() {
            let parent = Entity::find_by_id(parent_id)
                .filter(Column::OrganizationId.eq(organization_id))
                .filter(Column::Status.eq("active"))
                .one(&transaction)
                .await
                .map_err(persistence_error)?
                .ok_or_else(|| validation("The original contract is no longer active."))?;
            let mut active_parent = parent.into_active_model();
            active_parent.status = Set("amended".to_string());
            active_parent.status_reason =
                Set(Some(format!("Amended by {}", contract.contract_number)));
            active_parent.status_effective_date = Set(Some(contract.start_date.clone()));
            active_parent
                .update(&transaction)
                .await
                .map_err(persistence_error)?;
            audit::record(
                &transaction,
                organization_id,
                actor_id,
                "amended",
                "contract_document",
                parent_id,
                serde_json::json!({ "amended_by": contract.id }),
            )
            .await?;
        }
        let mut active_contract = contract.into_active_model();
        active_contract.status = Set("active".to_string());
        active_contract.status_reason = Set(None);
        active_contract.status_effective_date = Set(None);
        let model = active_contract
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            "issued",
            "contract_document",
            contract_id,
            serde_json::json!({ "contract_number": model.contract_number }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        to_contract_document(model)
    }

    async fn end(
        &self,
        organization_id: &str,
        actor_id: &str,
        contract_id: &str,
        input: crate::application::ContractEndInput,
    ) -> Result<ContractDocument, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let contract = Entity::find_by_id(contract_id)
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::Status.eq("active"))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let effective_date = chrono::NaiveDate::parse_from_str(&input.effective_date, "%Y-%m-%d")
            .map_err(|_| validation("Contract end date is invalid."))?;
        let start_date = chrono::NaiveDate::parse_from_str(&contract.start_date, "%Y-%m-%d")
            .map_err(|error| {
                tracing::error!(%error, "Persisted contract start date is invalid");
                ApplicationError::Persistence
            })?;
        let end_date =
            chrono::NaiveDate::parse_from_str(&contract.end_date, "%Y-%m-%d").map_err(|error| {
                tracing::error!(%error, "Persisted contract end date is invalid");
                ApplicationError::Persistence
            })?;
        if effective_date > end_date {
            return Err(validation(
                "Contract cancellation or termination cannot be after its end date.",
            ));
        }
        let status = if effective_date <= start_date {
            "cancelled"
        } else {
            "terminated"
        };
        let mut active = contract.into_active_model();
        active.status = Set(status.to_string());
        active.status_reason = Set(Some(input.reason.trim().to_string()));
        active.status_effective_date = Set(Some(input.effective_date.clone()));
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            organization_id,
            actor_id,
            status,
            "contract_document",
            contract_id,
            serde_json::json!({
                "effective_date": input.effective_date,
                "reason": input.reason.trim()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        to_contract_document(model)
    }
}

fn resolve_template(
    content: &str,
    values: &HashMap<&str, String>,
) -> Result<String, ApplicationError> {
    let mut rendered = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(open) = remaining.find("{{") {
        rendered.push_str(&remaining[..open]);
        let after_open = &remaining[open + 2..];
        let close = after_open
            .find("}}")
            .ok_or_else(|| validation("The contract template contains an incomplete field."))?;
        let key = after_open[..close].trim();
        let value = values.get(key).ok_or_else(|| {
            tracing::warn!(
                field = key,
                "Contract template contains an unsupported field"
            );
            validation("The contract template contains an unsupported field.")
        })?;
        rendered.push_str(value);
        remaining = &after_open[close + 2..];
    }
    rendered.push_str(remaining);
    if rendered.contains("}}") {
        return Err(validation(
            "The contract template contains an incomplete field.",
        ));
    }
    Ok(rendered)
}

fn docforge_document_snapshot(
    placeholders: &serde_json::Value,
) -> Result<Option<String>, ApplicationError> {
    let Some(document) = placeholders.get("docforge_document") else {
        return Ok(None);
    };
    let document_json = document
        .as_str()
        .ok_or_else(|| validation("The saved DocForge template document is invalid."))?;
    if document_json.trim().is_empty() {
        return Ok(None);
    }
    if document_json.len() > 2_000_000 {
        return Err(validation(
            "The saved DocForge template document is too large.",
        ));
    }

    let value: serde_json::Value = serde_json::from_str(document_json)
        .map_err(|_| validation("The saved DocForge template document is invalid."))?;
    let valid_page = |page: &serde_json::Value| {
        page.get("type").and_then(serde_json::Value::as_str) == Some("Page")
            && page
                .get("children")
                .and_then(serde_json::Value::as_array)
                .is_some()
    };
    let has_root = value.get("root").is_some_and(valid_page);
    let has_pages = value
        .get("pages")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|pages| !pages.is_empty() && pages.iter().all(valid_page));
    if !value.is_object()
        || !value.get("meta").is_some_and(serde_json::Value::is_object)
        || !(has_root || has_pages)
    {
        return Err(validation(
            "The saved DocForge template document is invalid.",
        ));
    }

    Ok(Some(document_json.to_string()))
}

fn serialize_docforge_context(values: &HashMap<&str, String>) -> Result<String, ApplicationError> {
    let mut context = serde_json::Map::new();
    for (path, value) in values {
        let Some((group, field)) = path.split_once('.') else {
            continue;
        };
        let group = context
            .entry(group.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        group
            .as_object_mut()
            .expect("template context groups are JSON objects")
            .insert(field.to_string(), serde_json::Value::String(value.clone()));
    }
    serde_json::to_string(&serde_json::Value::Object(context)).map_err(|error| {
        tracing::error!(%error, "Contract DocForge context could not be encoded");
        ApplicationError::Persistence
    })
}

fn format_amount(amount_minor: i64, decimal_digits: i16, currency_code: &str) -> String {
    format!(
        "{} {currency_code}",
        format_amount_value(amount_minor, decimal_digits)
    )
}

fn format_amount_value(amount_minor: i64, decimal_digits: i16) -> String {
    let scale = 10_i64.pow(decimal_digits as u32);
    let whole = amount_minor / scale;
    if decimal_digits == 0 {
        return format!("{whole}");
    }
    let fraction = amount_minor % scale;
    format!(
        "{whole}.{fraction:0width$}",
        width = decimal_digits as usize
    )
}

fn to_contract_document(model: Model) -> Result<ContractDocument, ApplicationError> {
    let document_style = serde_json::from_str(&model.document_style_json).map_err(|error| {
        tracing::error!(%error, "Saved contract document design could not be decoded");
        ApplicationError::Persistence
    })?;
    Ok(ContractDocument {
        id: model.id,
        organization_id: model.organization_id,
        contract_number: model.contract_number,
        template_id: model.template_id,
        template_version_id: model.template_version_id,
        template_name: model.template_name,
        template_locale: model.template_locale,
        property_id: model.property_id,
        property_name: model.property_name,
        property_code: model.property_code,
        unit_id: model.unit_id,
        unit_number: model.unit_number,
        tenant_id: model.tenant_id,
        tenant_name: model.tenant_name,
        start_date: model.start_date,
        end_date: model.end_date,
        currency_code: model.currency_code,
        currency_decimal_digits: model.currency_decimal_digits,
        rent_amount_minor: model.rent_amount_minor,
        deposit_amount_minor: model.deposit_amount_minor,
        document_body: model.document_body,
        docforge_document_json: model.docforge_document_json,
        docforge_context_json: model.docforge_context_json,
        document_style,
        created_by: model.created_by,
        created_at: model.created_at.to_rfc3339(),
        status: model.status,
        status_reason: model.status_reason,
        status_effective_date: model.status_effective_date,
        parent_contract_id: model.parent_contract_id,
    })
}

fn validation(message: &'static str) -> ApplicationError {
    crate::domain::DomainError::Validation(message).into()
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Contract document persistence operation failed");
    ApplicationError::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{
            ContractDocumentInput, ContractDocumentService, ContractTemplateInput,
            ContractTemplateService, IdentityService, OrganizationBootstrap,
        },
        domain::{AccessRole, ActorContext},
        infrastructure::{SeaOrmContractTemplateRepository, SeaOrmIdentityRepository},
        migration,
    };
    use sea_orm::{ConnectionTrait, Database, Statement};

    #[tokio::test]
    async fn created_documents_keep_the_template_version_and_filled_content_snapshot() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite database");
        migration::initialize(&database)
            .await
            .expect("initialize schema");
        let identities = IdentityService::new(SeaOrmIdentityRepository::new(&database));
        let (organization, admin) = identities
            .bootstrap_admin(OrganizationBootstrap {
                organization_name: "Contract documents",
                name: "Administrator",
                email: "contracts@example.com",
                password: "contract-document-test-password",
            })
            .await
            .expect("create organization and administrator");
        let actor = ActorContext::new(admin.id, organization.id, AccessRole::Admin);
        let property_id = Uuid::new_v4().to_string();
        let unit_id = Uuid::new_v4().to_string();
        let tenant_id = Uuid::new_v4().to_string();
        database
            .execute(Statement::from_string(
                database.get_database_backend(),
                format!(
                    "INSERT INTO properties (id, organization_id, code, name, property_type, address_line_1, city, country_code) VALUES ('{property_id}', '{}', 'DOC-1', 'Snapshot property', 'residential', 'Main Street', 'Riyadh', 'SA')",
                    organization.id
                ),
            ))
            .await
            .expect("insert property");
        database
            .execute(Statement::from_string(
                database.get_database_backend(),
                format!(
                    "INSERT INTO property_units (id, organization_id, property_id, unit_number, unit_type, bedrooms, bathrooms) VALUES ('{unit_id}', '{}', '{property_id}', 'A-1', 'apartment', 2, 1)",
                    organization.id
                ),
            ))
            .await
            .expect("insert unit");
        database
            .execute(Statement::from_string(
                database.get_database_backend(),
                format!(
                    "INSERT INTO tenants (id, organization_id, full_name, email) VALUES ('{tenant_id}', '{}', 'Snapshot tenant', 'tenant@example.com')",
                    organization.id
                ),
            ))
            .await
            .expect("insert tenant");

        let templates =
            ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database));
        let docforge_document_json = serde_json::json!({
            "version": "2.0",
            "meta": {
                "name": "Snapshot lease",
                "documentType": "contract",
                "entityType": "contract",
                "locale": "en",
                "pageSetup": {
                    "size": "A4",
                    "orientation": "portrait",
                    "margins": { "top": 18, "bottom": 18, "left": 18, "right": 18 },
                    "unit": "mm"
                }
            },
            "variables": {},
            "assets": {},
            "pages": [{
                "type": "Page",
                "id": "snapshot-page",
                "children": [{
                    "type": "Text",
                    "id": "tenant-name",
                    "props": {
                        "content": "{{tenant.full_name}} · {{unit.number}}",
                        "fontSize": 14,
                        "fontFamily": "Tahoma"
                    }
                }]
            }]
        })
        .to_string();
        let template_input = ContractTemplateInput {
            template_key: "snapshot-lease".to_string(),
            name: "Snapshot lease".to_string(),
            description: None,
            lease_type: "residential".to_string(),
            locale: "en".to_string(),
            content_format: "markdown".to_string(),
            content_body:
                "{{tenant.full_name}} - {{property.name}} - {{unit.number}} - {{contract.number}}"
                    .to_string(),
            placeholders_json: Some(
                serde_json::json!({
                    "design": {
                        "paper_size": "letter",
                        "accent_color": "#245A74",
                        "font_family": "Georgia",
                        "font_size": 18,
                        "alignment": "justify",
                        "page_margin_mm": 22,
                        "show_branding": true,
                        "brand_label": "Example Realty",
                        "show_footer": false
                    },
                    "docforge_document": docforge_document_json.clone()
                })
                .to_string(),
            ),
            change_note: None,
        };
        let template = templates
            .create(actor, template_input.clone())
            .await
            .expect("create contract template");
        let document =
            ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
                .create(
                    actor,
                    ContractDocumentInput {
                        draft_id: None,
                        parent_contract_id: None,
                        status: "active".to_string(),
                        template_id: template.id.clone(),
                        tenant_id: tenant_id.clone(),
                        property_id: property_id.clone(),
                        unit_id: unit_id.clone(),
                        start_date: "2026-01-01".to_string(),
                        end_date: "2026-12-31".to_string(),
                        currency_code: "SAR".to_string(),
                        rent_amount_minor: 120_000,
                        deposit_amount_minor: 0,
                    },
                )
                .await
                .expect("save generated contract document");

        let mut edited_template = template_input;
        edited_template.content_body = "A later template version".to_string();
        edited_template.placeholders_json = Some(
            r##"{"design":{"paper_size":"a4","accent_color":"#C7962F","font_family":"Tahoma","font_size":16,"alignment":"start","page_margin_mm":18,"show_branding":true,"brand_label":"","show_footer":true}}"##
                .to_string(),
        );
        edited_template.change_note = Some("Update after issue".to_string());
        templates
            .update(actor, &template.id, edited_template)
            .await
            .expect("update source template");

        let updated_document =
            ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
                .create(
                    actor,
                    ContractDocumentInput {
                        draft_id: None,
                        parent_contract_id: None,
                        status: "active".to_string(),
                        template_id: template.id.clone(),
                        tenant_id: tenant_id.clone(),
                        property_id: property_id.clone(),
                        unit_id: unit_id.clone(),
                        start_date: "2027-01-01".to_string(),
                        end_date: "2027-12-31".to_string(),
                        currency_code: "SAR".to_string(),
                        rent_amount_minor: 120_000,
                        deposit_amount_minor: 0,
                    },
                )
                .await
                .expect("new contract uses the updated template design");
        let saved = ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database))
            .list(actor, Some(&property_id))
            .await
            .expect("list saved contract for property");
        assert_eq!(saved.len(), 2);
        let original = saved
            .iter()
            .find(|item| item.id == document.id)
            .expect("find original contract");
        let updated = saved
            .iter()
            .find(|item| item.id == updated_document.id)
            .expect("find contract created with updated template");
        assert_eq!(original.template_version_id, template.versions[0].id);
        assert_eq!(
            original.document_body,
            format!(
                "Snapshot tenant - Snapshot property - A-1 - {}",
                document.contract_number
            )
        );
        assert_eq!(original.document_style.paper_size, "letter");
        assert_eq!(original.document_style.accent_color, "#245A74");
        assert_eq!(original.document_style.brand_label, "Example Realty");
        assert!(!original.document_style.show_footer);
        assert_eq!(
            original.docforge_document_json.as_deref(),
            Some(docforge_document_json.as_str())
        );
        let saved_context: serde_json::Value = serde_json::from_str(
            original
                .docforge_context_json
                .as_deref()
                .expect("custom design keeps its contract context"),
        )
        .expect("parse saved DocForge context");
        assert_eq!(saved_context["tenant"]["full_name"], "Snapshot tenant");
        assert_eq!(saved_context["property"]["name"], "Snapshot property");
        assert_eq!(saved_context["unit"]["number"], "A-1");
        assert_eq!(
            saved_context["contract"]["number"],
            document.contract_number
        );
        assert_eq!(updated.document_style.paper_size, "a4");
        assert_eq!(updated.document_style.accent_color, "#C7962F");
        assert!(updated.docforge_document_json.is_none());

        let contracts =
            ContractDocumentService::new(SeaOrmContractDocumentRepository::new(&database));
        let draft = contracts
            .create(
                actor,
                ContractDocumentInput {
                    draft_id: None,
                    parent_contract_id: None,
                    status: "draft".to_string(),
                    template_id: template.id.clone(),
                    tenant_id: tenant_id.clone(),
                    property_id: property_id.clone(),
                    unit_id: unit_id.clone(),
                    start_date: "2027-01-01".to_string(),
                    end_date: "2027-12-31".to_string(),
                    currency_code: "SAR".to_string(),
                    rent_amount_minor: 120_000,
                    deposit_amount_minor: 0,
                },
            )
            .await
            .expect("save contract draft");
        let edited_draft = contracts
            .create(
                actor,
                ContractDocumentInput {
                    draft_id: Some(draft.id.clone()),
                    parent_contract_id: None,
                    status: "draft".to_string(),
                    template_id: template.id.clone(),
                    tenant_id: tenant_id.clone(),
                    property_id: property_id.clone(),
                    unit_id: unit_id.clone(),
                    start_date: "2027-01-01".to_string(),
                    end_date: "2027-12-31".to_string(),
                    currency_code: "SAR".to_string(),
                    rent_amount_minor: 135_000,
                    deposit_amount_minor: 0,
                },
            )
            .await
            .expect("update contract draft");
        assert_eq!(edited_draft.id, draft.id);
        assert_eq!(edited_draft.status, "draft");
        assert_eq!(edited_draft.rent_amount_minor, 135_000);

        let issued = contracts
            .issue_draft(actor, &draft.id)
            .await
            .expect("issue contract draft");
        assert_eq!(issued.status, "active");
        let ended = contracts
            .end(
                actor,
                &issued.id,
                crate::application::ContractEndInput {
                    effective_date: "2027-03-01".to_string(),
                    reason: "Lease ended early".to_string(),
                },
            )
            .await
            .expect("end issued contract");
        assert_eq!(ended.status, "terminated");
        assert_eq!(ended.status_reason.as_deref(), Some("Lease ended early"));

        let amendment_draft = contracts
            .create(
                actor,
                ContractDocumentInput {
                    draft_id: None,
                    parent_contract_id: Some(updated_document.id.clone()),
                    status: "draft".to_string(),
                    template_id: template.id.clone(),
                    tenant_id: tenant_id.clone(),
                    property_id: property_id.clone(),
                    unit_id: unit_id.clone(),
                    start_date: "2027-07-01".to_string(),
                    end_date: "2027-12-31".to_string(),
                    currency_code: "SAR".to_string(),
                    rent_amount_minor: 140_000,
                    deposit_amount_minor: 0,
                },
            )
            .await
            .expect("save amendment draft");
        let amendment = contracts
            .issue_draft(actor, &amendment_draft.id)
            .await
            .expect("issue amendment");
        assert_eq!(
            amendment.parent_contract_id.as_deref(),
            Some(updated_document.id.as_str())
        );
        let saved_after_amendment = contracts
            .list(actor, Some(&property_id))
            .await
            .expect("list contracts after amendment");
        assert_eq!(
            saved_after_amendment
                .iter()
                .find(|item| item.id == updated_document.id)
                .expect("find superseded contract")
                .status,
            "amended"
        );

        let delete_draft = contracts
            .create(
                actor,
                ContractDocumentInput {
                    draft_id: None,
                    parent_contract_id: None,
                    status: "draft".to_string(),
                    template_id: template.id,
                    tenant_id,
                    property_id: property_id.clone(),
                    unit_id,
                    start_date: "2028-01-01".to_string(),
                    end_date: "2028-12-31".to_string(),
                    currency_code: "SAR".to_string(),
                    rent_amount_minor: 120_000,
                    deposit_amount_minor: 0,
                },
            )
            .await
            .expect("save draft for deletion");
        contracts
            .delete_draft(actor, &delete_draft.id)
            .await
            .expect("delete draft");
        assert!(
            contracts
                .list(actor, Some(&property_id))
                .await
                .expect("list contracts after deleting draft")
                .iter()
                .all(|item| item.id != delete_draft.id)
        );
    }
}
