# AqarDesk Desktop

A Tauri and SvelteKit desktop application for property, unit, tenant, and
payment management. The Rust desktop backend and persistence implementation
live in the `aqardesk-backend` modular-monolith crate.

## Architecture

- Desktop shell: Tauri
- Frontend: SvelteKit, TypeScript, and Tailwind CSS
- Backend: Rust modular monolith with domain, application, infrastructure,
  migration, and Tauri presentation boundaries
- Database: SeaORM and additive migrations; SQLite for local use and PostgreSQL
  for shared deployments

## Workspace

- `apps/desktop`: Tauri desktop application
- `apps/web`: SvelteKit frontend
- `backend`: domain, application, persistence, and migration modules
- `packages`: shared frontend packages

## Development

Install Bun and Rust, then install the JavaScript workspace dependencies:

```bash
bun install
```

Start the frontend or desktop shell:

```bash
bun run dev:web
bun run desktop:dev
```

The desktop setup flow supports local SQLite, direct PostgreSQL, and API
endpoint health checks. Connection settings are stored in the application
data directory in `connection.json`, separate from in-memory application
state. The file stores only the PostgreSQL credential reference; passwords and
saved sessions remain in the operating-system credential store. Current
platform commands support local and direct database authentication; API-mode
authentication and settings, audit, dashboard, and contract-template
workflows still need migration.

## Checks

```bash
bun run check:web
cargo check --workspace --manifest-path Cargo.toml
cargo test -p aqardesk-backend --manifest-path Cargo.toml
```
