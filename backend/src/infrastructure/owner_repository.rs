use crate::{
    application::{ApplicationError, OwnerRepository, Page, PageRequest},
    domain::{OrganizationId, Owner, OwnerId, OwnerInput, UserId},
    infrastructure::{
        audit,
        models::{owner, owner_phone},
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SeaOrmOwnerRepository {
    database: DatabaseConnection,
}

impl SeaOrmOwnerRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl OwnerRepository for SeaOrmOwnerRepository {
    async fn list(&self, organization_id: OrganizationId) -> Result<Vec<Owner>, ApplicationError> {
        let models = owner::Entity::find()
            .filter(owner::Column::OrganizationId.eq(organization_id.to_string()))
            .order_by_asc(owner::Column::Name)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        if models.is_empty() {
            return Ok(Vec::new());
        }

        let owner_ids = models
            .iter()
            .map(|model| model.id.clone())
            .collect::<Vec<_>>();
        let phones = owner_phone::Entity::find()
            .filter(owner_phone::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(owner_phone::Column::OwnerId.is_in(owner_ids))
            .order_by_asc(owner_phone::Column::CreatedAt)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut phones_by_owner = HashMap::new();
        for phone in phones {
            phones_by_owner
                .entry(phone.owner_id.clone())
                .or_insert(phone);
        }

        models
            .into_iter()
            .map(|model| {
                let phone = phones_by_owner.get(&model.id).ok_or_else(|| {
                    tracing::error!(owner_id = %model.id, "Persisted owner has no contact phone");
                    ApplicationError::Persistence
                })?;
                restore_owner(model, phone)
            })
            .collect()
    }

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        page: PageRequest,
    ) -> Result<Page<Owner>, ApplicationError> {
        let models = owner::Entity::find()
            .filter(owner::Column::OrganizationId.eq(organization_id.to_string()))
            .order_by_asc(owner::Column::Name)
            .order_by_asc(owner::Column::Id)
            .limit(page.fetch_limit())
            .offset(page.offset())
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        if models.is_empty() {
            return Ok(Page::from_items(Vec::new(), page.limit()));
        }

        let owner_ids = models
            .iter()
            .map(|model| model.id.clone())
            .collect::<Vec<_>>();
        let phones = owner_phone::Entity::find()
            .filter(owner_phone::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(owner_phone::Column::OwnerId.is_in(owner_ids))
            .order_by_asc(owner_phone::Column::CreatedAt)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;
        let mut phones_by_owner = HashMap::new();
        for phone in phones {
            phones_by_owner
                .entry(phone.owner_id.clone())
                .or_insert(phone);
        }
        let owners = models
            .into_iter()
            .map(|model| {
                let phone = phones_by_owner.get(&model.id).ok_or_else(|| {
                    tracing::error!(owner_id = %model.id, "Persisted owner has no contact phone");
                    ApplicationError::Persistence
                })?;
                restore_owner(model, phone)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::from_items(owners, page.limit()))
    }

    async fn insert(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let model = owner::ActiveModel {
            id: Set(owner.id.to_string()),
            organization_id: Set(owner.organization_id.to_string()),
            name: Set(owner.input.name.clone()),
            created_at: Set(owner.created_at),
            updated_at: Set(owner.updated_at),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        let phone = insert_phone(
            &transaction,
            &owner.organization_id.to_string(),
            &owner.id.to_string(),
            &owner.input,
        )
        .await?;
        audit::record(
            &transaction,
            &owner.organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "owner",
            &owner.id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_owner(model, &phone)
    }

    async fn update(&self, owner: Owner, actor_id: UserId) -> Result<Owner, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = owner::Entity::find_by_id(owner.id.to_string())
            .filter(owner::Column::OrganizationId.eq(owner.organization_id.to_string()))
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut active = existing.into_active_model();
        active.name = Set(owner.input.name.clone());
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;

        let existing_phone = owner_phone::Entity::find()
            .filter(owner_phone::Column::OrganizationId.eq(owner.organization_id.to_string()))
            .filter(owner_phone::Column::OwnerId.eq(owner.id.to_string()))
            .order_by_asc(owner_phone::Column::CreatedAt)
            .one(&transaction)
            .await
            .map_err(persistence_error)?;
        let phone = if let Some(existing_phone) = existing_phone {
            let mut active = existing_phone.into_active_model();
            active.phone_number = Set(owner.input.phone_number.clone());
            active.label = Set(owner.input.phone_label.clone());
            active.is_whatsapp = Set(owner.input.is_whatsapp);
            active
                .update(&transaction)
                .await
                .map_err(persistence_error)?
        } else {
            insert_phone(
                &transaction,
                &owner.organization_id.to_string(),
                &owner.id.to_string(),
                &owner.input,
            )
            .await?
        };
        audit::record(
            &transaction,
            &owner.organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "owner",
            &owner.id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_owner(model, &phone)
    }
}

async fn insert_phone<C: sea_orm::ConnectionTrait>(
    database: &C,
    organization_id: &str,
    owner_id: &str,
    input: &OwnerInput,
) -> Result<owner_phone::Model, ApplicationError> {
    owner_phone::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        organization_id: Set(organization_id.to_string()),
        owner_id: Set(owner_id.to_string()),
        phone_number: Set(input.phone_number.clone()),
        label: Set(input.phone_label.clone()),
        is_whatsapp: Set(input.is_whatsapp),
        created_at: Set(Utc::now()),
    }
    .insert(database)
    .await
    .map_err(persistence_error)
}

fn restore_owner(
    model: owner::Model,
    phone: &owner_phone::Model,
) -> Result<Owner, ApplicationError> {
    let id = OwnerId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted owner identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&model.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let input = OwnerInput {
        name: model.name,
        phone_number: phone.phone_number.clone(),
        phone_label: phone.label.clone(),
        is_whatsapp: phone.is_whatsapp,
    };
    Owner::restore(
        id,
        organization_id,
        input,
        model.created_at,
        model.updated_at,
    )
    .map_err(Into::into)
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Owner persistence operation failed");
    ApplicationError::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::OwnerDirectory,
        domain::{AccessRole, ActorContext},
    };
    use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

    async fn database() -> DatabaseConnection {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite test database");
        for sql in [
            "CREATE TABLE owners (id TEXT PRIMARY KEY NOT NULL, organization_id TEXT NOT NULL, name TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, UNIQUE (id, organization_id))",
            "CREATE TABLE owner_phones (id TEXT PRIMARY KEY NOT NULL, organization_id TEXT NOT NULL, owner_id TEXT NOT NULL, phone_number TEXT NOT NULL, label TEXT, is_whatsapp BOOLEAN NOT NULL, created_at TEXT NOT NULL, UNIQUE (owner_id, phone_number))",
            "CREATE TABLE audit_events (id TEXT PRIMARY KEY NOT NULL, organization_id TEXT NOT NULL, actor_user_id TEXT, action TEXT NOT NULL, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, details_json TEXT, created_at TEXT NOT NULL)",
        ] {
            database
                .execute(Statement::from_string(DbBackend::Sqlite, sql))
                .await
                .expect("create owner persistence fixture");
        }
        database
    }

    fn actor(organization_id: OrganizationId, role: AccessRole) -> crate::domain::ActorContext {
        ActorContext::new(UserId::new(), organization_id, role)
    }

    fn input(name: &str, phone_number: &str) -> OwnerInput {
        OwnerInput {
            name: name.to_string(),
            phone_number: phone_number.to_string(),
            phone_label: Some("Mobile".to_string()),
            is_whatsapp: true,
        }
    }

    #[tokio::test]
    async fn owner_directory_creates_updates_and_scopes_records_by_organization() {
        let database = database().await;
        let organization_id = OrganizationId::new();
        let directory = OwnerDirectory::new(SeaOrmOwnerRepository::new(&database));
        let created = directory
            .create(
                actor(organization_id, AccessRole::Admin),
                input("Owner One", "0500000000"),
            )
            .await
            .expect("create owner");

        assert_eq!(created.input.phone_number, "0500000000");
        assert_eq!(created.input.phone_label.as_deref(), Some("Mobile"));
        assert!(
            directory
                .list(actor(OrganizationId::new(), AccessRole::Admin))
                .await
                .expect("list another organization")
                .is_empty()
        );
        assert!(matches!(
            directory
                .update(
                    actor(OrganizationId::new(), AccessRole::Admin),
                    created.id,
                    input("Unauthorized", "0500000001")
                )
                .await,
            Err(ApplicationError::NotFound)
        ));
        assert!(matches!(
            directory
                .create(
                    actor(organization_id, AccessRole::Readonly),
                    input("Readonly", "0500000002")
                )
                .await,
            Err(ApplicationError::Forbidden)
        ));

        let updated = directory
            .update(
                actor(organization_id, AccessRole::Admin),
                created.id,
                input("Updated Owner", "0550000000"),
            )
            .await
            .expect("update owner");
        assert_eq!(updated.input.name, "Updated Owner");
        assert_eq!(updated.input.phone_number, "0550000000");
        assert_eq!(
            directory
                .list(actor(organization_id, AccessRole::Admin))
                .await
                .expect("list owner")
                .len(),
            1
        );
    }
}
