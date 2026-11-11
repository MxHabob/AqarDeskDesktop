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
                r#"CREATE TABLE IF NOT EXISTS currencies (
                    code VARCHAR(3) PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    name_ar TEXT,
                    symbol TEXT NOT NULL,
                    symbol_position TEXT NOT NULL DEFAULT 'suffix',
                    decimal_digits SMALLINT NOT NULL DEFAULT 2,
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CHECK (length(code) = 3 AND code = upper(code)),
                    CHECK (decimal_digits BETWEEN 0 AND 4)
                )"#,
                "INSERT INTO currencies (code, name, symbol, decimal_digits) VALUES ('SAR', 'Saudi Riyal', 'ر.س', 2) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, symbol, decimal_digits) VALUES ('USD', 'US Dollar', '$', 2) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, symbol, decimal_digits) VALUES ('EUR', 'Euro', '€', 2) ON CONFLICT (code) DO NOTHING",
                "CREATE TABLE IF NOT EXISTS platform_installation (id INTEGER PRIMARY KEY CHECK (id = 1), initialized BOOLEAN NOT NULL DEFAULT FALSE)",
                "INSERT INTO platform_installation (id, initialized) VALUES (1, FALSE) ON CONFLICT (id) DO NOTHING",
                r#"CREATE TABLE IF NOT EXISTS organizations (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    legal_name TEXT,
                    default_currency VARCHAR(3) NOT NULL DEFAULT 'SAR',
                    timezone TEXT NOT NULL DEFAULT 'Asia/Riyadh',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (default_currency) REFERENCES currencies(code) ON DELETE RESTRICT
                )"#,
                r#"CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    full_name TEXT NOT NULL,
                    email TEXT NOT NULL,
                    password_hash TEXT NOT NULL,
                    access_role TEXT NOT NULL DEFAULT 'staff',
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
                    locked_until TIMESTAMPTZ,
                    last_login_at TIMESTAMPTZ,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    CHECK (access_role IN ('admin', 'staff', 'readonly'))
                )"#,
                r#"CREATE TABLE IF NOT EXISTS organization_currencies (
                    organization_id TEXT NOT NULL,
                    currency_code VARCHAR(3) NOT NULL,
                    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (organization_id, currency_code),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
                    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,
                    FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT
                )"#,
                "INSERT INTO organization_currencies (organization_id, currency_code, is_enabled) SELECT id, default_currency, TRUE FROM organizations WHERE 1 = 1 ON CONFLICT (organization_id, currency_code) DO NOTHING",
                "CREATE UNIQUE INDEX IF NOT EXISTS uq_platform_users_email ON users (email)",
                "CREATE INDEX IF NOT EXISTS idx_platform_users_org_role ON users (organization_id, access_role, is_active)",
                r#"CREATE TABLE IF NOT EXISTS user_sessions (
                    id TEXT PRIMARY KEY NOT NULL,
                    user_id TEXT NOT NULL,
                    token_hash TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    expires_at TIMESTAMPTZ NOT NULL,
                    revoked_at TIMESTAMPTZ,
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_platform_sessions_user_expiry ON user_sessions (user_id, expires_at)",
                "CREATE INDEX IF NOT EXISTS idx_platform_sessions_expiry ON user_sessions (expires_at)",
                r#"CREATE TABLE IF NOT EXISTS properties (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    code TEXT NOT NULL,
                    name TEXT NOT NULL,
                    property_type TEXT NOT NULL,
                    description TEXT,
                    address_line_1 TEXT NOT NULL,
                    address_line_2 TEXT,
                    city TEXT NOT NULL,
                    region TEXT,
                    postal_code TEXT,
                    country_code VARCHAR(2) NOT NULL,
                    status TEXT NOT NULL DEFAULT 'active',
                    created_by TEXT,
                    updated_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    UNIQUE (organization_id, code),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    CHECK (length(trim(code)) > 0 AND code = upper(trim(code))),
                    CHECK (length(trim(name)) > 0),
                    CHECK (length(country_code) = 2 AND country_code = upper(country_code)),
                    CHECK (status IN ('active', 'inactive', 'archived'))
                )"#,
                r#"CREATE TABLE IF NOT EXISTS property_units (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    property_id TEXT NOT NULL,
                    unit_number TEXT NOT NULL,
                    unit_type TEXT NOT NULL,
                    floor_number INTEGER,
                    area_square_meters REAL CHECK (area_square_meters IS NULL OR area_square_meters > 0),
                    bedrooms INTEGER NOT NULL DEFAULT 0 CHECK (bedrooms >= 0),
                    bathrooms REAL NOT NULL DEFAULT 0 CHECK (bathrooms >= 0),
                    market_rent_minor BIGINT CHECK (market_rent_minor IS NULL OR market_rent_minor >= 0),
                    currency_code VARCHAR(3),
                    status TEXT NOT NULL DEFAULT 'available',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    UNIQUE (property_id, unit_number),
                    FOREIGN KEY (property_id, organization_id) REFERENCES properties(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,
                    CHECK ((market_rent_minor IS NULL) = (currency_code IS NULL)),
                    CHECK (status IN ('available', 'maintenance', 'inactive'))
                )"#,
                r#"CREATE TABLE IF NOT EXISTS tenants (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    full_name TEXT NOT NULL,
                    email TEXT,
                    phone TEXT,
                    mailing_address TEXT,
                    notes TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    CHECK ((email IS NOT NULL AND length(trim(email)) > 0) OR (phone IS NOT NULL AND length(trim(phone)) > 0))
                )"#,
                r#"CREATE TABLE IF NOT EXISTS payments (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    received_at TIMESTAMPTZ NOT NULL,
                    amount_minor BIGINT NOT NULL CHECK (amount_minor > 0),
                    currency_code VARCHAR(3) NOT NULL,
                    payment_method TEXT NOT NULL,
                    external_reference TEXT,
                    status TEXT NOT NULL DEFAULT 'posted',
                    notes TEXT,
                    recorded_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    voided_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id, currency_code, tenant_id),
                    FOREIGN KEY (tenant_id, organization_id) REFERENCES tenants(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (recorded_by, organization_id) REFERENCES users(id, organization_id) ON DELETE RESTRICT,
                    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,
                    CHECK (payment_method IN ('cash', 'bank_transfer', 'card', 'cheque', 'online', 'other')),
                    CHECK ((status = 'void') = (voided_at IS NOT NULL))
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_platform_properties_org_status ON properties (organization_id, status) WHERE deleted_at IS NULL",
                "CREATE INDEX IF NOT EXISTS idx_platform_units_org_property ON property_units (organization_id, property_id) WHERE deleted_at IS NULL",
                "CREATE INDEX IF NOT EXISTS idx_platform_tenants_org_name ON tenants (organization_id, full_name) WHERE deleted_at IS NULL",
                "CREATE INDEX IF NOT EXISTS idx_platform_payments_org_date ON payments (organization_id, received_at)",
                r#"CREATE TABLE IF NOT EXISTS audit_events (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    actor_user_id TEXT,
                    action TEXT NOT NULL,
                    entity_type TEXT NOT NULL,
                    entity_id TEXT NOT NULL,
                    details_json TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE RESTRICT,
                    FOREIGN KEY (actor_user_id) REFERENCES users(id) ON DELETE SET NULL
                )"#,
                "CREATE INDEX IF NOT EXISTS idx_platform_audit_org_created ON audit_events (organization_id, created_at)",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Platform schema migration is irreversible to protect business data.".to_string(),
        ))
    }
}
