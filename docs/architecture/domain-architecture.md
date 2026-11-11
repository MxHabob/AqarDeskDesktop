# Domain architecture

The domain layer is intentionally free of HTTP, SQLx, and framework dependencies. It defines entities, value objects, invariants, and module responsibilities.

## Initial domain contracts

- `aqardesk-domain`: shared value objects and identifiers for customers, organizations, branches, users, and money
- `aqardesk-app`: application service orchestration and validation logic for core ERP concerns
- `aqardesk-infrastructure`: SQLx and external service implementation details

## Modules

The foundation explicitly names the following module families: core, finance, sales, purchasing, inventory, POS, CRM, supply chain, manufacturing, HR, assets, projects, documents, reporting, integrations, and automation.

The domain model intentionally avoids adding complex accounting schemas before the database and audit requirements are finalized.
