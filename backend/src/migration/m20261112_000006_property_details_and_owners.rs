use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "ALTER TABLE properties ADD COLUMN district TEXT",
                "ALTER TABLE properties ADD COLUMN neighborhood TEXT",
                "ALTER TABLE properties ADD COLUMN lot_area_square_meters REAL CHECK (lot_area_square_meters IS NULL OR lot_area_square_meters >= 0)",
                "ALTER TABLE properties ADD COLUMN building_area_square_meters REAL CHECK (building_area_square_meters IS NULL OR building_area_square_meters >= 0)",
                "ALTER TABLE properties ADD COLUMN frontage_meters REAL CHECK (frontage_meters IS NULL OR frontage_meters >= 0)",
                "ALTER TABLE properties ADD COLUMN depth_meters REAL CHECK (depth_meters IS NULL OR depth_meters >= 0)",
                "ALTER TABLE properties ADD COLUMN latitude REAL CHECK (latitude IS NULL OR latitude BETWEEN -90 AND 90)",
                "ALTER TABLE properties ADD COLUMN longitude REAL CHECK (longitude IS NULL OR longitude BETWEEN -180 AND 180)",
                r#"CREATE TABLE IF NOT EXISTS owners (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    CHECK (length(trim(name)) > 0)
                )"#,
                r#"CREATE TABLE IF NOT EXISTS owner_phones (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL,
                    phone_number TEXT NOT NULL,
                    label TEXT,
                    is_whatsapp BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (owner_id, phone_number),
                    FOREIGN KEY (owner_id, organization_id) REFERENCES owners(id, organization_id) ON DELETE CASCADE,
                    CHECK (length(trim(phone_number)) > 0)
                )"#,
                r#"CREATE TABLE IF NOT EXISTS property_owners (
                    property_id TEXT NOT NULL,
                    organization_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL,
                    ownership_share_percent REAL,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (property_id, owner_id),
                    FOREIGN KEY (property_id, organization_id) REFERENCES properties(id, organization_id) ON DELETE CASCADE,
                    FOREIGN KEY (owner_id, organization_id) REFERENCES owners(id, organization_id) ON DELETE RESTRICT,
                    CHECK (ownership_share_percent IS NULL OR ownership_share_percent BETWEEN 0 AND 100),
                    CHECK (sort_order >= 0)
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_platform_owners_org_name ON owners (organization_id, name)",
                "CREATE INDEX IF NOT EXISTS idx_platform_owner_phones_owner ON owner_phones (owner_id)",
                "CREATE INDEX IF NOT EXISTS idx_platform_property_owners_owner ON property_owners (organization_id, owner_id)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Property ownership migration is irreversible to protect business data.".to_string(),
        ))
    }
}
