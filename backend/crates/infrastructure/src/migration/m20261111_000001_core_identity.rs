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
                r#"CREATE TABLE currencies (
                    code VARCHAR(3) PRIMARY KEY NOT NULL
                        CHECK (length(code) = 3 AND code = upper(code)),
                    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
                    name_ar TEXT,
                    symbol TEXT NOT NULL,
                    symbol_position TEXT NOT NULL DEFAULT 'prefix'
                        CHECK (symbol_position IN ('prefix', 'suffix')),
                    decimal_digits SMALLINT NOT NULL DEFAULT 2
                        CHECK (decimal_digits BETWEEN 0 AND 4),
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
                )"#,
                r#"CREATE TABLE organizations (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
                    legal_name TEXT,
                    default_currency VARCHAR(3) NOT NULL DEFAULT 'SAR'
                        CHECK (length(default_currency) = 3 AND default_currency = upper(default_currency)),
                    timezone TEXT NOT NULL DEFAULT 'Asia/Riyadh'
                        CHECK (length(trim(timezone)) > 0),
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id),
                    CONSTRAINT fk_organizations_default_currency
                        FOREIGN KEY (default_currency) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                r#"CREATE TABLE users (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    full_name TEXT NOT NULL CHECK (length(trim(full_name)) > 0),
                    email TEXT NOT NULL CHECK (email = lower(trim(email)) AND length(email) <= 254),
                    password_hash TEXT NOT NULL CHECK (length(password_hash) >= 32),
                    access_role TEXT NOT NULL DEFAULT 'staff'
                        CHECK (access_role IN ('admin', 'staff', 'readonly')),
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    failed_login_attempts INTEGER NOT NULL DEFAULT 0
                        CHECK (failed_login_attempts >= 0),
                    locked_until TIMESTAMPTZ,
                    last_login_at TIMESTAMPTZ,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id),
                    CONSTRAINT fk_users_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE UNIQUE INDEX uq_users_email ON users (email)",
                "CREATE INDEX idx_users_organization_role ON users (organization_id, access_role, is_active)",
                r#"CREATE TABLE user_sessions (
                    id TEXT PRIMARY KEY NOT NULL,
                    user_id TEXT NOT NULL,
                    token_hash TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    expires_at TIMESTAMPTZ NOT NULL,
                    revoked_at TIMESTAMPTZ,
                    CONSTRAINT ck_user_sessions_expiry CHECK (expires_at > created_at),
                    CONSTRAINT fk_user_sessions_user
                        FOREIGN KEY (user_id) REFERENCES users(id)
                        ON UPDATE CASCADE ON DELETE CASCADE
                )"#,
                "CREATE INDEX idx_user_sessions_user_expiry ON user_sessions (user_id, expires_at)",
                "CREATE INDEX idx_user_sessions_expiry ON user_sessions (expires_at)",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "DROP TABLE user_sessions",
                "DROP TABLE users",
                "DROP TABLE organizations",
            ],
        )
        .await
    }
}
