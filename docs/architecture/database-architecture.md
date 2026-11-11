# Database architecture

AqarDesk uses SeaORM and SeaORM migrations over SQLite and PostgreSQL. The
desktop setup flow selects and stores the database connection without relying
on `DATABASE_URL` or `.env`.

## Rules implemented at the foundation level

- Store monetary amounts as integer minor units and FX rates as database
  `NUMERIC`, never floating point.
- Keep migrations ordered, reversible, and covered by SQLite tests.
- Scope organization-owned relationships with composite foreign keys where
  applicable.
- Store PostgreSQL passwords in the operating system credential store.
- Separate startup health checks from runtime availability.

## Implemented foundation

The schema includes organizations, users and sessions, properties and units,
tenants, leases and charges, payments and allocations, audit events, a
multi-currency catalog and exchange rates, and versioned contract templates.

## Notes

This is not yet a production accounting ledger. Exchange-rate management UI,
automated rate providers, and contract document rendering/storage services
remain future work.
