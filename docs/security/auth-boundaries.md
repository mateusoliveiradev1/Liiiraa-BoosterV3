# Auth-Ready API Boundaries

Authentication, billing, license enforcement, and account management are not
part of the current Liiiraa Booster implementation. The API contract still
reserves a private boundary so future auth can be added without moving public
procedures or weakening validation.

## Current Public API

The shipped public tRPC surface is the `apiProcedureContracts` export from
`packages/api-contract`. These procedures remain callable without auth and must
keep public-safe validation, rate limiting, error redaction, and least-privilege
logging:

- `benchmarks.sync`
- `catalog.latest`
- `featureflags.evaluate`
- `releases.channels`
- `releases.latest`
- `system.health`

Public procedures must not return Neon credentials, user secrets, raw local
paths, updater private keys, or internal feature-flag rules. Benchmark sync
continues to require explicit consent even while it is public.

## Reserved Private API

The `privateApiProcedureContracts` export is a reserved manifest only. It is not
loaded by the current API router and is not accepted by the default public
security envelope. The reserved private procedures are:

- `account.profile`
- `devices.register`
- `licenses.status`

Each reserved private procedure must keep:

- `visibility: "private"`
- `errorRedaction: "private"`
- a future auth policy with `auth.required: true`
- at least one private scope such as `account:read`, `devices:write`, or
  `licenses:read`
- principal-scoped rate limiting
- least-privilege audit fields that exclude tokens, raw payloads, cookies, and
  database URLs

## Future Auth Implementation Rules

When auth is implemented later, the runtime router may import private contracts
only after it has a real principal resolver, session or device-attestation
validator, principal-scoped rate limiter, and private error-redaction path.

Public and private procedures must stay in separate manifests. A private
procedure appearing in `apiProcedureContracts` is treated as a contract failure,
not as an auth feature.

The contract test suite verifies this separation with
`assertAuthReadyBoundaryCoverage()` and by proving that default public envelope
validation rejects reserved private procedures.
