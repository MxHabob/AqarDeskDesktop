use crate::models::{audit_event, currency, organization_currency, payment, tenant};
use aqardesk_app::{
    AppError,
    services::payments::{Payment, PaymentInput, PaymentRepository},
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SeaOrmPaymentRepository<'db> {
    database: &'db DatabaseConnection,
}

impl<'db> SeaOrmPaymentRepository<'db> {
    pub fn new(database: &'db DatabaseConnection) -> Self {
        Self { database }
    }

    async fn insert_audit(
        &self,
        database: &impl sea_orm::ConnectionTrait,
        organization_id: &str,
        actor_id: &str,
        action: &str,
        payment_id: &str,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        audit_event::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(organization_id.to_string()),
            actor_user_id: Set(Some(actor_id.to_string())),
            action: Set(action.to_string()),
            entity_type: Set("payment".to_string()),
            entity_id: Set(payment_id.to_string()),
            details_json: Set(Some(details.to_string())),
            created_at: Set(Utc::now()),
        }
        .insert(database)
        .await
        .map(|_| ())
        .map_err(|error| {
            AppError::Persistence(format!("Failed to write payment audit event: {error}"))
        })
    }
}

#[async_trait::async_trait]
impl PaymentRepository for SeaOrmPaymentRepository<'_> {
    async fn list(&self, organization_id: &str) -> Result<Vec<Payment>, AppError> {
        let payments = payment::Entity::find()
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .order_by_desc(payment::Column::ReceivedAt)
            .all(self.database)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to list payments: {error}")))?;
        let tenant_ids = payments
            .iter()
            .map(|item| item.tenant_id.clone())
            .collect::<Vec<_>>();
        let tenants = if tenant_ids.is_empty() {
            Vec::new()
        } else {
            tenant::Entity::find()
                .filter(tenant::Column::OrganizationId.eq(organization_id))
                .filter(tenant::Column::Id.is_in(tenant_ids))
                .all(self.database)
                .await
                .map_err(|error| {
                    AppError::Persistence(format!("Failed to resolve payment tenants: {error}"))
                })?
        };
        let tenant_names = tenants
            .into_iter()
            .map(|item| (item.id, item.full_name))
            .collect::<HashMap<_, _>>();
        payments
            .into_iter()
            .map(|item| {
                let tenant_name = tenant_names.get(&item.tenant_id).cloned().ok_or_else(|| {
                    AppError::Persistence(format!(
                        "Payment {} references a tenant that could not be loaded.",
                        item.id
                    ))
                })?;
                Ok(payment_summary(item, tenant_name))
            })
            .collect()
    }

    async fn record(
        &self,
        organization_id: &str,
        actor_id: &str,
        input: PaymentInput,
    ) -> Result<Payment, AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin payment recording: {error}"))
        })?;
        let tenant_model = tenant::Entity::find_by_id(&input.tenant_id)
            .filter(tenant::Column::OrganizationId.eq(organization_id))
            .filter(tenant::Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to validate payment tenant: {error}"))
            })?
            .ok_or_else(|| AppError::NotFound("Active tenant was not found.".to_string()))?;
        let currency_exists = currency::Entity::find_by_id(&input.currency_code)
            .filter(currency::Column::IsActive.eq(true))
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to validate payment currency: {error}"))
            })?
            .is_some();
        let currency_enabled = organization_currency::Entity::find()
            .filter(organization_currency::Column::OrganizationId.eq(organization_id))
            .filter(organization_currency::Column::CurrencyCode.eq(&input.currency_code))
            .filter(organization_currency::Column::IsEnabled.eq(true))
            .one(&transaction)
            .await
            .map_err(|error| {
                AppError::Persistence(format!("Failed to validate organization currency: {error}"))
            })?
            .is_some();
        if !currency_exists || !currency_enabled {
            return Err(AppError::Validation(
                "Payment currency must be active and enabled for this organization.".to_string(),
            ));
        }
        let received_at = DateTime::parse_from_rfc3339(&input.received_at)
            .map_err(|error| AppError::Validation(format!("Invalid payment date: {error}")))?
            .with_timezone(&Utc);
        let model = payment::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(organization_id.to_string()),
            tenant_id: Set(input.tenant_id),
            received_at: Set(received_at),
            amount_minor: Set(input.amount_minor),
            currency_code: Set(input.currency_code),
            payment_method: Set(input.payment_method),
            external_reference: Set(input.external_reference),
            status: Set("posted".to_string()),
            notes: Set(input.notes),
            recorded_by: Set(Some(actor_id.to_string())),
            created_at: Set(Utc::now()),
            voided_at: Set(None),
        }
        .insert(&transaction)
        .await
        .map_err(|error| AppError::Persistence(format!("Failed to record payment: {error}")))?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            "recorded",
            &model.id,
            serde_json::json!({
                "tenant_id": model.tenant_id,
                "amount_minor": model.amount_minor,
                "currency_code": model.currency_code,
                "payment_method": model.payment_method,
            }),
        )
        .await?;
        transaction.commit().await.map_err(|error| {
            AppError::Persistence(format!("Failed to save payment record: {error}"))
        })?;
        Ok(payment_summary(model, tenant_model.full_name))
    }

    async fn void(
        &self,
        organization_id: &str,
        actor_id: &str,
        payment_id: &str,
    ) -> Result<(), AppError> {
        let transaction = self.database.begin().await.map_err(|error| {
            AppError::Persistence(format!("Failed to begin payment void: {error}"))
        })?;
        let model = payment::Entity::find_by_id(payment_id)
            .filter(payment::Column::OrganizationId.eq(organization_id))
            .one(&transaction)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to find payment: {error}")))?
            .ok_or_else(|| AppError::NotFound("Payment was not found.".to_string()))?;
        if model.status != "posted" {
            return Err(AppError::Validation(
                "Only posted payments can be voided.".to_string(),
            ));
        }
        let mut active = model.into_active_model();
        active.status = Set("void".to_string());
        active.voided_at = Set(Some(Utc::now()));
        active
            .update(&transaction)
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to void payment: {error}")))?;
        self.insert_audit(
            &transaction,
            organization_id,
            actor_id,
            "voided",
            payment_id,
            serde_json::json!({}),
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| AppError::Persistence(format!("Failed to save payment void: {error}")))
    }
}

fn payment_summary(model: payment::Model, tenant_name: String) -> Payment {
    Payment {
        id: model.id,
        tenant_id: model.tenant_id,
        tenant_name,
        received_at: model.received_at.to_rfc3339(),
        amount_minor: model.amount_minor,
        currency_code: model.currency_code,
        payment_method: model.payment_method,
        external_reference: model.external_reference,
        status: model.status,
        notes: model.notes,
        voided_at: model.voided_at.map(|value| value.to_rfc3339()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aqardesk_app::services::payments::{PaymentInput, PaymentService};
    use sea_orm::{ActiveModelTrait, Database, EntityTrait, QuerySelect};

    #[tokio::test]
    async fn payment_receipts_are_scoped_audited_and_voided_without_deletion() {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect in-memory database");
        crate::run_migrations(&database)
            .await
            .expect("apply migrations");
        let admin = crate::users::create_initial_admin(
            &database,
            "AqarDesk",
            "Admin",
            "admin@example.com",
            "a-secure-password",
        )
        .await
        .expect("create initial administrator");
        let tenant_model = tenant::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            organization_id: Set(admin.organization_id.clone()),
            full_name: Set("Tenant One".to_string()),
            email: Set(Some("tenant@example.com".to_string())),
            phone: Set(None),
            mailing_address: Set(None),
            notes: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            deleted_at: Set(None),
        }
        .insert(&database)
        .await
        .expect("create tenant");
        let second_organization_id = Uuid::new_v4().to_string();
        crate::models::organization::ActiveModel {
            id: Set(second_organization_id.clone()),
            name: Set("Other organization".to_string()),
            legal_name: Set(None),
            default_currency: Set("SAR".to_string()),
            timezone: Set("Asia/Riyadh".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&database)
        .await
        .expect("create second organization");

        let service = PaymentService::new(SeaOrmPaymentRepository::new(&database));
        let payment = service
            .record(
                &admin.organization_id,
                &admin.id,
                PaymentInput {
                    tenant_id: tenant_model.id,
                    received_at: "2026-11-11T12:00:00Z".to_string(),
                    amount_minor: 12500,
                    currency_code: "sar".to_string(),
                    payment_method: "cash".to_string(),
                    external_reference: Some("RCPT-001".to_string()),
                    notes: None,
                },
            )
            .await
            .expect("record payment");
        assert_eq!(payment.tenant_name, "Tenant One");
        assert_eq!(payment.status, "posted");

        assert_eq!(
            service
                .list(&admin.organization_id)
                .await
                .expect("list organization payments")
                .len(),
            1
        );
        assert!(
            service
                .list(&second_organization_id)
                .await
                .expect("list other organization's payments")
                .is_empty()
        );

        service
            .void(&admin.organization_id, &admin.id, &payment.id)
            .await
            .expect("void payment");
        let payments = service
            .list(&admin.organization_id)
            .await
            .expect("list voided payment");
        assert_eq!(payments.len(), 1);
        assert_eq!(payments[0].status, "void");
        assert!(payments[0].voided_at.is_some());
        assert!(
            service
                .void(&admin.organization_id, &admin.id, &payment.id)
                .await
                .is_err()
        );
    }
}
