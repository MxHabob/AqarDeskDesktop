# Database foundation

AqarDesk uses SeaORM with SeaORM migrations and supports SQLite and PostgreSQL.
The desktop setup flow lets the user choose and save the connection; the
application does not require `DATABASE_URL` or `.env` for database selection.
PostgreSQL passwords are stored in the operating system credential store.

## Rules

- Keep monetary amounts as integer minor units and exchange rates as
  `NUMERIC`; never use floating point for money.
- Keep schema changes in ordered, non-destructive migrations and verify them
  on SQLite and PostgreSQL. Treat data-preserving migrations as irreversible
  unless a safe rollback is explicitly implemented.
- Scope tenant-owned records and foreign keys by organization wherever
  practical.
- Preserve posted financial history through audit records and immutable
  payment allocations.
- Keep credentials out of the persisted connection configuration.

## Current schema

- Identity: organizations, users, sessions, and audit events.
- Property management: properties, units, tenants, leases, charges, payments,
  and allocations.
- Property records include district and neighborhood, land/building areas,
  frontage and depth, and validated coordinates. Owners are organization-scoped
  records with phone contacts and a property-owner join carrying optional
  ownership shares.
- Multi-currency: currency catalog, organization-enabled currencies, dated FX
  rates, and payment allocations retaining the applied rate and both currency
  amounts.
- Contracts: templates, immutable template versions, and generated lease
  document metadata.

The current schema is a foundation, not a complete accounting ledger or
document-generation workflow. FX rates are entered manually; storage and
rendering of contract files still require application services.

Property-type-specific detail tables (for example apartment, villa, warehouse,
and commercial features) are not part of this desktop schema yet. The owner
directory manages each owner's name and primary phone; additional phone
contacts are supported by the schema but do not yet have a dedicated editor.

## Application roles

The desktop application stores built-in and organization-defined roles in the
same organization-scoped role catalog, with one permission table keyed by role
ID. Users reference that catalog. Administrators have fixed full access;
custom roles can be assigned only business permissions for properties, owners,
units, tenants, and payments. Role permissions are enforced in backend
services, and a role assigned to a user cannot be deleted. Existing staff and
readonly permission settings are migrated into the unified catalog.
