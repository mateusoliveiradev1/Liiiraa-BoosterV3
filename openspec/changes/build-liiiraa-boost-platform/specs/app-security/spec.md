## ADDED Requirements

### Requirement: Threat Model
The project SHALL maintain a threat model for the desktop app, privileged agent, updater, cloud API, local storage, telemetry, and anti-cheat trust boundaries.

#### Scenario: New privileged feature
- **WHEN** a feature adds a privileged command
- **THEN** the threat model SHALL be updated before implementation is considered complete.

### Requirement: Maximum Practical Security Baseline
The project SHALL use `security-max-plan.md` as the baseline for repo, desktop, elevated agent, cloud API, release, and telemetry security.

#### Scenario: Security-sensitive task
- **WHEN** a task touches privileged commands, updater, telemetry, remote catalog, local secrets, CI, signing, or cloud API security
- **THEN** the task SHALL satisfy the relevant checklist in `security-max-plan.md`.

### Requirement: Least-Privilege Desktop Permissions
The Tauri app SHALL use minimal capabilities and permissions per window or webview.

#### Scenario: Frontend route does not need filesystem access
- **WHEN** a window or route does not need filesystem access
- **THEN** it SHALL NOT receive filesystem permissions.

#### Scenario: Capability review
- **WHEN** a new Tauri plugin is added
- **THEN** its permissions SHALL be reviewed and scoped before merge.

### Requirement: Secure IPC Boundary
All frontend-to-Rust calls SHALL use typed commands, schema validation, and deny-by-default authorization.

#### Scenario: Unknown command
- **WHEN** frontend code invokes an unknown or unregistered privileged action
- **THEN** the request SHALL be denied and logged.

#### Scenario: Invalid command payload
- **WHEN** a command payload fails validation
- **THEN** the command SHALL make no system changes.

### Requirement: Elevated Agent Boundary
Privileged Windows actions SHALL be isolated behind a narrow elevated boundary with explicit command allowlists and audit logging.

#### Scenario: Shell-like argument injection
- **WHEN** an input contains shell control characters or unexpected paths
- **THEN** the elevated boundary SHALL reject it unless the command schema explicitly permits that value.

#### Scenario: Privileged action executed
- **WHEN** any privileged action runs
- **THEN** an audit entry SHALL record command ID, requester, plan ID, changed resources, timestamp, result, and rollback reference.

### Requirement: No Remote Code in Desktop Webview
The desktop app SHALL block remote scripts and unnecessary remote content through CSP and packaging policy.

#### Scenario: External script requested
- **WHEN** desktop UI attempts to load an unapproved remote script
- **THEN** CSP SHALL block it.

### Requirement: Secret Handling
The desktop build SHALL NOT contain Neon credentials, private signing keys, API admin secrets, or updater private keys.

#### Scenario: Release build
- **WHEN** a release build is produced
- **THEN** secret scanning SHALL verify that forbidden secrets are absent from artifacts.

### Requirement: Telemetry Privacy
Telemetry and benchmark sync SHALL be opt-in, redacted, and reversible.

#### Scenario: User disables telemetry
- **WHEN** telemetry is disabled
- **THEN** the app SHALL stop future uploads and keep benchmark history local unless manually exported.

### Requirement: API Security Baseline
The cloud API SHALL follow OWASP ASVS and OWASP API Security Top 10 baselines appropriate to the current feature set.

#### Scenario: Public procedure added
- **WHEN** a new public API procedure is added
- **THEN** it SHALL define validation, rate-limit policy, error-redaction behavior, and logging behavior.

### Requirement: Supply-Chain Security
The project SHALL check JavaScript and Rust dependencies for known vulnerabilities and risky licenses.

#### Scenario: Dependency update
- **WHEN** dependencies are updated
- **THEN** audit tooling SHALL run before merge.

### Requirement: Anti-Cheat Trust Boundary
The app SHALL avoid behavior that resembles anti-cheat bypass, kernel tampering, game memory manipulation, or game binary modification.

#### Scenario: Optimization conflicts with anti-cheat
- **WHEN** a tweak would modify game memory, BattlEye files, kernel debugging, test-signing, or driver signature state
- **THEN** the tweak SHALL be blocked.
