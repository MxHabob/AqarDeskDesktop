# ADR: Greenfield modular-monolith backend

## Status

Accepted; core replacement and currently wired administration workflows are
implemented. Legacy database import and lease/charge workflows remain open.

## Context

The previous backend had multiple crates with business operations crossing
presentation and persistence boundaries. Reorganizing those crates would
preserve the implementation being replaced and make the transition difficult
to verify. Product data must remain protected even though internal interfaces
may break.

## Decision

Create `aqardesk-backend` in the `backend` directory as a modular-monolith crate with
explicit domain, application, infrastructure, migration, and presentation
modules. Keep database table names and meanings where possible so existing
customer data can be reused without export-and-reimport. Migrations are
additive, and repositories never issue destructive schema or record changes.
Keep Tauri commands as presentation adapters and separate domain-owned
repositories and application contracts. Dashboard metrics must be backed by
persisted data; unsupported lease and charge metrics are omitted rather than
represented by success-shaped zeroes.

## Consequences

- New use cases and repositories are tested independently of legacy code.
- Additive migrations do not discard existing tables or rows.
- Existing production database layouts still require an explicit compatibility
  and import verification before claiming complete data migration.
- Missing lease, charge, and API-authentication workflows remain explicit
  product gaps rather than being approximated by dashboard placeholders.
