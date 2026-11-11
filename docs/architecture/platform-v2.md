# Platform v2 architecture

The replacement backend is implemented in the `aqardesk-platform` crate. It is
a modular monolith and does not depend on the previous `domain`, `app`, or
`infrastructure` crates.

## Boundaries

- `domain`: validated identifiers, money, identity and access roles, property,
  unit, tenant, and payment invariants.
- `application`: use cases and persistence ports. Authorization is enforced at
  the application boundary and receives a backend-created `ActorContext`.
- `infrastructure`: SeaORM repositories, password hashing, transactions,
  structured diagnostics, and audit writes.
- `presentation`: frontend-safe error codes and messages. Database details are
  logged, not returned to callers.
- `migration`: an additive, repeatable schema migration and schema verifier.

Repositories scope reads and writes by organization. Write operations and their
audit entries share a transaction. Archive operations retain the source row and
set the deletion timestamp instead of deleting business history.

## Persistence and transition

The initial migration creates missing tables and indexes but does not drop,
rename, or truncate existing tables. On databases already initialized by the
previous release, the business tables are reused in place; existing rows,
password hashes, IDs, sessions, and migration history remain in the database.
Migration application and schema verification are separate operations so
startup fails explicitly if required structures are unavailable.

New SQLite schemas declare unit measurements using `REAL`. Queries against
existing SQLite schemas cast measurements to `REAL` at the persistence boundary,
so SQLite numeric-affinity integer values remain readable without rewriting or
discarding the user's original values.

## Validation

`cargo test -p aqardesk-platform --all-features` covers domain validation,
authorization, migration repeatability, authentication, and transactional
repository behavior against SQLite, including migration of a legacy NUMERIC
measurement schema. The Tauri application now initializes the platform runtime
and exposes replacement connection, identity, property, unit, tenant, and
payment commands. The frontend and the remaining organization, dashboard,
contract, and lease workflows have not yet been switched to those commands.
