# AqarDesk Desktop

A desktop application foundation for managing real estate, tenants, and rental
contracts. It is a separate project based on the architecture and toolchain of
MULX, with a Tauri desktop shell, a SvelteKit frontend, a Rust backend, and
PostgreSQL.

This repository is the starting foundation; the property-management workflows
and data model still need to be implemented.

## Architecture

- Desktop shell: Tauri
- Frontend: SvelteKit, TypeScript, and Tailwind CSS
- Backend: Rust with Axum, organized into domain, application, and infrastructure
  crates
- Database: PostgreSQL with SQLx migrations
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

Copy `.env.example` to `.env` and configure the local PostgreSQL connection.
Start the frontend and backend in separate terminals:

```bash
bun run dev:web
cargo run --manifest-path backend/api/Cargo.toml
```

To launch the desktop application:

```bash
bun run desktop:dev
```

The desktop shell loads the web frontend, and its production build uses the
static SvelteKit adapter.

## Checks

```bash
bun run check:web
cargo check --workspace --manifest-path Cargo.toml
```
