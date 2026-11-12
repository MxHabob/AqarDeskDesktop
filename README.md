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

## Private repository updater hosting

The desktop updater can use a private source repository while downloading
publicly accessible, signed update files from Cloudflare R2:

1. Create an R2 bucket and attach an HTTPS custom domain to it. Make the bucket
   publicly readable through that domain; do not expose the S3 API credentials.
   The updater domain must serve the application's path prefix, such as
   `/aqardesk/latest.json`.
2. Create an R2 API token with Object Read & Write permissions limited to this
   bucket. Use its S3 Access Key ID and Secret Access Key as GitHub Actions
   secrets; never put them in the repository or desktop application.
3. In the repository's **Settings → Secrets and variables → Actions**, add
   these variables:
   - `TAURI_UPDATER_PUBLIC_URL`: the HTTPS origin of the R2 custom domain, for
     example `https://updates.example.com` (no path or trailing filename).
   - Use a separate object path prefix for every product sharing the bucket.
     This release workflow uses `aqardesk` and stores its manifest at
     `/aqardesk/latest.json`; keep Orbis files under a different prefix such as
     `/orbis/`.
   - `CLOUDFLARE_R2_ACCOUNT_ID`: the Cloudflare account ID.
   - `CLOUDFLARE_R2_BUCKET`: the bucket name.
   - `TAURI_UPDATER_PUBLIC_KEY`: the Tauri updater public key, as for existing
     signed releases.
4. Add these Actions secrets:
   - `CLOUDFLARE_R2_ACCESS_KEY_ID`
   - `CLOUDFLARE_R2_SECRET_ACCESS_KEY`
   - Keep the existing `TAURI_SIGNING_PRIVATE_KEY` and, if configured,
     `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
5. Ensure the custom domain or its cache rules do not cache
   `/aqardesk/latest.json`.
   Versioned bundles are immutable and can be cached. No CORS configuration is
   required for the native desktop updater.

The release workflow uploads signed platform bundles under
`/aqardesk/releases/<tag>/`, then publishes `/aqardesk/latest.json` last.
Production builds are configured to check that manifest on the R2 domain. The
GitHub Release can remain private; the updater files must be publicly
downloadable for clients that are not authenticated to GitHub. After each
successful upload, the workflow keeps the three newest AqarDesk versions in
R2 and removes older AqarDesk release objects; `latest.json` and other
products' prefixes are not pruned.
