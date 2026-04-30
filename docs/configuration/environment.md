# Environment Configuration

Liiiraa Booster keeps committed environment files as templates only. Real local
values belong in ignored `.env` files, while deployed secrets belong in the
hosting or CI secret store.

## Templates

| File | Purpose | Secret policy |
| --- | --- | --- |
| `.env.example` | Shared local defaults for root tooling. | Public values only. |
| `apps/api/.env.example` | Fastify, tRPC, Neon, rate-limit, and logging settings. | Secret names are listed, but values stay empty. |
| `apps/desktop/.env.example` | Tauri/Vite desktop public runtime settings. | No database URLs, updater private keys, or service credentials. |
| `apps/web/.env.example` | Public web runtime settings. | No credentials or service tokens. |
| `packages/config/.env.example` | Local switches for shared config tooling. | Public values only. |

## Cloud Boundary

The desktop and web apps must call the cloud API for product data. They must not
connect directly to Neon. Neon connection strings are server-side API settings:

- `NEON_DATABASE_URL` is the pooled runtime connection used by the API.
- `NEON_DATABASE_DIRECT_URL` is reserved for reviewed Drizzle migrations when a
  direct connection is required.

## Local Setup

Copy only the template needed for the surface you are running:

```powershell
Copy-Item apps\api\.env.example apps\api\.env.local
Copy-Item apps\desktop\.env.example apps\desktop\.env.local
Copy-Item apps\web\.env.example apps\web\.env.local
```

Do not fill secret values into any `*.example` file. Keep local overrides in
ignored files such as `.env.local`.
