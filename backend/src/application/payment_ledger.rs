use crate::{
    application::{require_permission, ApplicationError, Page, PageRequest},
    domain::{
        ActorContext, OrganizationId, Payment, PaymentId, PaymentInput, Permission, TenantId,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentContract {
    pub id: String,
    pub contract_number: String,
    pub property_name: String,
    pub property_code: String,
    pub unit_number: String,
    pub start_date: String,
    pub end_date: String,
    pub currency_code: String,
    pub currency_decimal_digits: i16,
    pub rent_amount_minor: i64,
}

#[async_trait::async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn list(&self, organization_id: OrganizationId)
        -> Result<Vec<Payment>, ApplicationError>;

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError> {
        Ok(Page::from_items(
            self.list(organization_id).await?,
            page.limit(),
        ))
    }

    async fn list_page_for_property(
        &self,
        organization_id: OrganizationId,
        property_id: &str,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError>;

    async fn insert(&self, payment: Payment) -> Result<Payment, ApplicationError>;

    async fn contracts(
        &self,
        organization_id: OrganizationId,
        tenant_id: TenantId,
    ) -> Result<Vec<PaymentContract>, ApplicationError>;

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

    pub async fn list_page(
        &self,
        actor: ActorContext,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError> {
        require_permission(actor, Permission::ViewPayments)?;
        self.payments.list_page(actor.organization_id(), page).await
    }

    pub async fn list_page_for_property(
        &self,
        actor: ActorContext,
        property_id: &str,
        page: PageRequest,
    ) -> Result<Page<Payment>, ApplicationError> {
        require_permission(actor, Permission::ViewPayments)?;
        self.payments
            .list_page_for_property(actor.organization_id(), property_id, page)
            .await
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

    pub async fn contracts(
        &self,
        actor: ActorContext,
        tenant_id: TenantId,
    ) -> Result<Vec<PaymentContract>, ApplicationError> {
        require_permission(actor, Permission::RecordPayments)?;
        self.payments
            .contracts(actor.organization_id(), tenant_id)
            .await
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
