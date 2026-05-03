## ADDED Requirements

### Requirement: Signed Desktop Releases
Windows release artifacts SHALL be code-signed before public distribution.

#### Scenario: Unsigned release artifact
- **WHEN** a release artifact is unsigned
- **THEN** the release pipeline SHALL block stable distribution.

#### Scenario: Public identity shown
- **WHEN** the user opens release/update trust information
- **THEN** the app SHALL identify the product as signed by Liiiraa and show signing/update status clearly.

### Requirement: Signed Updater Artifacts
Tauri updater artifacts SHALL be signed and verified before installation.

#### Scenario: Update metadata missing signature
- **WHEN** update metadata lacks a valid signature
- **THEN** the app SHALL reject the update.

### Requirement: Update System Design
The update system SHALL follow `update-system.md`.

#### Scenario: App update implemented
- **WHEN** app update functionality is implemented
- **THEN** it SHALL include signed Tauri artifacts, HTTPS endpoints in production, channel rules, invalid-signature rejection, and no private keys in desktop or repo.

#### Scenario: Remote tweak catalog implemented
- **WHEN** remote tweak catalog updates are implemented
- **THEN** the catalog SHALL be schema-versioned, validated, integrity checked or signed, rollback-capable, and incapable of introducing arbitrary scripts or new privileged command IDs.

### Requirement: Signed Git Tags
Stable release tags SHALL be signed.

#### Scenario: Stable release tag
- **WHEN** a stable release is created
- **THEN** the release pipeline SHALL require a signed Git tag.

### Requirement: Release Channels
The product SHALL support dev, beta, and stable release channels.

#### Scenario: Risky optimizer change
- **WHEN** a release includes new privileged or Lab tweaks
- **THEN** it SHALL ship to beta before stable.

### Requirement: Remote Catalog Rollback
The remote tweak catalog SHALL support versioning, staged rollout, and rollback.

#### Scenario: Bad catalog version detected
- **WHEN** telemetry or support indicates a catalog version is unsafe
- **THEN** the backend SHALL be able to disable or roll back that catalog version.

### Requirement: Crash and Error Reporting
Crash/error reporting SHALL be opt-in or clearly disclosed, redacted, and tied to release version.

#### Scenario: Crash report generated
- **WHEN** a crash report is collected
- **THEN** it SHALL exclude secrets, personal files, and raw registry dumps unless explicitly approved.

### Requirement: Operational Audit
The platform SHALL keep operational audit logs for release changes, catalog changes, and privileged optimization actions.

#### Scenario: Catalog modified
- **WHEN** a tweak catalog entry changes
- **THEN** the change SHALL record author, reason, timestamp, source references, risk change, and rollback plan.
