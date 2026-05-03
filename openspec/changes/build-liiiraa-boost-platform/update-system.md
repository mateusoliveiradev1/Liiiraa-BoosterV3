# Update System

The update system must be boring, signed, observable, and recoverable.

## Goals
- Deliver app updates safely.
- Deliver remote tweak catalog updates safely.
- Support dev, beta, and stable channels.
- Prevent unsigned or tampered updates.
- Allow rollback/kill-switch for bad tweak catalog releases.
- Never store private updater keys in the desktop app or repository.

## Tauri App Updates
Use the Tauri updater plugin.

Required:
- `tauri-plugin-updater`
- updater public key embedded in `tauri.conf.json`
- private key stored only in secure release environment
- `createUpdaterArtifacts = true`
- HTTPS update endpoints in production
- signed updater artifacts
- Windows install mode selected intentionally, default `passive` unless UX testing chooses otherwise
- no `dangerousInsecureTransportProtocol` in production

Update metadata must include:
- version
- platform
- arch
- URL
- signature
- release notes
- publication date
- channel
- minimum supported previous version when needed

## Channels
- dev: internal testing, can break
- beta: real install path, signed, staged, for risky features first
- stable: only after beta soak or explicit approval

Channel rules:
- Lab tweak changes ship to beta first.
- Privileged-agent changes ship to beta first.
- Updater changes ship to beta first.
- Stable releases require signed tag and signed artifact.

## Remote Tweak Catalog Updates
Remote catalog is not code execution.

Catalog can change:
- tweak metadata
- eligibility rules
- source links
- risk classification
- defaultEnabled flags
- rollout percentages
- blocked versions

Catalog must not contain:
- raw arbitrary scripts
- shell command strings
- unsigned executable payloads
- new privileged command IDs not present in the signed app

Catalog rules:
- schema-versioned
- signed or integrity checked
- validated by app before use
- cached locally with last-known-good fallback
- rollout can be paused or rolled back
- high-risk changes require manual approval

## Rollback
App rollback:
- if update install fails, app should remain on previous installed version where installer supports it
- if new app starts but catalog fails validation, app uses last-known-good catalog
- if local optimization fails after update, rollback uses local snapshots independent from cloud

Catalog rollback:
- backend can disable a catalog version
- app refuses catalog versions marked revoked
- app reports catalog version in benchmark and audit logs

## Tests
- invalid signature rejected
- invalid JSON rejected
- older revoked version rejected
- unsupported platform ignored
- beta user does not receive stable-only metadata incorrectly
- stable user does not receive beta update unless opted in
- last-known-good catalog used when latest fails
- no private signing key in build output
