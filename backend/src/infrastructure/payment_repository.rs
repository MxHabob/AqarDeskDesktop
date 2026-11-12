use crate::{
    application::{ApplicationError, Page, PageRequest, PaymentContract, PaymentRepository},
    domain::{
        CurrencyCode, Money, OrganizationId, Payment, PaymentAllocation, PaymentId, PaymentMethod,
        PaymentStatus, TenantId, UserId,
    },
    infrastructure::{
        audit,
        models::{payment, payment_allocation, payment_contract, payment_property_contract},
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SeaOrmPaymentRepository {
    database: DatabaseConnection,
}

impl SeaOrmPaymentRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }

    async fn fetch_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
        payment_ids: Option<Vec<String>>,
    ) -> Result<Page<Payment>, ApplicationError> {
        if payment_ids.as_ref().is_some_and(Vec::is_empty) {
            return Ok(Page::from_items(Vec::new(), page.limit()));
        }

        let mut query = payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id.to_string()));
        if let Some(payment_ids) = payment_ids {
            query = query.filter(payment::Column::Id.is_in(payment_ids));
        }
        let models = query
            .order_by_desc(payment::Column::ReceivedAt)
            .order_by_desc(payment::Column::Id)
            .limit(page.fetch_limit())
            .offset(page.offset())
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        if models.is_empty() {
            return Ok(Page::from_items(Vec::new(), page.limit()));
        }
        let payment_ids = models
            .iter()
            .map(|model| model.id.clone())
            .collect::<Vec<_>>();
        let allocations = payment_allocation::Entity::find()
            .filter(payment_allocation::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(payment_allocation::Column::PaymentId.is_in(payment_ids))
            .order_by_asc(payment_allocation::Column::ContractNumber)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut allocations_by_payment = HashMap::<String, Vec<PaymentAllocation>>::new();
        for allocation in allocations {
            allocations_by_payment
                .entry(allocation.payment_id)
                .or_default()
                .push(PaymentAllocation {
                    contract_document_id: allocation.contract_document_id,
                    contract_number: allocation.contract_number,
                    property_name: allocation.property_name,
                    property_code: allocation.property_code,
                    unit_number: allocation.unit_number,
                    amount_minor: allocation.amount_minor,
                });
        }
        let payments = models
            .into_iter()
            .map(|model| {
                let allocations = allocations_by_payment.remove(&model.id).unwrap_or_default();
                restore_payment(model, allocations)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::from_items(payments, page.limit()))
    }
}

#[async_trait::async_trait]
impl PaymentRepository for SeaOrmPaymentRepository {
    async fn list(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Payment>, ApplicationError> {
        let models = payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id.to_string()))
            .order_by_desc(payment::Column::ReceivedAt)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        if models.is_empty() {
            return Ok(Vec::new());
        }
        let payment_ids = models
            .iter()
            .map(|model| model.id.clone())
            .collect::<Vec<_>>();
        let allocations = payment_allocation::Entity::find()
            .filter(payment_allocation::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(payment_allocation::Column::PaymentId.is_in(payment_ids))
            .order_by_asc(payment_allocation::Column::ContractNumber)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut allocations_by_payment = HashMap::<String, Vec<PaymentAllocation>>::new();
        for allocation in allocations {
            allocations_by_payment
                .entry(allocation.payment_id)
                .or_default()
                .push(PaymentAllocation {
                    contract_document_id: allocation.contract_document_id,
                    contract_number: allocation.contract_number,
                    property_name: allocation.property_name,
                    property_code: allocation.property_code,
                    unit_number: allocation.unit_number,
                    amount_minor: allocation.amount_minor,
                });
        }
        models
            .into_iter()
            .map(|model| {
                let allocations = allocations_by_payment.remove(&model.id).unwrap_or_default();
                restore_payment(model, allocations)
            })
            .collect()
    }

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError> {
        self.fetch_page(organization_id, page, None).await
    }

    async fn list_page_for_property(
        &self,
        organization_id: OrganizationId,
        property_id: &str,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError> {
        let contracts = payment_property_contract::Entity::find()
            .filter(
                payment_property_contract::Column::OrganizationId.eq(organization_id.to_string()),
            )
            .filter(payment_property_contract::Column::PropertyId.eq(property_id))
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let contract_ids = contracts
            .into_iter()
            .map(|contract| contract.id)
            .collect::<Vec<_>>();
        if contract_ids.is_empty() {
            return Ok(Page::from_items(Vec::new(), page.limit()));
        }
        let allocations = payment_allocation::Entity::find()
            .filter(payment_allocation::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(payment_allocation::Column::ContractDocumentId.is_in(contract_ids))
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let payment_ids = allocations
            .into_iter()
            .map(|allocation| allocation.payment_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        self.fetch_page(organization_id, page, Some(payment_ids))
            .await
    }

    async fn insert(&self, mut payment: Payment) -> Result<Payment, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        for allocation in &mut payment.allocations {
            let contract = payment_contract::Entity::find_by_id(&allocation.contract_document_id)
                .filter(
                    payment_contract::Column::OrganizationId
                        .eq(payment.organization_id.to_string()),
                )
                .filter(payment_contract::Column::TenantId.eq(payment.tenant_id.to_string()))
                .filter(
                    payment_contract::Column::CurrencyCode
                        .eq(payment.amount.currency().as_str()),
                )
                .one(&transaction)
                .await
                .map_err(persistence_error)?
                .ok_or_else(|| {
                    crate::domain::DomainError::Validation(
                        "Each payment allocation must reference a contract for this tenant and currency.",
                    )
                })?;
            allocation.contract_number = contract.contract_number;
            allocation.property_name = contract.property_name;
            allocation.property_code = contract.property_code;
            allocation.unit_number = contract.unit_number;
        }
        let model = payment::ActiveModel {
            id: Set(payment.id.to_string()),
            organization_id: Set(payment.organization_id.to_string()),
            tenant_id: Set(payment.tenant_id.to_string()),
            received_at: Set(payment.received_at),
            amount_minor: Set(payment.amount.amount_minor()),
            currency_code: Set(payment.amount.currency().as_str().to_string()),
            payment_method: Set(payment.method.as_str().to_string()),
            external_reference: Set(payment.external_reference.clone()),
            status: Set(payment.status.as_str().to_string()),
            notes: Set(payment.notes.clone()),
            recorded_by: Set(payment.actor_id.map(|actor| actor.to_string())),
            created_at: Set(Utc::now()),
            voided_at: Set(payment.voided_at),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        for allocation in &payment.allocations {
            payment_allocation::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                organization_id: Set(payment.organization_id.to_string()),
                payment_id: Set(payment.id.to_string()),
                contract_document_id: Set(allocation.contract_document_id.clone()),
                contract_number: Set(allocation.contract_number.clone()),
                property_name: Set(allocation.property_name.clone()),
                property_code: Set(allocation.property_code.clone()),
                unit_number: Set(allocation.unit_number.clone()),
                amount_minor: Set(allocation.amount_minor),
            }
            .insert(&transaction)
            .await
            .map_err(persistence_error)?;
        }
        audit::record(
            &transaction,
            &payment.organization_id.to_string(),
            &payment
                .actor_id
                .map_or_else(String::new, |actor| actor.to_string()),
            "recorded",
            "payment",
            &payment.id.to_string(),
            serde_json::json!({
                "amount_minor": payment.amount.amount_minor(),
                "currency_code": payment.amount.currency().as_str(),
                "allocation_count": payment.allocations.len()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_payment(model, payment.allocations)
    }

    async fn contracts(
        &self,
        organization_id: OrganizationId,
        tenant_id: TenantId,
    ) -> Result<Vec<PaymentContract>, ApplicationError> {
        payment_contract::Entity::find()
            .filter(payment_contract::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(payment_contract::Column::TenantId.eq(tenant_id.to_string()))
            .filter(payment_contract::Column::Status.eq("active"))
            .order_by_asc(payment_contract::Column::ContractNumber)
            .all(&self.database)
            .await
            .map_err(persistence_error)
            .map(|contracts| {
                contracts
                    .into_iter()
                    .map(|contract| PaymentContract {
                        id: contract.id,
                        contract_number: contract.contract_number,
                        property_name: contract.property_name,
                        property_code: contract.property_code,
                        unit_number: contract.unit_number,
                        start_date: contract.start_date,
                        end_date: contract.end_date,
                        currency_code: contract.currency_code,
                        currency_decimal_digits: contract.currency_decimal_digits,
                        rent_amount_minor: contract.rent_amount_minor,
                    })
                    .collect()
            })
    }

    async fn void(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        payment_id: PaymentId,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = payment::Entity::find_by_id(payment_id.to_string())
            .filter(payment::Column::OrganizationId.eq(organization_id.to_string()))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        if existing.status != PaymentStatus::Posted.as_str() {
            return Err(ApplicationError::NotFound);
        }
        let mut active = existing.into_active_model();
        let now = Utc::now();
        active.status = Set(PaymentStatus::Void.as_str().to_string());
        active.voided_at = Set(Some(now));
        active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "voided",
            "payment",
            &payment_id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }
}

fn restore_payment(
    model: payment::Model,
    allocations: Vec<PaymentAllocation>,
) -> Result<Payment, ApplicationError> {
    let id = PaymentId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted payment identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&model.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let tenant_id = TenantId::parse(&model.tenant_id).map_err(|error| {
        tracing::error!(%error, "Persisted tenant identifier is invalid");
        ApplicationError::Persistence
    })?;
    let actor_id = model
        .recorded_by
        .map(|value| {
            UserId::parse(&value).map_err(|error| {
                tracing::error!(%error, "Persisted payment actor identifier is invalid");
                ApplicationError::Persistence
            })
        })
        .transpose()?;
    let currency = CurrencyCode::try_from(model.currency_code)?;
    let amount = Money::new(model.amount_minor, currency)?;

    Ok(Payment {
        id,
        organization_id,
        actor_id,
        tenant_id,
        received_at: model.received_at,
        amount,
        method: PaymentMethod::parse(&model.payment_method)?,
        allocations,
        external_reference: model.external_reference,
        status: PaymentStatus::parse(&model.status)?,
        notes: model.notes,
        voided_at: model.voided_at,
    })
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Payment persistence operation failed");
    ApplicationError::Persistence
}
