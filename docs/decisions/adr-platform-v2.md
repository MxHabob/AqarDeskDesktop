# ADR: Greenfield modular-monolith backend

## Status

Accepted; implementation is in progress.

## Context

The previous backend had multiple crates with business operations crossing
presentation and persistence boundaries. Reorganizing those crates would
preserve the implementation being replaced and make the transition difficult
to verify.

## Decision

Create `aqardesk-platform` as an independent modular-monolith crate with
explicit domain, application, infrastructure, migration, and presentation
modules. Keep database table names and meanings where possible so existing
customer data can be reused without export-and-reimport. Migrations are
additive, and repositories never issue destructive schema or record changes.

## Consequences

- New use cases and repositories are tested independently from legacy code.
- Existing persisted data is retained in place; no database reset is required.
- A domain is switched to the new Tauri adapter only after its contract and
  migration path are verified.
- Legacy crates remain temporarily while their application workflows are
  migrated. Removing them before all routes have replacements would silently
  disable product capabilities.
