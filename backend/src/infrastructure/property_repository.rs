use crate::{
    application::{ApplicationError, PropertyRepository},
    domain::{OrganizationId, Property, PropertyId, PropertyInput, PropertyStatus, PropertyType},
    infrastructure::models::{owner, owner_phone},
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, TransactionTrait,
};
use std::collections::HashMap;

mod property_owner {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "property_owners")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub property_id: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub owner_id: String,
        pub organization_id: String,
        pub ownership_share_percent: Option<Decimal>,
        pub sort_order: i32,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

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
    pub district: Option<String>,
    pub neighborhood: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub lot_area_square_meters: Option<Decimal>,
    pub building_area_square_meters: Option<Decimal>,
    pub frontage_meters: Option<Decimal>,
    pub depth_meters: Option<Decimal>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
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

        let owners = load_property_owners(&self.database, &models).await?;
        models
            .into_iter()
            .map(|model| {
                let property_owners = owners.get(&model.id).cloned().unwrap_or_default();
                restore_property(model, property_owners)
            })
            .collect()
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
            district: Set(input.district.clone()),
            neighborhood: Set(input.neighborhood.clone()),
            postal_code: Set(input.postal_code.clone()),
            country_code: Set(input.country_code.clone()),
            lot_area_square_meters: Set(input.lot_area_square_meters),
            building_area_square_meters: Set(input.building_area_square_meters),
            frontage_meters: Set(input.frontage_meters),
            depth_meters: Set(input.depth_meters),
            latitude: Set(input.latitude),
            longitude: Set(input.longitude),
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
        sync_property_owners(
            &transaction,
            &property.organization_id.to_string(),
            &property.id.to_string(),
            &input.owners,
        )
        .await?;
        let owners = load_property_owners(&transaction, std::slice::from_ref(&model)).await?;
        let property_owners = owners.get(&model.id).cloned().unwrap_or_default();
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
        restore_property(model, property_owners)
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
        active.district = Set(input.district.clone());
        active.neighborhood = Set(input.neighborhood.clone());
        active.postal_code = Set(input.postal_code.clone());
        active.country_code = Set(input.country_code.clone());
        active.lot_area_square_meters = Set(input.lot_area_square_meters);
        active.building_area_square_meters = Set(input.building_area_square_meters);
        active.frontage_meters = Set(input.frontage_meters);
        active.depth_meters = Set(input.depth_meters);
        active.latitude = Set(input.latitude);
        active.longitude = Set(input.longitude);
        active.status = Set(input.status.as_str().to_string());
        active.updated_by = Set(Some(actor_id.to_string()));
        active.updated_at = Set(Utc::now());
        let model = active
            .update(&transaction)
            .await
            .map_err(persistence_error)?;
        sync_property_owners(
            &transaction,
            &property.organization_id.to_string(),
            &property.id.to_string(),
            &input.owners,
        )
        .await?;
        let owners = load_property_owners(&transaction, std::slice::from_ref(&model)).await?;
        let property_owners = owners.get(&model.id).cloned().unwrap_or_default();
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
        restore_property(model, property_owners)
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

fn restore_property(
    model: Model,
    owners: Vec<crate::domain::PropertyOwnerInput>,
) -> Result<Property, ApplicationError> {
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
        district: model.district,
        neighborhood: model.neighborhood,
        postal_code: model.postal_code,
        country_code: model.country_code,
        lot_area_square_meters: model.lot_area_square_meters,
        building_area_square_meters: model.building_area_square_meters,
        frontage_meters: model.frontage_meters,
        depth_meters: model.depth_meters,
        latitude: model.latitude,
        longitude: model.longitude,
        owners,
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

async fn load_property_owners<C: ConnectionTrait>(
    database: &C,
    properties: &[Model],
) -> Result<HashMap<String, Vec<crate::domain::PropertyOwnerInput>>, ApplicationError> {
    if properties.is_empty() {
        return Ok(HashMap::new());
    }

    let organization_id = &properties[0].organization_id;
    let property_ids: Vec<String> = properties
        .iter()
        .map(|property| property.id.clone())
        .collect();
    let links = property_owner::Entity::find()
        .filter(property_owner::Column::OrganizationId.eq(organization_id.clone()))
        .filter(property_owner::Column::PropertyId.is_in(property_ids))
        .order_by_asc(property_owner::Column::SortOrder)
        .order_by_asc(property_owner::Column::OwnerId)
        .all(database)
        .await
        .map_err(persistence_error)?;
    if links.is_empty() {
        return Ok(HashMap::new());
    }

    let owner_ids: Vec<String> = links.iter().map(|link| link.owner_id.clone()).collect();
    let owners = owner::Entity::find()
        .filter(owner::Column::OrganizationId.eq(organization_id.clone()))
        .filter(owner::Column::Id.is_in(owner_ids.clone()))
        .all(database)
        .await
        .map_err(persistence_error)?;
    let phones = owner_phone::Entity::find()
        .filter(owner_phone::Column::OrganizationId.eq(organization_id.clone()))
        .filter(owner_phone::Column::OwnerId.is_in(owner_ids))
        .order_by_asc(owner_phone::Column::CreatedAt)
        .all(database)
        .await
        .map_err(persistence_error)?;
    let owners_by_id: HashMap<String, owner::Model> = owners
        .into_iter()
        .map(|owner| (owner.id.clone(), owner))
        .collect();
    let mut phones_by_owner = HashMap::new();
    for phone in phones {
        phones_by_owner
            .entry(phone.owner_id.clone())
            .or_insert(phone);
    }

    let mut result: HashMap<String, Vec<crate::domain::PropertyOwnerInput>> = HashMap::new();
    for link in links {
        let owner = owners_by_id.get(&link.owner_id).ok_or_else(|| {
            tracing::error!(owner_id = %link.owner_id, "Property ownership refers to a missing owner");
            ApplicationError::Persistence
        })?;
        let phone = phones_by_owner.get(&link.owner_id).ok_or_else(|| {
            tracing::error!(owner_id = %link.owner_id, "Property owner has no contact phone");
            ApplicationError::Persistence
        })?;
        result
            .entry(link.property_id)
            .or_default()
            .push(crate::domain::PropertyOwnerInput {
                id: Some(owner.id.clone()),
                name: owner.name.clone(),
                phone_number: phone.phone_number.clone(),
                phone_label: phone.label.clone(),
                is_whatsapp: phone.is_whatsapp,
                ownership_share_percent: link.ownership_share_percent,
            });
    }
    Ok(result)
}

async fn sync_property_owners<C: ConnectionTrait>(
    database: &C,
    organization_id: &str,
    property_id: &str,
    owners: &[crate::domain::PropertyOwnerInput],
) -> Result<(), ApplicationError> {
    let owner_ids: Vec<String> = owners.iter().filter_map(|owner| owner.id.clone()).collect();
    let mut stale_links = property_owner::Entity::delete_many()
        .filter(property_owner::Column::OrganizationId.eq(organization_id.to_string()))
        .filter(property_owner::Column::PropertyId.eq(property_id.to_string()));
    if !owner_ids.is_empty() {
        stale_links = stale_links.filter(property_owner::Column::OwnerId.is_not_in(owner_ids));
    }
    stale_links
        .exec(database)
        .await
        .map_err(persistence_error)?;

    for (sort_order, input) in owners.iter().enumerate() {
        let sort_order = i32::try_from(sort_order).map_err(|_| {
            tracing::error!("Property owner ordering exceeds the supported range");
            ApplicationError::Persistence
        })?;
        let owner_id = if let Some(id) = &input.id {
            let existing = owner::Entity::find_by_id(id.clone())
                .filter(owner::Column::OrganizationId.eq(organization_id.to_string()))
                .one(database)
                .await
                .map_err(persistence_error)?
                .ok_or(ApplicationError::NotFound)?;
            let mut active = existing.into_active_model();
            active.name = Set(input.name.clone());
            active.updated_at = Set(Utc::now());
            active.update(database).await.map_err(persistence_error)?.id
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            owner::ActiveModel {
                id: Set(id.clone()),
                organization_id: Set(organization_id.to_string()),
                name: Set(input.name.clone()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            }
            .insert(database)
            .await
            .map_err(persistence_error)?
            .id
        };

        let existing_phone = owner_phone::Entity::find()
            .filter(owner_phone::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(owner_phone::Column::OwnerId.eq(owner_id.clone()))
            .order_by_asc(owner_phone::Column::CreatedAt)
            .one(database)
            .await
            .map_err(persistence_error)?;
        if let Some(existing_phone) = existing_phone {
            let mut active = existing_phone.into_active_model();
            active.phone_number = Set(input.phone_number.clone());
            active.label = Set(input.phone_label.clone());
            active.is_whatsapp = Set(input.is_whatsapp);
            active.update(database).await.map_err(persistence_error)?;
        } else {
            owner_phone::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                organization_id: Set(organization_id.to_string()),
                owner_id: Set(owner_id.clone()),
                phone_number: Set(input.phone_number.clone()),
                label: Set(input.phone_label.clone()),
                is_whatsapp: Set(input.is_whatsapp),
                created_at: Set(Utc::now()),
            }
            .insert(database)
            .await
            .map_err(persistence_error)?;
        }

        let link = property_owner::Entity::find()
            .filter(property_owner::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(property_owner::Column::PropertyId.eq(property_id.to_string()))
            .filter(property_owner::Column::OwnerId.eq(owner_id.clone()))
            .one(database)
            .await
            .map_err(persistence_error)?;
        if let Some(link) = link {
            let mut active = link.into_active_model();
            active.ownership_share_percent = Set(input.ownership_share_percent);
            active.sort_order = Set(sort_order);
            active.update(database).await.map_err(persistence_error)?;
        } else {
            property_owner::ActiveModel {
                property_id: Set(property_id.to_string()),
                owner_id: Set(owner_id),
                organization_id: Set(organization_id.to_string()),
                ownership_share_percent: Set(input.ownership_share_percent),
                sort_order: Set(sort_order),
                created_at: Set(Utc::now()),
            }
            .insert(database)
            .await
            .map_err(persistence_error)?;
        }
    }
    Ok(())
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
                    district TEXT,
                    neighborhood TEXT,
                    postal_code TEXT,
                    country_code TEXT NOT NULL,
                    lot_area_square_meters REAL,
                    building_area_square_meters REAL,
                    frontage_meters REAL,
                    depth_meters REAL,
                    latitude REAL,
                    longitude REAL,
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
                r#"CREATE TABLE owners (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE (id, organization_id)
                )"#,
            ))
            .await
            .expect("create owner fixture");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE owner_phones (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL,
                    phone_number TEXT NOT NULL,
                    label TEXT,
                    is_whatsapp BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TEXT NOT NULL,
                    UNIQUE (owner_id, phone_number)
                )"#,
            ))
            .await
            .expect("create owner phone fixture");
        database
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE property_owners (
                    property_id TEXT NOT NULL,
                    organization_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL,
                    ownership_share_percent REAL,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (property_id, owner_id)
                )"#,
            ))
            .await
            .expect("create property owner fixture");
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
            district: Some("Al Olaya".to_string()),
            neighborhood: Some("North".to_string()),
            postal_code: None,
            country_code: "SA".to_string(),
            lot_area_square_meters: Some(Decimal::new(120000, 2)),
            building_area_square_meters: Some(Decimal::new(85000, 2)),
            frontage_meters: None,
            depth_meters: None,
            latitude: None,
            longitude: None,
            owners: Vec::new(),
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

    #[tokio::test]
    async fn property_measurements_and_owner_contacts_round_trip_and_update() {
        let database = database().await;
        let organization_id = OrganizationId::new();
        let actor = ActorContext::new(UserId::new(), organization_id, AccessRole::Admin);
        let catalog = PropertyCatalog::new(SeaOrmPropertyRepository::new(&database));
        let mut property_input = input("APT-1", "Residence");
        property_input.owners = vec![
            crate::domain::PropertyOwnerInput {
                id: None,
                name: "  Property owner  ".to_string(),
                phone_number: " 0500000000 ".to_string(),
                phone_label: Some(" Main ".to_string()),
                is_whatsapp: true,
                ownership_share_percent: Some(Decimal::from(60)),
            },
            crate::domain::PropertyOwnerInput {
                id: None,
                name: "Second owner".to_string(),
                phone_number: "0500000001".to_string(),
                phone_label: None,
                is_whatsapp: false,
                ownership_share_percent: Some(Decimal::from(40)),
            },
        ];

        let created = catalog
            .create(actor, property_input)
            .await
            .expect("create property with owner");
        assert_eq!(
            created.input.lot_area_square_meters,
            Some(Decimal::new(120000, 2))
        );
        assert_eq!(created.input.district.as_deref(), Some("Al Olaya"));
        assert_eq!(created.input.owners.len(), 2);
        assert_eq!(created.input.owners[0].name, "Property owner");
        assert_eq!(created.input.owners[0].phone_number, "0500000000");
        assert!(created.input.owners[0].is_whatsapp);

        let owner_id = created.input.owners[0]
            .id
            .clone()
            .expect("stored owner identifier");
        let mut updated_input = created.input.clone();
        updated_input.owners[0].name = "Updated owner".to_string();
        updated_input.owners[0].ownership_share_percent = Some(Decimal::from(75));
        updated_input.owners.truncate(1);
        let updated = catalog
            .update(actor, created.id, updated_input)
            .await
            .expect("update owner details");

        assert_eq!(
            updated.input.owners[0].id.as_deref(),
            Some(owner_id.as_str())
        );
        assert_eq!(updated.input.owners[0].name, "Updated owner");
        assert_eq!(
            updated.input.owners[0].ownership_share_percent,
            Some(Decimal::from(75))
        );
        assert_eq!(updated.input.owners.len(), 1);
        let owner_count = database
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS count FROM owners",
            ))
            .await
            .expect("count preserved owners")
            .expect("owner count exists")
            .try_get_by::<i64, _>("count")
            .expect("read owner count");
        assert_eq!(owner_count, 2);
        assert_eq!(catalog.list(actor).await.expect("list").len(), 1);
    }
}
