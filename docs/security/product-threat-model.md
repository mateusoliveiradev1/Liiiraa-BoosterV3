# Product Threat Model

This baseline threat model covers Liiiraa Booster V1: the Tauri desktop
interface, privileged Windows agent, updater, remote catalog, cloud API,
benchmark and telemetry flows, and PUBG/anti-cheat trust boundary.

## Security Objectives

- A frontend compromise must not become local file, registry, shell, or
  privileged agent control.
- Privileged changes must be typed, allowlisted, audited, backed up where
  applicable, reversible, and denied by default.
- Updates and remote catalogs must be signed, validated, channel-scoped, and
  fail closed.
- Cloud and telemetry systems must collect the minimum necessary data with
  explicit consent, redacted logs, and server-side secrets only.
- PUBG and BattlEye trust must be preserved by avoiding game memory access,
  anti-cheat file changes, kernel debug/test-signing changes, and bypass-like
  behavior.

## Assets

| Asset | Security goal |
| --- | --- |
| Local Windows settings, registry, services, power plans, driver profiles | Change only through validated tweak definitions with backup and rollback metadata. |
| Elevated agent protocol | Accept only known command IDs and structured arguments from authorized callers. |
| User benchmark, telemetry, crash, and device metadata | Keep opt-in, minimal, redacted, and deletable where the product stores it. |
| API credentials, Neon credentials, update signing keys | Never ship to desktop/web clients or committed files. |
| Release artifacts, updater metadata, remote tweak catalog | Verify signatures and integrity before use. |
| PUBG files, BattlEye components, and game process state | Read only safe install/config metadata; never tamper with anti-cheat or game memory. |

## Trust Boundaries

1. Renderer/UI boundary: React/Tauri frontend code is treated as untrusted after
   XSS, dependency compromise, or local asset tampering.
2. IPC boundary: every desktop command crosses a validation and capability
   boundary before it reaches native code.
3. Privileged agent boundary: elevated operations run outside the renderer and
   require narrow command IDs, structured arguments, audit records, and deny
   rules.
4. Update/catalog boundary: release metadata, binaries, and remote tweak
   catalogs are untrusted until signatures, channels, schema, and integrity are
   verified.
5. API boundary: cloud endpoints assume client input, telemetry payloads, and
   benchmark sync requests are hostile until validated and rate limited.
6. Anti-cheat boundary: PUBG and BattlEye state is a separate trust domain; the
   optimizer must avoid behavior that resembles cheat tooling or bypasses.

## Threat Scenarios and Controls

| Area | Threat scenario | Required controls | Verification hooks |
| --- | --- | --- | --- |
| Frontend compromise | Malicious UI code invokes hidden native commands, reads secrets, writes arbitrary files, or asks the app to execute shell/registry changes. | Strict CSP, no remote scripts, scoped Tauri capabilities, typed IPC schemas, command allowlist, no arbitrary shell/file APIs, no Neon or updater private keys in frontend artifacts. | Unknown IPC command denied, invalid IPC payload denied, secret scan, artifact scan for Neon URLs and private keys. |
| Privileged agent abuse | A compromised renderer or local user sends arbitrary paths, command injection strings, dangerous registry edits, game-memory requests, or kernel/debug changes to the elevated agent. | Command IDs instead of raw commands, fixed executable paths, canonical path validation, deny dangerous paths, structured arguments, request authorization, audit log, rollback references, hard denials for anti-cheat and kernel/debug surfaces. | Unknown agent command denied, unsafe path denied, command injection string denied, Lab tweak blocked in Safe mode, anti-cheat boundary cannot be bypassed. |
| Updater | An attacker performs MITM, downgrade, malicious channel switch, unsigned binary delivery, compromised catalog delivery, or private signing-key exposure. | Tauri signature verification, signed release tags, code-signed installer, channel allowlist, no updater private key in artifacts, signed catalog metadata, rollback/kill-switch for bad catalogs, fail-closed update handling. | Unsigned update denied, invalid catalog denied, revoked catalog denied, release artifact scan, channel metadata validation. |
| API | Hostile clients bypass validation, abuse CORS, flood endpoints, force verbose errors, inject database payloads, or trick future auth-ready endpoints. | Zod/tRPC validation, CORS allowlist, rate limits, request IDs, redacted errors, server-side Neon credentials only, least-privilege database roles, contract tests for public/private boundaries. | API validation tests, rate-limit tests, CORS tests, error-redaction tests, no Neon URL in desktop/web artifacts. |
| Telemetry | Benchmark, device, crash, or audit data is collected without consent, contains PII/secrets, leaks local paths, or becomes re-identifiable. | Opt-in consent gates, data minimization, payload schema allowlists, local pending-sync queue, redaction before logging/upload, user-visible privacy settings, no raw credentials or full local paths. | Consent tests, telemetry payload snapshots, redaction tests, crash-report opt-in tests. |
| Anti-cheat trust | Optimizer actions look like cheat tooling by touching PUBG memory, BattlEye files/services, test-signing/debug BCD flags, hacked kernels, or anti-cheat dependencies. | No game memory access, no BattlEye file/service modification, no kernel/test-signing/debug tweaks, no bypass guidance, read-only game detection, config snapshot before suggestions, official driver/profile APIs only, audit notes for game-related changes. | Anti-cheat denial tests, PUBG guardrail tests, no kernel/debug tweak tests, catalog review for anti-cheat notes. |

## Denied Behavior

The product must explicitly refuse these actions even when a third-party
optimizer claims a performance benefit:

- Disable Defender globally.
- Disable Windows Update globally.
- Disable UAC.
- Disable the pagefile globally.
- Rename or replace Windows system binaries.
- Apply bulk undocumented registry packs.
- Enable kernel debugging, test signing, hacked kernels, or BCD timer folklore.
- Disable anti-cheat services, alter BattlEye files, or touch PUBG process
  memory.
- Ship arbitrary scripts in tweak catalogs or updates.
- Store API, Neon, telemetry, or updater signing secrets in frontend or desktop
  artifacts.

## Review Gates

Security review is required before adding any new elevated command, update
mechanism, remote catalog capability, telemetry field, local secret, Lab tweak,
native-code dependency, or release build. The review must confirm that the
change has a deny-by-default path, validation coverage, rollback or explicit
irreversibility notes, and a test hook tied to the scenarios above.
