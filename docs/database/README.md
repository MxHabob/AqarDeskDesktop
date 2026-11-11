# Database foundation

AqarDesk uses SeaORM with SeaORM migrations and supports SQLite and PostgreSQL.
The desktop setup flow lets the user choose and save the connection; the
application does not require `DATABASE_URL` or `.env` for database selection.
PostgreSQL passwords are stored in the operating system credential store.

## Rules

- Keep monetary amounts as integer minor units and exchange rates as
  `NUMERIC`; never use floating point for money.
- Keep schema changes in ordered, reversible migrations and verify them on
  SQLite and PostgreSQL.
- Scope tenant-owned records and foreign keys by organization wherever
  practical.
- Preserve posted financial history through audit records and immutable
  payment allocations.
- Keep credentials out of the persisted connection configuration.

## Current schema

- Identity: organizations, users, sessions, and audit events.
- Property management: properties, units, tenants, leases, charges, payments,
  and allocations.
- Multi-currency: currency catalog, organization-enabled currencies, dated FX
  rates, and payment allocations retaining the applied rate and both currency
  amounts.
- Contracts: templates, immutable template versions, and generated lease
  document metadata.

The current schema is a foundation, not a complete accounting ledger or
document-generation workflow. FX rates are entered manually; storage and
rendering of contract files still require application services.
