# Updates and releases

AqarDesk desktop releases are built and published by
`.github/workflows/release.yml` when a `v*` tag is pushed.

## Principles

- Release artifacts should be reproducible and checksum-verified.
- Database migrations must be validated before rollout.
- Pre-update backup and health verification are mandatory.
- Development, test, and production builds must be separated.

## Preparing and publishing a desktop release

1. Start from a clean working tree on `main`.
2. For the first release, prepare the version already configured in the app
   (for example, `bun run release:prepare -- 1.0.0`). For later releases, run
   `bun run release:prepare -- patch` (or `minor`, `major`, or an explicit
   higher version). Preview first with `bun run release:prepare -- --dry-run 1.0.0`
   for the first release, or `bun run release:prepare -- --dry-run patch` later.
3. Review and complete the generated `releases/vX.Y.Z.md`, then commit the
   version and release-note changes.
4. Push the commit and matching `vX.Y.Z` tag. The workflow validates the tag
   and version files, builds signed Windows/macOS updater bundles, verifies and
   merges updater metadata, and publishes the GitHub release.

The release workflow currently supports Windows x64 and Apple Silicon macOS
updater packages. The release tag must point to a commit reachable from `main`.

Before the first release, configure these repository settings:

- Variable `TAURI_UPDATER_PUBLIC_KEY`: the exact contents of the Tauri-generated
  `.pub` file for the updater signing key. Do not base64-encode it again. It is
  validated and injected into a temporary build config.
- Secret `TAURI_SIGNING_PRIVATE_KEY`: the matching private signing key.
- Secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password, or an empty value
  for an unencrypted key.

Keep the private key and its password out of the repository. Releases fail
early when the signing key is missing, the tag/version disagree, or release
notes still contain preparation placeholders.
