# Platform domain architecture

The desktop backend is implemented directly in `backend` as a modular
monolith.

- `domain` owns validated identifiers, money, identity and access roles,
  properties, units, tenants, and payments.
- `application` owns use cases, authorization, and persistence ports.
- `infrastructure` implements SeaORM repositories, password hashing,
  transactions, and audit writes.
- `migration` initializes and verifies the additive platform schema.
- Tauri commands are thin presentation adapters in the desktop crate.

The platform currently implements authentication and administration,
properties, units, tenants, and payments. See
[`platform-v2.md`](./platform-v2.md) for migration and verification details.
