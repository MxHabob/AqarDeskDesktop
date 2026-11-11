use crate::{
    application::{ApplicationError, PaymentRepository},
    domain::{
        CurrencyCode, Money, OrganizationId, Payment, PaymentId, PaymentMethod, PaymentStatus,
        TenantId, UserId,
    },
    infrastructure::{audit, models::payment},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, TransactionTrait,
};

pub struct SeaOrmPaymentRepository {
    database: DatabaseConnection,
}

impl SeaOrmPaymentRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
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

        models.into_iter().map(restore_payment).collect()
    }

    async fn insert(&self, payment: Payment) -> Result<Payment, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
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
                "currency_code": payment.amount.currency().as_str()
            }),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_payment(model)
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

fn restore_payment(model: payment::Model) -> Result<Payment, ApplicationError> {
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
