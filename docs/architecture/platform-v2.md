# Platform v2 architecture

The replacement backend is the `backend` crate (`aqardesk-backend`). It is a
modular monolith and does not depend on the removed legacy backend crates.

## Boundaries

- `domain`: validated identifiers, money, identity and access roles, property,
  unit, tenant, and payment invariants.
- `application`: use cases and persistence ports organized by identity,
  organization settings, currencies, user profile, audit log, contract
  templates, dashboard, properties, units, tenants, and payments.
  Authorization is enforced at the application boundary and receives a
  backend-created `ActorContext`.
- `infrastructure`: SeaORM repositories, password hashing, transactions,
  structured diagnostics, audit writes, and organization-scoped dashboard
  queries.
- `presentation`: frontend-safe error codes and messages. Database details are
  logged, not returned to callers.
- `migration`: additive, repeatable migrations and schema verification. The
contract-template migration preserves the existing tables and adds templates,
version history, and indexes.

Repositories scope reads and writes by organization. Write operations and their
audit entries share a transaction. Archive operations retain the source row and
set the deletion timestamp instead of deleting business history.

## Persistence and transition

Migrations create missing tables and indexes but do not drop, rename, or
truncate existing tables. Existing rows are not deleted by these migrations.
An automated import/compatibility migration from every prior production schema
has not yet been verified; do not treat successful schema initialization as
proof that arbitrary legacy data has been imported.

The backend crate lives directly in `backend` (not in a second nested
`platform` project). Desktop connection options are persisted to
`<app-data>/connection.json`; only non-secret options and operating-system
credential references are stored there. Database passwords and saved sessions
are kept in the OS credential store, while active connections and users exist
only in runtime state.
Migration application and schema verification are separate operations so
startup fails explicitly if required structures are unavailable.

New SQLite schemas declare unit measurements using `REAL`. Queries against
existing SQLite schemas cast measurements to `REAL` at the persistence boundary,
so SQLite numeric-affinity integer values remain readable without rewriting or
discarding the user's original values.

## Validation

The dashboard aggregates organization-scoped active properties, non-archived
units and tenants, posted payments, and unit statuses. Monthly posted-payment
series use the organization's IANA time zone, exclude voided payments, include
empty months in the selected 3/6/12 month period, and group amounts by currency.
Lease, charge, and generated-document metrics are intentionally not returned:
those data domains are not implemented in the current persistence model, and
displaying placeholder zeroes would be misleading.

Organization settings, organization currencies, profile updates, paginated
audit history, versioned contract templates, and supported dashboard metrics
are wired from the frontend through registered Tauri commands to application
services and scoped SeaORM repositories. Write-side audit records are committed
in the same transaction as their corresponding change.

Validation includes Rust unit and SQLite integration tests for migration
repeatability, organization-scoped dashboard counts, local-month boundaries,
currency totals, template versioning, settings, profile updates, and audit
history. The contracts routes remain placeholders, API-mode authentication is
not implemented, and an automated importer for all legacy database layouts
remains outstanding.
