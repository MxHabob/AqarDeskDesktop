use crate::{
    application::{ApplicationError, Page, PageRequest, UnitRepository},
    domain::{
        CurrencyCode, OrganizationId, PropertyId, PropertyUnit, PropertyUnitInput, UnitId,
        UnitStatus, UnitType, UserId,
    },
    infrastructure::{audit, models::unit},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DbBackend, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};
use std::str::FromStr;

#[derive(Debug, FromQueryResult)]
struct SqliteUnitRow {
    id: String,
    organization_id: String,
    property_id: String,
    unit_number: String,
    unit_type: String,
    floor_number: Option<i32>,
    area_square_meters: Option<f64>,
    bedrooms: i32,
    bathrooms: f64,
    market_rent_minor: Option<i64>,
    currency_code: Option<String>,
    status: String,
    created_at: sea_orm::prelude::DateTimeUtc,
    updated_at: sea_orm::prelude::DateTimeUtc,
}

struct UnitFields {
    id: String,
    organization_id: String,
    property_id: String,
    unit_number: String,
    unit_type: String,
    floor_number: Option<i32>,
    area_square_meters: Option<rust_decimal::Decimal>,
    bedrooms: i32,
    bathrooms: rust_decimal::Decimal,
    market_rent_minor: Option<i64>,
    currency_code: Option<String>,
    status: String,
    created_at: sea_orm::prelude::DateTimeUtc,
    updated_at: sea_orm::prelude::DateTimeUtc,
}

pub struct SeaOrmUnitRepository {
    database: DatabaseConnection,
}

impl SeaOrmUnitRepository {
    pub fn new(database: &DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
        }
    }
}

#[async_trait::async_trait]
impl UnitRepository for SeaOrmUnitRepository {
    async fn list(
        &self,
        organization_id: OrganizationId,
        property_id: Option<PropertyId>,
    ) -> Result<Vec<PropertyUnit>, ApplicationError> {
        if self.database.get_database_backend() == DbBackend::Sqlite {
            return self.list_sqlite(organization_id, property_id).await;
        }
        let mut query = unit::Entity::find()
            .filter(unit::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(unit::Column::DeletedAt.is_null())
            .order_by_asc(unit::Column::UnitNumber);
        if let Some(property_id) = property_id {
            query = query.filter(unit::Column::PropertyId.eq(property_id.to_string()));
        }
        let models = query.all(&self.database).await.map_err(persistence_error)?;
        models.into_iter().map(restore_unit).collect()
    }

    async fn list_page(
        &self,
        organization_id: OrganizationId,
        property_id: Option<PropertyId>,
        page: PageRequest,
    ) -> Result<Page<PropertyUnit>, ApplicationError> {
        let models = if self.database.get_database_backend() == DbBackend::Sqlite {
            self.list_sqlite_page(organization_id, property_id, page)
                .await?
        } else {
            let mut query = unit::Entity::find()
                .filter(unit::Column::OrganizationId.eq(organization_id.to_string()))
                .filter(unit::Column::DeletedAt.is_null())
                .order_by_asc(unit::Column::UnitNumber)
                .order_by_asc(unit::Column::Id);
            if let Some(property_id) = property_id {
                query = query.filter(unit::Column::PropertyId.eq(property_id.to_string()));
            }
            query
                .limit(page.fetch_limit())
                .offset(page.offset())
                .all(&self.database)
                .await
                .map_err(persistence_error)?
                .into_iter()
                .map(restore_unit)
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(Page::from_items(models, page.limit()))
    }

    async fn insert(
        &self,
        unit: PropertyUnit,
        actor_id: UserId,
    ) -> Result<PropertyUnit, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        unit::ActiveModel {
            id: Set(unit.id.to_string()),
            organization_id: Set(unit.organization_id.to_string()),
            property_id: Set(unit.input.property_id.to_string()),
            unit_number: Set(unit.input.unit_number.clone()),
            unit_type: Set(unit.input.unit_type.as_str().to_string()),
            floor_number: Set(unit.input.floor_number),
            area_square_meters: Set(unit.input.area_square_meters),
            bedrooms: Set(i32::try_from(unit.input.bedrooms).map_err(|_| {
                ApplicationError::Domain(crate::domain::DomainError::Validation(
                    "Unit bedroom count is invalid.",
                ))
            })?),
            bathrooms: Set(unit.input.bathrooms),
            market_rent_minor: Set(unit.input.market_rent_minor),
            currency_code: Set(unit
                .input
                .currency_code
                .as_ref()
                .map(|code| code.as_str().to_string())),
            status: Set(unit.input.status.as_str().to_string()),
            created_at: Set(unit.created_at),
            updated_at: Set(unit.updated_at),
            deleted_at: Set(None),
        }
        .insert(&transaction)
        .await
        .map_err(persistence_error)?;
        audit::record(
            &transaction,
            &unit.organization_id.to_string(),
            &actor_id.to_string(),
            "created",
            "property_unit",
            &unit.id.to_string(),
            serde_json::json!({"property_id": unit.input.property_id.to_string()}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        self.load_by_id(unit.id, unit.organization_id).await
    }

    async fn update(
        &self,
        unit: PropertyUnit,
        actor_id: UserId,
    ) -> Result<PropertyUnit, ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let active = unit::ActiveModel {
            property_id: Set(unit.input.property_id.to_string()),
            unit_number: Set(unit.input.unit_number.clone()),
            unit_type: Set(unit.input.unit_type.as_str().to_string()),
            floor_number: Set(unit.input.floor_number),
            area_square_meters: Set(unit.input.area_square_meters),
            bedrooms: Set(i32::try_from(unit.input.bedrooms).map_err(|_| {
                ApplicationError::Domain(crate::domain::DomainError::Validation(
                    "Unit bedroom count is invalid.",
                ))
            })?),
            bathrooms: Set(unit.input.bathrooms),
            market_rent_minor: Set(unit.input.market_rent_minor),
            currency_code: Set(unit
                .input
                .currency_code
                .as_ref()
                .map(|code| code.as_str().to_string())),
            status: Set(unit.input.status.as_str().to_string()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        let result = unit::Entity::update_many()
            .set(active)
            .filter(unit::Column::Id.eq(unit.id.to_string()))
            .filter(unit::Column::OrganizationId.eq(unit.organization_id.to_string()))
            .filter(unit::Column::DeletedAt.is_null())
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;
        if result.rows_affected == 0 {
            return Err(ApplicationError::NotFound);
        }
        audit::record(
            &transaction,
            &unit.organization_id.to_string(),
            &actor_id.to_string(),
            "updated",
            "property_unit",
            &unit.id.to_string(),
            serde_json::json!({"property_id": unit.input.property_id.to_string()}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)?;
        self.load_by_id(unit.id, unit.organization_id).await
    }

    async fn archive(
        &self,
        organization_id: OrganizationId,
        unit_id: UnitId,
        actor_id: UserId,
    ) -> Result<(), ApplicationError> {
        let transaction = self.database.begin().await.map_err(persistence_error)?;
        let now = Utc::now();
        let result = unit::Entity::update_many()
            .col_expr(
                unit::Column::DeletedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                unit::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(unit::Column::Id.eq(unit_id.to_string()))
            .filter(unit::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(unit::Column::DeletedAt.is_null())
            .exec(&transaction)
            .await
            .map_err(persistence_error)?;
        if result.rows_affected == 0 {
            return Err(ApplicationError::NotFound);
        }
        audit::record(
            &transaction,
            &organization_id.to_string(),
            &actor_id.to_string(),
            "archived",
            "property_unit",
            &unit_id.to_string(),
            serde_json::json!({}),
        )
        .await?;
        transaction.commit().await.map_err(persistence_error)
    }
}

impl SeaOrmUnitRepository {
    async fn list_sqlite(
        &self,
        organization_id: OrganizationId,
        property_id: Option<PropertyId>,
    ) -> Result<Vec<PropertyUnit>, ApplicationError> {
        let columns = "id, organization_id, property_id, unit_number, unit_type, floor_number, \
            CAST(area_square_meters AS REAL) AS area_square_meters, bedrooms, \
            CAST(bathrooms AS REAL) AS bathrooms, market_rent_minor, currency_code, status, \
            created_at, updated_at";
        let (sql, values) = if let Some(property_id) = property_id {
            (
                format!(
                    "SELECT {columns} FROM property_units WHERE organization_id = ? AND property_id = ? AND deleted_at IS NULL ORDER BY unit_number ASC"
                ),
                vec![
                    organization_id.to_string().into(),
                    property_id.to_string().into(),
                ],
            )
        } else {
            (
                format!(
                    "SELECT {columns} FROM property_units WHERE organization_id = ? AND deleted_at IS NULL ORDER BY unit_number ASC"
                ),
                vec![organization_id.to_string().into()],
            )
        };
        let rows = self
            .database
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(persistence_error)?;
        rows.into_iter()
            .map(|row| {
                let row = SqliteUnitRow::from_query_result(&row, "").map_err(persistence_error)?;
                restore_sqlite_unit(row)
            })
            .collect()
    }

    async fn list_sqlite_page(
        &self,
        organization_id: OrganizationId,
        property_id: Option<PropertyId>,
        page: PageRequest,
    ) -> Result<Vec<PropertyUnit>, ApplicationError> {
        let columns = "id, organization_id, property_id, unit_number, unit_type, floor_number, \
            CAST(area_square_meters AS REAL) AS area_square_meters, bedrooms, \
            CAST(bathrooms AS REAL) AS bathrooms, market_rent_minor, currency_code, status, \
            created_at, updated_at";
        let (sql, values) = if let Some(property_id) = property_id {
            (
                format!(
                    "SELECT {columns} FROM property_units WHERE organization_id = ? AND property_id = ? AND deleted_at IS NULL ORDER BY unit_number ASC, id ASC LIMIT ? OFFSET ?"
                ),
                vec![
                    organization_id.to_string().into(),
                    property_id.to_string().into(),
                    i64::try_from(page.fetch_limit()).map_err(|_| ApplicationError::Persistence)?.into(),
                    i64::try_from(page.offset()).map_err(|_| ApplicationError::Persistence)?.into(),
                ],
            )
        } else {
            (
                format!(
                    "SELECT {columns} FROM property_units WHERE organization_id = ? AND deleted_at IS NULL ORDER BY unit_number ASC, id ASC LIMIT ? OFFSET ?"
                ),
                vec![
                    organization_id.to_string().into(),
                    i64::try_from(page.fetch_limit()).map_err(|_| ApplicationError::Persistence)?.into(),
                    i64::try_from(page.offset()).map_err(|_| ApplicationError::Persistence)?.into(),
                ],
            )
        };
        let rows = self
            .database
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                values,
            ))
            .await
            .map_err(persistence_error)?;
        rows.into_iter()
            .map(|row| {
                let row = SqliteUnitRow::from_query_result(&row, "").map_err(persistence_error)?;
                restore_sqlite_unit(row)
            })
            .collect()
    }

    async fn load_by_id(
        &self,
        unit_id: UnitId,
        organization_id: OrganizationId,
    ) -> Result<PropertyUnit, ApplicationError> {
        if self.database.get_database_backend() == DbBackend::Sqlite {
            let sql = "SELECT id, organization_id, property_id, unit_number, unit_type, \
                floor_number, CAST(area_square_meters AS REAL) AS area_square_meters, bedrooms, \
                CAST(bathrooms AS REAL) AS bathrooms, market_rent_minor, currency_code, status, \
                created_at, updated_at FROM property_units \
                WHERE id = ? AND organization_id = ? AND deleted_at IS NULL";
            let row = self
                .database
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    sql,
                    [
                        unit_id.to_string().into(),
                        organization_id.to_string().into(),
                    ],
                ))
                .await
                .map_err(persistence_error)?
                .ok_or(ApplicationError::NotFound)?;
            return restore_sqlite_unit(
                SqliteUnitRow::from_query_result(&row, "").map_err(persistence_error)?,
            );
        }
        let model = unit::Entity::find_by_id(unit_id.to_string())
            .filter(unit::Column::OrganizationId.eq(organization_id.to_string()))
            .filter(unit::Column::DeletedAt.is_null())
            .one(&self.database)
            .await
            .map_err(persistence_error)?
            .ok_or(ApplicationError::NotFound)?;
        restore_unit(model)
    }
}

fn restore_sqlite_unit(model: SqliteUnitRow) -> Result<PropertyUnit, ApplicationError> {
    let area_square_meters = model
        .area_square_meters
        .map(decimal_from_sqlite_float)
        .transpose()?;
    let bathrooms = decimal_from_sqlite_float(model.bathrooms)?;
    restore_unit_fields(UnitFields {
        id: model.id,
        organization_id: model.organization_id,
        property_id: model.property_id,
        unit_number: model.unit_number,
        unit_type: model.unit_type,
        floor_number: model.floor_number,
        area_square_meters,
        bedrooms: model.bedrooms,
        bathrooms,
        market_rent_minor: model.market_rent_minor,
        currency_code: model.currency_code,
        status: model.status,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

fn restore_unit(model: unit::Model) -> Result<PropertyUnit, ApplicationError> {
    restore_unit_fields(UnitFields {
        id: model.id,
        organization_id: model.organization_id,
        property_id: model.property_id,
        unit_number: model.unit_number,
        unit_type: model.unit_type,
        floor_number: model.floor_number,
        area_square_meters: model.area_square_meters,
        bedrooms: model.bedrooms,
        bathrooms: model.bathrooms,
        market_rent_minor: model.market_rent_minor,
        currency_code: model.currency_code,
        status: model.status,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

fn restore_unit_fields(fields: UnitFields) -> Result<PropertyUnit, ApplicationError> {
    let id = UnitId::parse(&fields.id).map_err(|error| {
        tracing::error!(%error, "Persisted unit identifier is invalid");
        ApplicationError::Persistence
    })?;
    let organization_id = OrganizationId::parse(&fields.organization_id).map_err(|error| {
        tracing::error!(%error, "Persisted organization identifier is invalid");
        ApplicationError::Persistence
    })?;
    let property_id = PropertyId::parse(&fields.property_id).map_err(|error| {
        tracing::error!(%error, "Persisted property identifier is invalid");
        ApplicationError::Persistence
    })?;
    let bedrooms = u32::try_from(fields.bedrooms).map_err(|error| {
        tracing::error!(%error, "Persisted unit bedroom count is invalid");
        ApplicationError::Persistence
    })?;
    let currency_code = fields
        .currency_code
        .map(CurrencyCode::try_from)
        .transpose()?;
    let input = PropertyUnitInput {
        property_id,
        unit_number: fields.unit_number,
        unit_type: UnitType::parse(&fields.unit_type)?,
        floor_number: fields.floor_number,
        area_square_meters: fields.area_square_meters,
        bedrooms,
        bathrooms: fields.bathrooms,
        market_rent_minor: fields.market_rent_minor,
        currency_code,
        status: UnitStatus::parse(&fields.status)?,
    };
    PropertyUnit::restore(
        id,
        organization_id,
        input,
        fields.created_at,
        fields.updated_at,
    )
    .map_err(Into::into)
}

fn decimal_from_sqlite_float(value: f64) -> Result<rust_decimal::Decimal, ApplicationError> {
    rust_decimal::Decimal::from_str(&value.to_string()).map_err(|error| {
        tracing::error!(%error, "Persisted SQLite unit measurement is invalid");
        ApplicationError::Persistence
    })
}

fn persistence_error(error: sea_orm::DbErr) -> ApplicationError {
    tracing::error!(%error, "Unit persistence operation failed");
    ApplicationError::Persistence
}
