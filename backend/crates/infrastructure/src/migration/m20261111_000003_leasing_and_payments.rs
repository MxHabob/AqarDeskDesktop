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
                r#"CREATE TABLE leases (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    lease_number TEXT NOT NULL CHECK (length(trim(lease_number)) > 0),
                    unit_id TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    start_on DATE NOT NULL,
                    end_on DATE NOT NULL,
                    rent_amount_minor BIGINT NOT NULL CHECK (rent_amount_minor > 0),
                    deposit_amount_minor BIGINT NOT NULL DEFAULT 0 CHECK (deposit_amount_minor >= 0),
                    currency_code VARCHAR(3) NOT NULL
                        CHECK (length(currency_code) = 3 AND currency_code = upper(currency_code)),
                    billing_period TEXT NOT NULL DEFAULT 'monthly'
                        CHECK (billing_period IN ('weekly', 'monthly', 'quarterly', 'semi_annual', 'annual')),
                    due_day SMALLINT NOT NULL DEFAULT 1 CHECK (due_day BETWEEN 1 AND 28),
                    status TEXT NOT NULL DEFAULT 'draft'
                        CHECK (status IN ('draft', 'pending', 'active', 'expired', 'terminated', 'void')),
                    terms TEXT,
                    signed_at TIMESTAMPTZ,
                    terminated_at TIMESTAMPTZ,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    deleted_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id),
                    UNIQUE (id, organization_id, currency_code),
                    UNIQUE (id, organization_id, currency_code, tenant_id),
                    UNIQUE (organization_id, lease_number),
                    CONSTRAINT ck_leases_date_range CHECK (end_on > start_on),
                    CONSTRAINT ck_leases_signing CHECK (status NOT IN ('active', 'expired', 'terminated') OR signed_at IS NOT NULL),
                    CONSTRAINT fk_leases_unit
                        FOREIGN KEY (unit_id, organization_id) REFERENCES property_units(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_leases_tenant
                        FOREIGN KEY (tenant_id, organization_id) REFERENCES tenants(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_leases_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_leases_currency
                        FOREIGN KEY (currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_leases_org_status_dates ON leases (organization_id, status, start_on, end_on) WHERE deleted_at IS NULL",
                "CREATE INDEX idx_leases_unit_dates ON leases (unit_id, start_on, end_on) WHERE deleted_at IS NULL",
                "CREATE INDEX idx_leases_tenant_status ON leases (tenant_id, status) WHERE deleted_at IS NULL",
                "CREATE UNIQUE INDEX uq_leases_one_active_per_unit ON leases (unit_id) WHERE status = 'active' AND deleted_at IS NULL",
                r#"CREATE TABLE lease_charges (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    lease_id TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    charge_type TEXT NOT NULL
                        CHECK (charge_type IN ('rent', 'deposit', 'utility', 'fee', 'other')),
                    description TEXT,
                    period_start DATE NOT NULL,
                    period_end DATE NOT NULL,
                    due_on DATE NOT NULL,
                    amount_minor BIGINT NOT NULL CHECK (amount_minor > 0),
                    currency_code VARCHAR(3) NOT NULL
                        CHECK (length(currency_code) = 3 AND currency_code = upper(currency_code)),
                    status TEXT NOT NULL DEFAULT 'due'
                        CHECK (status IN ('scheduled', 'due', 'partially_paid', 'paid', 'void')),
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    voided_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id, currency_code, tenant_id),
                    UNIQUE (lease_id, charge_type, period_start, period_end),
                    CONSTRAINT ck_lease_charges_period CHECK (period_end >= period_start),
                    CONSTRAINT ck_lease_charges_void CHECK ((status = 'void') = (voided_at IS NOT NULL)),
                    CONSTRAINT fk_lease_charges_lease
                        FOREIGN KEY (lease_id, organization_id, currency_code, tenant_id)
                        REFERENCES leases(id, organization_id, currency_code, tenant_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_lease_charges_tenant
                        FOREIGN KEY (tenant_id, organization_id)
                        REFERENCES tenants(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_lease_charges_currency
                        FOREIGN KEY (currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_lease_charges_due ON lease_charges (organization_id, status, due_on)",
                r#"CREATE TABLE payments (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    received_at TIMESTAMPTZ NOT NULL,
                    amount_minor BIGINT NOT NULL CHECK (amount_minor > 0),
                    currency_code VARCHAR(3) NOT NULL
                        CHECK (length(currency_code) = 3 AND currency_code = upper(currency_code)),
                    payment_method TEXT NOT NULL
                        CHECK (payment_method IN ('cash', 'bank_transfer', 'card', 'cheque', 'online', 'other')),
                    external_reference TEXT,
                    status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'void')),
                    notes TEXT,
                    recorded_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    voided_at TIMESTAMPTZ,
                    UNIQUE (id, organization_id, currency_code, tenant_id),
                    CONSTRAINT ck_payments_void CHECK ((status = 'void') = (voided_at IS NOT NULL)),
                    CONSTRAINT fk_payments_tenant
                        FOREIGN KEY (tenant_id, organization_id) REFERENCES tenants(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_payments_recorded_by
                        FOREIGN KEY (recorded_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_payments_currency
                        FOREIGN KEY (currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_payments_tenant_date ON payments (tenant_id, received_at)",
                "CREATE INDEX idx_payments_org_date ON payments (organization_id, received_at)",
                "CREATE UNIQUE INDEX uq_payments_external_reference ON payments (organization_id, external_reference) WHERE external_reference IS NOT NULL",
                r#"CREATE TABLE payment_allocations (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    payment_id TEXT NOT NULL,
                    lease_charge_id TEXT NOT NULL,
                    tenant_id TEXT NOT NULL,
                    payment_currency_code VARCHAR(3) NOT NULL,
                    charge_currency_code VARCHAR(3) NOT NULL,
                    amount_in_payment_minor BIGINT NOT NULL CHECK (amount_in_payment_minor > 0),
                    amount_in_charge_minor BIGINT NOT NULL CHECK (amount_in_charge_minor > 0),
                    fx_rate NUMERIC(24, 12) NOT NULL CHECK (fx_rate > 0),
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (payment_id, lease_charge_id),
                    CONSTRAINT ck_payment_allocations_exchange CHECK (
                        (payment_currency_code = charge_currency_code
                            AND amount_in_payment_minor = amount_in_charge_minor
                            AND fx_rate = 1) OR
                        (payment_currency_code <> charge_currency_code AND fx_rate > 0)
                    ),
                    CONSTRAINT fk_payment_allocations_payment
                        FOREIGN KEY (payment_id, organization_id, payment_currency_code, tenant_id)
                        REFERENCES payments(id, organization_id, currency_code, tenant_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_payment_allocations_charge
                        FOREIGN KEY (lease_charge_id, organization_id, charge_currency_code, tenant_id)
                        REFERENCES lease_charges(id, organization_id, currency_code, tenant_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_payment_allocations_payment_currency
                        FOREIGN KEY (payment_currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_payment_allocations_charge_currency
                        FOREIGN KEY (charge_currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_payment_allocations_charge ON payment_allocations (lease_charge_id)",
                r#"CREATE TABLE audit_events (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    actor_user_id TEXT,
                    action TEXT NOT NULL CHECK (length(trim(action)) > 0),
                    entity_type TEXT NOT NULL CHECK (length(trim(entity_type)) > 0),
                    entity_id TEXT NOT NULL,
                    details_json TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT fk_audit_events_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_audit_events_actor
                        FOREIGN KEY (actor_user_id, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_audit_events_org_date ON audit_events (organization_id, created_at)",
                "CREATE INDEX idx_audit_events_entity ON audit_events (organization_id, entity_type, entity_id)",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "DROP TABLE audit_events",
                "DROP TABLE payment_allocations",
                "DROP TABLE payments",
                "DROP TABLE lease_charges",
                "DROP TABLE leases",
            ],
        )
        .await
    }
}
