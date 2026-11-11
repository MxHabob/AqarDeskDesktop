# Customer-premises deployment

AqarDesk is designed for dedicated single-customer installation on a local server environment. The deployment model intentionally keeps runtime, database, storage, and backup configuration local to the customer environment.

## Current implementation status

- Docker Compose service for local PostgreSQL provisioning
- Desktop-managed connection settings and operating-system credential storage
- Local reverse proxy foundation using Caddy
- Persistent volume pattern for PostgreSQL data

## Constraints

- No shared control plane or centralized licensing administration is implemented.
- External licensing checks are isolated behind an adapter and must not block ordinary local ERP workflows.
- The API process requires the connection configuration created by the desktop
  app (`--connection-config`); PostgreSQL credentials must be available to the
  operating-system credential store used by that process.
