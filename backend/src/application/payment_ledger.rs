use crate::{
    application::{ApplicationError, require_permission},
    domain::{ActorContext, OrganizationId, Payment, PaymentId, PaymentInput, Permission},
};

#[async_trait::async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn list(&self, organization_id: OrganizationId)
    -> Result<Vec<Payment>, ApplicationError>;

    async fn insert(&self, payment: Payment) -> Result<Payment, ApplicationError>;

    async fn void(
        &self,
        organization_id: OrganizationId,
        actor_id: crate::domain::UserId,
        payment_id: PaymentId,
    ) -> Result<(), ApplicationError>;
}

pub struct PaymentLedger<R> {
    payments: R,
}

impl<R: PaymentRepository> PaymentLedger<R> {
    pub fn new(payments: R) -> Self {
        Self { payments }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Payment>, ApplicationError> {
        require_permission(actor, Permission::ViewPayments)?;
        self.payments.list(actor.organization_id()).await
    }

    pub async fn record(
        &self,
        actor: ActorContext,
        input: PaymentInput,
    ) -> Result<Payment, ApplicationError> {
        require_permission(actor, Permission::RecordPayments)?;
        let payment = Payment::record(
            PaymentId::new(),
            actor.organization_id(),
            actor.user_id(),
            input,
        )?;
        self.payments.insert(payment).await
    }

    pub async fn void(
        &self,
        actor: ActorContext,
        payment_id: PaymentId,
    ) -> Result<(), ApplicationError> {
        require_permission(actor, Permission::VoidPayments)?;
        self.payments
            .void(actor.organization_id(), actor.user_id(), payment_id)
            .await
    }
}
