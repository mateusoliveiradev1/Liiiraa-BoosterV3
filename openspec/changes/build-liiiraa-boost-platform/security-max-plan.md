# Security Max Plan

There is no absolute "maximum security", but this is the target bar for a sellable desktop optimizer that touches privileged Windows settings.

## Layers

### Repository
- signed commits
- signed tags
- protected `main`
- required checks
- no force-push to protected branches
- CODEOWNERS for sensitive paths
- Dependabot/security alerts
- secret scanning
- CodeQL
- pinned GitHub Actions for release workflows
- least-privilege GitHub Actions permissions
- artifact attestations for releases

### Desktop App
- Tauri capabilities scoped per window
- strict CSP
- no remote scripts
- typed IPC only
- command allowlist
- no arbitrary shell execution
- no Neon secrets
- no updater private key
- local secrets in OS-protected storage
- telemetry opt-in and redacted

### Elevated Agent
- narrow command protocol
- typed request schema
- request authorization
- command IDs instead of raw commands
- fixed executable paths
- structured arguments
- audit log
- rollback references
- deny unknown commands
- deny dangerous paths
- no game memory access
- no kernel/anti-cheat tampering

### Cloud API
- Zod validation
- tRPC contract tests
- rate limits
- CORS allowlist
- request ID and audit logs
- error redaction
- Neon credentials only server-side
- migrations tested before deploy
- future auth ready but not fake-auth now

### Release
- Windows code signing
- signed Tauri updates
- signed release tags
- private keys in protected CI environment only
- release provenance/artifact attestations
- rollback/kill-switch for remote tweak catalog

## Required Security Tests
- unknown IPC command denied
- invalid IPC payload denied
- unsafe path denied
- command injection string denied
- unsigned update denied
- invalid catalog denied
- revoked catalog denied
- secret scan passes
- no Neon URL in desktop artifact
- no updater private key in artifact
- Lab tweak cannot run in Safe mode
- anti-cheat boundary cannot be bypassed

## Security Review Gates
Security review is required before:
- new elevated command
- new update mechanism
- new remote catalog capability
- new telemetry field
- new local secret
- new Lab tweak
- new dependency that executes native code
- any release build
