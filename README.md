# AqarDesk Desktop

A desktop application foundation for managing real estate, tenants, and rental
contracts. It is a separate project based on the architecture and toolchain of
MULX, with a Tauri desktop shell, a SvelteKit frontend, a Rust backend, and
SeaORM persistence for SQLite and PostgreSQL.

The current foundation includes organization setup, user authentication and
administration, versioned database migrations, property and lease data models,
multi-currency accounting primitives, and versioned contract templates.
Property-management screens and workflows are still under development.

## Architecture

- Desktop shell: Tauri
- Frontend: SvelteKit, TypeScript, and Tailwind CSS
- Backend: Rust with Axum, organized into domain, application, and infrastructure
  crates
- Database: SeaORM and SeaORM migrations; SQLite for local use and PostgreSQL
  for shared deployments
- Local development services: Docker Compose

## Workspace

- `apps/desktop`: Tauri desktop application
- `apps/web`: SvelteKit frontend
- `backend/api`: HTTP API
- `backend/worker`: background jobs
- `backend/crates/domain`: domain types and rules
- `backend/crates/app`: application services
- `backend/crates/infrastructure`: database and external-service adapters
- `packages`: shared frontend packages

## Development

Install Bun and Rust, then install the JavaScript workspace dependencies:

```bash
bun install
```

Install Bun and Rust, then install the JavaScript workspace dependencies:

```bash
bun install
```

Start the frontend or desktop shell:

```bash
bun run dev:web
bun run desktop:dev
```

The desktop setup flow lets the user choose a local SQLite database,
PostgreSQL connection, or API server. Connection settings are stored in the
application data directory; PostgreSQL passwords are kept in the operating
system credential store, not in `.env`.

The API server reads the connection configuration created by the desktop app:

```powershell
cargo run --manifest-path backend/api/Cargo.toml -- --connection-config "PATH_TO_CONNECTION_CONFIG"
```

For local PostgreSQL development, Docker Compose provisions only the database.
Set its `POSTGRES_PASSWORD` in the shell before starting Compose; then select
that PostgreSQL connection in the desktop setup flow.

## Checks

```bash
bun run check:web
cargo check --workspace --manifest-path Cargo.toml
cargo test -p aqardesk-infrastructure --manifest-path Cargo.toml
```
