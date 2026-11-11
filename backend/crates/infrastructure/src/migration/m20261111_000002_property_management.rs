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
                r#"CREATE TABLE properties (
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
                )"#,
                "CREATE INDEX idx_properties_org_status ON properties (organization_id, status) WHERE deleted_at IS NULL",
                "CREATE INDEX idx_properties_city ON properties (organization_id, city) WHERE deleted_at IS NULL",
                r#"CREATE TABLE property_units (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    property_id TEXT NOT NULL,
                    unit_number TEXT NOT NULL CHECK (length(trim(unit_number)) > 0),
                    unit_type TEXT NOT NULL
                        CHECK (unit_type IN ('apartment', 'office', 'retail', 'warehouse', 'parking', 'other')),
                    floor_number INTEGER,
                    area_square_meters NUMERIC(10, 2)
                        CHECK (area_square_meters IS NULL OR area_square_meters > 0),
                    bedrooms INTEGER NOT NULL DEFAULT 0 CHECK (bedrooms >= 0),
                    bathrooms NUMERIC(4, 1) NOT NULL DEFAULT 0 CHECK (bathrooms >= 0),
                    market_rent_minor BIGINT CHECK (market_rent_minor IS NULL OR market_rent_minor >= 0),
                    currency_code VARCHAR(3),
                    status TEXT NOT NULL DEFAULT 'available'
                        CHECK (status IN ('available', 'maintenance', 'inactive')),
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    UNIQUE (property_id, unit_number),
                    CONSTRAINT ck_property_units_rent_currency CHECK (
                        (market_rent_minor IS NULL AND currency_code IS NULL) OR
                        (market_rent_minor IS NOT NULL AND currency_code IS NOT NULL
                            AND length(currency_code) = 3 AND currency_code = upper(currency_code))
                    ),
                    CONSTRAINT fk_property_units_property
                        FOREIGN KEY (property_id, organization_id) REFERENCES properties(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_property_units_currency
                        FOREIGN KEY (currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_property_units_org_status ON property_units (organization_id, status) WHERE deleted_at IS NULL",
                r#"CREATE TABLE tenants (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    full_name TEXT NOT NULL CHECK (length(trim(full_name)) > 0),
                    email TEXT CHECK (email IS NULL OR (email = lower(trim(email)) AND length(email) <= 254)),
                    phone TEXT,
                    mailing_address TEXT,
                    notes TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    CONSTRAINT ck_tenants_contact CHECK (
                        (email IS NOT NULL AND length(trim(email)) > 0) OR
                        (phone IS NOT NULL AND length(trim(phone)) > 0)
                    ),
                    CONSTRAINT fk_tenants_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_tenants_org_name ON tenants (organization_id, full_name) WHERE deleted_at IS NULL",
                "CREATE INDEX idx_tenants_email ON tenants (email) WHERE email IS NOT NULL AND deleted_at IS NULL",
                "CREATE INDEX idx_tenants_phone ON tenants (phone) WHERE phone IS NOT NULL AND deleted_at IS NULL",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "DROP TABLE tenants",
                "DROP TABLE property_units",
                "DROP TABLE properties",
            ],
        )
        .await
    }
}
