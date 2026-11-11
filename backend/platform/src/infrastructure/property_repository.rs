use crate::{
    application::{ApplicationError, PropertyRepository},
    domain::{OrganizationId, Property, PropertyId, PropertyInput, PropertyStatus, PropertyType},
};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, TransactionTrait,
};

#[derive(Clone, Debug, PartialEq, sea_orm::DeriveEntityModel)]
#[sea_orm(table_name = "properties")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub organization_id: String,
    pub code: String,
    pub name: String,
    pub property_type: String,
    pub description: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub status: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: sea_orm::prelude::DateTimeUtc,
    pub updated_at: sea_orm::prelude::DateTimeUtc,
    pub deleted_at: Option<sea_orm::prelude::DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, sea_orm::EnumIter, sea_orm::DeriveRelation)]
pub enum Relation {}

impl sea_orm::ActiveModelBehavior for ActiveModel {}

pub struct SeaOrmPropertyRepository {
    database: DatabaseConnection,
}

impl SeaOrmPropertyRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl PropertyRepository for SeaOrmPropertyRepository {
    async fn list(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Property>, ApplicationError> {
        let models = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id.to_string()))
            .filter(Column::DeletedAt.is_null())
            .order_by_asc(Column::Name)
            .all(&self.database)
            .await
            .map_err(persistence_error)?;

        models.into_iter().map(restore_property).collect()
    }

    async fn insert(
        &self,
        property: Property,
        actor_id: crate::domain::UserId,
    ) -> Result<Property, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let input = &property.input;
        let model = ActiveModel {
            id: Set(property.id.to_string()),
            organization_id: Set(property.organization_id.to_string()),
            code: Set(input.code.clone()),
            name: Set(input.name.clone()),
            property_type: Set(input.property_type.as_str().to_string()),
            description: Set(input.description.clone()),
            address_line_1: Set(input.address_line_1.clone()),
            address_line_2: Set(input.address_line_2.clone()),
            city: Set(input.city.clone()),
            region: Set(input.region.clone()),
            postal_code: Set(input.postal_code.clone()),
            country_code: Set(input.country_code.clone()),
            status: Set(input.status.as_str().to_string()),
            created_by: Set(Some(actor_id.to_string())),
            updated_by: Set(Some(actor_id.to_string())),
            created_at: Set(property.created_at),
            updated_at: Set(property.updated_at),
            deleted_at: Set(None),
        };
        let model = model
            .insert(&transaction)
            .await
            .map_err(persistence_error)?;
        crate::infrastructure::audit::record(
            &transaction,
            &property.organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "property",
            &property.id.to_string(),
            serde_json::json!({"code": property.input.code}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_property(model)
    }

    async fn update(
        &self,
        property: Property,
        actor_id: crate::domain::UserId,
    ) -> Result<Property, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = Entity::find_by_id(property.id.to_string())
            .filter(Column::OrganizationId.eq(property.organization_id.to_string()))
            .filter(Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let input = &property.input;
        let mut active = existing.into_active_model();
        active.code = Set(input.code.clone());
        active.name = Set(input.name.clone());
        active.property_type = Set(input.property_type.as_str().to_string());
        active.description = Set(input.description.clone());
        active.address_line_1 = Set(input.address_line_1.clone());
        active.address_line_2 = Set(input.address_line_2.clone());
        active.city = Set(input.city.clone());
        active.region = Set(input.region.clone());
        active.postal_code = Set(input.postal_code.clone());
        active.country_code = Set(input.country_code.clone());
        active.status = Set(input.status.as_str().to_string());
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        crate::infrastructure::audit::record(
            &transaction,
            &property.organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "property",
            &property.id.to_string(),
            serde_json::json!({"code": property.input.code}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        restore_property(model)
    }

    async fn archive(
        &self,
        organization_id: OrganizationId,
        property_id: PropertyId,
        actor_id: crate::domain::UserId,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let existing = Entity::find_by_id(property_id.to_string())
            .filter(Column::OrganizationId.eq(organization_id.to_string()))
            .filter(Column::DeletedAt.is_null())
            .one(&transaction)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        let mut active = existing.into_active_model();
        let now = Utc::now();
        active.status = Set(PropertyStatus::Archived.as_str().to_string());
        active.deleted_at = Set(Some(now));
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(now);
        active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        crate::infrastructure::audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "archived",
            "property",
            &property_id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        Ok(())
    }
}

fn restore_property(model: Model) -> Result<Property, ApplicationError> {
    let id = PropertyId::parse(&model.id).map_err(|error| {
        tracing::error!(%error, "Persisted property identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&model.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let input = PropertyInput {
        code: model.code,
        name: model.name,
        property_type: PropertyType::parse(&model.property_type)?,
        description: model.description,
        address_line_1: model.address_line_1,
        address_line_2: model.address_line_2,
        city: model.city,
        region: model.region,
        postal_code: model.postal_code,
        country_code: model.country_code,
        status: PropertyStatus::parse(&model.status)?,
    };
    Property::restore(
        id,
        organization_id,
        input,
        model.created_at,
        model.updated_at,
    )
    .map_err(Into::into)
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Property persistence operation failed");
    ApplicationError::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::PropertyCatalog,
        domain::{AccessRole, ActorContext, UserId},
    };
    use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

    async fn database() -> DatabaseConnection {
        let database = Database::connect("sqlite::memory:")
            .await
            .expect("connect SQLite test database");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "CREATE TABLE organizations (id TEXT PRIMARY KEY NOT NULL)",
            ))
            .await
            .expect("create organization fixture");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE properties (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    code TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    property_type TEXT NOT NULL,
                    description TEXT,
                    address_line_1 TEXT NOT NULL,
                    address_line_2 TEXT,
                    city TEXT NOT NULL,
                    region TEXT,
                    postal_code TEXT,
                    country_code TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_by TEXT,
                    updated_by TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    deleted_at TEXT
                )"#,
            ))
            .await
            .expect("create property fixture");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE audit_events (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    actor_user_id TEXT,
                    action TEXT NOT NULL,
                    entity_type TEXT NOT NULL,
                    entity_id TEXT NOT NULL,
                    details_json TEXT,
                    created_at TEXT NOT NULL
                )"#,
            ))
            .await
            .expect("create audit fixture");
        database
    }

    fn input(code: &str, name: &str) -> PropertyInput {
        PropertyInput {
            code: code.to_string(),
            name: name.to_string(),
            property_type: PropertyType::Residential,
            description: None,
            address_line_1: "Main Street".to_string(),
            address_line_2: None,
            city: "Riyadh".to_string(),
            region: None,
            postal_code: None,
            country_code: "SA".to_string(),
            status: PropertyStatus::Active,
        }
    }

    #[tokio::test]
    async fn catalog_persists_scoped_properties_and_archives_without_deletion() {
        let database = database().await;
        let organization_id = OrganizationId::new();
        let other_organization_id = OrganizationId::new();
        let actor = ActorContext::new(UserId::new(), organization_id, AccessRole::Admin);
        let other_actor =
            ActorContext::new(UserId::new(), other_organization_id, AccessRole::Admin);
        let catalog = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database));
        let created = catalog
            .create(actor, input(" apt-1 ", "  Residence "))
            .await
            .expect("create property");

        assert_eq!(created.input.code, "APT-1");
        assert_eq!(created.input.name, "Residence");
        assert_eq!(catalog.list(actor).await.expect("list").len(), 1);
        assert!(
            catalog
                .list(other_actor)
                .await
                .expect("list another organization")
                .is_empty()
        );

        catalog
            .archive(actor, created.id)
            .await
            .expect("archive property");
        assert!(catalog.list(actor).await.expect("list").is_empty());

        let archived = Entity::find_by_id(created.id.to_string())
            .one(&database)
            .await
            .expect("read archived property")
            .expect("archived row remains");
        assert_eq!(archived.status, "archived");
        assert!(archived.deleted_at.is_some());
    }

    #[tokio::test]
    async fn update_and_archive_are_scoped_to_the_owning_organization() {
        let database = database().await;
        let organization_id = OrganizationId::new();
        let other_organization_id = OrganizationId::new();
        let actor = ActorContext::new(UserId::new(), organization_id, AccessRole::Admin);
        let other_actor =
            ActorContext::new(UserId::new(), other_organization_id, AccessRole::Admin);
        let catalog = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database));
        let created = catalog
            .create(actor, input("APT-1", "Residence"))
            .await
            .expect("create property");

        let update = catalog
            .update(other_actor, created.id, input("APT-2", "Changed"))
            .await;
        assert!(matches!(update, Err(ApplicationError::NotFound)));
        assert!(matches!(
            catalog.archive(other_actor, created.id).await,
            Err(ApplicationError::NotFound)
        ));
        assert_eq!(
            catalog.list(actor).await.expect("list")[0].input.name,
            "Residence"
        );
    }
}
