# AqarDesk licensing

AqarDesk verifies Mulverse-signed Ed25519 entitlements in the desktop backend. The UI gate is supplementary; Tauri commands enforce operational status, organization binding, feature entitlements, and resource limits.

## Local configuration

Copy `.env.example` to `.env` and fill the AqarDesk product values from the Mulverse console. `.env` is ignored by Git; do not put a private signing key in the desktop project.

| Variable | Value |
| --- | --- |
| `MULVERSE_LICENSE_API_BASE_URL` | Mulverse API base URL; production must use HTTPS. |
| `MULVERSE_LICENSE_PRODUCT_CODE` | AqarDesk product code as registered in Mulverse. |
| `MULVERSE_LICENSE_PRODUCT_ID` | AqarDesk product UUID from Mulverse. |
| `MULVERSE_LICENSE_KEY_PREFIX` | AqarDesk license-key prefix; the client adds a trailing hyphen when needed. Keys have four 5-character alphanumeric groups after the prefix. |
| `MULVERSE_LICENSE_PUBLIC_KEY_ID` | `kid` identifying the public Ed25519 verification key. |
| `MULVERSE_LICENSE_PUBLIC_KEY_B64` | Base64-encoded 32-byte public verification key. Never use a private key here. |

The desktop Rust build script loads these values from the workspace `.env` and passes them into the compiled application. Values supplied by the build process take precedence. Cargo rebuilds when the file or those environment variables change. Restart the desktop dev/build command after updating the values. With product identity or verification values missing, licensing remains unavailable and protected operations fail closed.

If trial registration succeeds but entitlement verification reports a mismatch, confirm that `MULVERSE_LICENSE_PRODUCT_ID` is the UUID of the same AqarDesk product configured in Mulverse and that `MULVERSE_LICENSE_PUBLIC_KEY_ID` and `MULVERSE_LICENSE_PUBLIC_KEY_B64` are a matching active verification key pair. Restart `desktop:dev` after changing `.env`.

## Mulverse catalog proposal

These are suggested catalog entries for the AqarDesk product. Create them in Mulverse using the exact entitlement and limit keys below; the app does not create remote products or plans.

| Plan key | Suggested entitlements | Suggested limits |
| --- | --- | --- |
| `STARTER` | `properties`, `units`, `tenants`, `payments`, `contracts`, `user_management` | `properties.max=1`, `units.max=20`, `tenants.max=50`, `users.max=2`, `currencies.max=1`, `contract_templates.max=0` |
| `PRO` | All `STARTER` entitlements plus `contract_templates`, `audit_log`, `multi_currency` | `properties.max=5`, `units.max=250`, `tenants.max=1000`, `users.max=10`, `currencies.max=5`, `contract_templates.max=20` |
| `BUSINESS` | All supported entitlements | `properties.max=50`, `units.max=3000`, `tenants.max=15000`, `users.max=50`, `currencies.max=20`, `contract_templates.max=100` |
| `TRIAL` | All supported entitlements | Suggested trial: `properties.max=5`, `units.max=250`, `tenants.max=500`, `users.max=5`, `currencies.max=3`, `contract_templates.max=10` |

Supported entitlements: `properties`, `units`, `tenants`, `payments`, `contracts`, `user_management`, `contract_templates`, `audit_log`, and `multi_currency`.

Supported limits: `properties.max`, `units.max`, `tenants.max`, `users.max`, `currencies.max`, and `contract_templates.max`. An omitted or null limit is treated as unlimited, so set every applicable limit explicitly in the Mulverse catalog. A 14-day trial is a suggested starting point; choose the actual trial duration in Mulverse, where trial registrations and refreshes are centrally signed and are not granted from a local clock.

The suggested quotas are initial business defaults, not hard-coded plan decisions. Adjust the catalog in Mulverse before issuing licenses if different commercial limits are needed.
