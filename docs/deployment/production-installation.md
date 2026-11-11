# Production installation

Production installation follows a dedicated customer server deployment model. The application should be installed on a hardened local server with controlled network access and separate storage for PostgreSQL and uploaded files.

## Required steps

1. Provision a dedicated server or VM for the tenant.
2. Provision PostgreSQL via Docker Compose or a managed local instance.
3. Configure the database through the desktop setup flow; the API process
   reads its saved connection file with `--connection-config`.
4. Run the backend API and worker.
5. Deploy the web application or desktop shell behind the reverse proxy.
6. Validate health checks and database connectivity.
7. Apply backup and restore policies before production use.

## Current status

The infrastructure foundation is implemented, but production-grade migration,
session persistence, and backup validation remain next-phase tasks.
