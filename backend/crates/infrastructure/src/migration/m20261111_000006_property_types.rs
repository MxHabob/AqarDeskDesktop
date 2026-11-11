use super::execute_statements;
use sea_orm::{DbBackend, DbErr};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const PROPERTY_TYPES: &str = "'residential', 'commercial', 'mixed_use', 'land', 'other', 'building', 'house_villa', 'investment', 'warehouse', 'yard', 'apartment'";

const CREATE_PROPERTIES_WITH_NEW_TYPES: &str = r#"CREATE TABLE properties_replacement (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0 AND code = upper(trim(code))),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    property_type TEXT NOT NULL
        CHECK (property_type IN ('residential', 'commercial', 'mixed_use', 'land', 'other', 'building', 'house_villa', 'investment', 'warehouse', 'yard', 'apartment')),
    description TEXT,
    address_line_1 TEXT NOT NULL CHECK (length(trim(address_line_1)) > 0),
    address_line_2 TEXT,
    city TEXT NOT NULL CHECK (length(trim(city)) > 0),
    region TEXT,
    postal_code TEXT,
    country_code VARCHAR(2) NOT NULL
        CHECK (length(country_code) = 2 AND country_code = upper(country_code)),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    created_by TEXT,
    updated_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, organization_id),
    UNIQUE (organization_id, code),
    CONSTRAINT fk_properties_organization
        FOREIGN KEY (organization_id) REFERENCES organizations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_properties_created_by
        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_properties_updated_by
        FOREIGN KEY (updated_by, organization_id) REFERENCES users(id, organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT
)"#;

const CREATE_PROPERTIES_WITH_LEGACY_TYPES: &str = r#"CREATE TABLE properties_replacement (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0 AND code = upper(trim(code))),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    property_type TEXT NOT NULL
        CHECK (property_type IN ('residential', 'commercial', 'mixed_use', 'land', 'other')),
    description TEXT,
    address_line_1 TEXT NOT NULL CHECK (length(trim(address_line_1)) > 0),
    address_line_2 TEXT,
    city TEXT NOT NULL CHECK (length(trim(city)) > 0),
    region TEXT,
    postal_code TEXT,
    country_code VARCHAR(2) NOT NULL
        CHECK (length(country_code) = 2 AND country_code = upper(country_code)),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    created_by TEXT,
    updated_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    UNIQUE (id, organization_id),
    UNIQUE (organization_id, code),
    CONSTRAINT fk_properties_organization
        FOREIGN KEY (organization_id) REFERENCES organizations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_properties_created_by
        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_properties_updated_by
        FOREIGN KEY (updated_by, organization_id) REFERENCES users(id, organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT
)"#;

const COPY_PROPERTIES_SQLITE: &str = r#"INSERT INTO properties_replacement (
    id, organization_id, code, name, property_type, description,
    address_line_1, address_line_2, city, region, postal_code, country_code,
    status, created_by, updated_by, created_at, updated_at, deleted_at
)
SELECT
    id, organization_id, code, name,
    CASE property_type
        WHEN 'building' THEN 'residential'
        WHEN 'house_villa' THEN 'residential'
        WHEN 'investment' THEN 'mixed_use'
        WHEN 'warehouse' THEN 'commercial'
        WHEN 'yard' THEN 'land'
        WHEN 'apartment' THEN 'residential'
        ELSE property_type
    END,
    description, address_line_1, address_line_2, city, region, postal_code,
    country_code, status, created_by, updated_by, created_at, updated_at, deleted_at
FROM properties"#;

const COPY_PROPERTIES: &str = r#"INSERT INTO properties_replacement (
    id, organization_id, code, name, property_type, description,
    address_line_1, address_line_2, city, region, postal_code, country_code,
    status, created_by, updated_by, created_at, updated_at, deleted_at
)
SELECT
    id, organization_id, code, name, property_type, description,
    address_line_1, address_line_2, city, region, postal_code, country_code,
    status, created_by, updated_by, created_at, updated_at, deleted_at
FROM properties"#;

const RESTORE_PROPERTY_INDEXES: [&str; 2] = [
    "CREATE INDEX idx_properties_org_status ON properties (organization_id, status) WHERE deleted_at IS NULL",
    "CREATE INDEX idx_properties_city ON properties (organization_id, city) WHERE deleted_at IS NULL",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DbBackend::Sqlite {
            return rebuild_sqlite_properties(
                manager,
                CREATE_PROPERTIES_WITH_NEW_TYPES,
                COPY_PROPERTIES,
            )
            .await;
        }

        execute_statements(
            manager,
            &[
                "ALTER TABLE properties DROP CONSTRAINT properties_property_type_check",
                &format!(
                    "ALTER TABLE properties ADD CONSTRAINT ck_properties_property_type CHECK (property_type IN ({PROPERTY_TYPES}))"
                ),
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DbBackend::Sqlite {
            return rebuild_sqlite_properties(
                manager,
                CREATE_PROPERTIES_WITH_LEGACY_TYPES,
                COPY_PROPERTIES_SQLITE,
            )
            .await;
        }

        execute_statements(
            manager,
            &[
                "UPDATE properties SET property_type = CASE property_type \
                    WHEN 'building' THEN 'residential' \
                    WHEN 'house_villa' THEN 'residential' \
                    WHEN 'investment' THEN 'mixed_use' \
                    WHEN 'warehouse' THEN 'commercial' \
                    WHEN 'yard' THEN 'land' \
                    WHEN 'apartment' THEN 'residential' \
                    ELSE property_type END",
                "ALTER TABLE properties DROP CONSTRAINT ck_properties_property_type",
                "ALTER TABLE properties ADD CONSTRAINT properties_property_type_check CHECK (property_type IN ('residential', 'commercial', 'mixed_use', 'land', 'other'))",
            ],
        )
        .await
    }
}

async fn rebuild_sqlite_properties(
    manager: &SchemaManager<'_>,
    create_table: &str,
    copy_rows: &str,
) -> Result<(), DbErr> {
    let connection = manager.get_connection();
    let has_properties = manager.has_table("properties").await?;
    let has_replacement = manager.has_table("properties_replacement").await?;

    if !has_properties && !has_replacement {
        return Err(DbErr::Custom(
            "Cannot recover property type migration: both properties tables are missing."
                .to_string(),
        ));
    }

    connection
        .execute_unprepared("PRAGMA foreign_keys = OFF")
        .await?;

    let migration_result = if has_properties {
        let reset_incomplete_replacement = if has_replacement {
            connection
                .execute_unprepared("DROP TABLE properties_replacement")
                .await
                .map(|_| ())
        } else {
            Ok(())
        };

        match reset_incomplete_replacement {
            Ok(()) => {
                execute_statements(
                    manager,
                    &[
                        create_table,
                        copy_rows,
                        "DROP TABLE properties",
                        "ALTER TABLE properties_replacement RENAME TO properties",
                        RESTORE_PROPERTY_INDEXES[0],
                        RESTORE_PROPERTY_INDEXES[1],
                    ],
                )
                .await
            }
            Err(error) => Err(error),
        }
    } else {
        execute_statements(
            manager,
            &[
                "ALTER TABLE properties_replacement RENAME TO properties",
                RESTORE_PROPERTY_INDEXES[0],
                RESTORE_PROPERTY_INDEXES[1],
            ],
        )
        .await
    };
    let foreign_keys_result = connection
        .execute_unprepared("PRAGMA foreign_keys = ON")
        .await;

    match (migration_result, foreign_keys_result) {
        (Ok(()), Ok(_)) => Ok(()),
        (Err(migration_error), Ok(_)) => Err(migration_error),
        (Ok(()), Err(foreign_keys_error)) => Err(foreign_keys_error),
        (Err(migration_error), Err(foreign_keys_error)) => Err(DbErr::Custom(format!(
            "{migration_error}; additionally failed to re-enable SQLite foreign keys: {foreign_keys_error}"
        ))),
    }
}
