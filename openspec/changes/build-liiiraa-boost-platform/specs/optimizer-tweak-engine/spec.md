## ADDED Requirements

### Requirement: Tweak Definition Contract
Every optimization SHALL be represented as a `TweakDefinition` with metadata, risk, sources, detection, plan, backup, apply, verify, rollback, do, and dont fields.

#### Scenario: V1 tweak matrix coverage
- **WHEN** the V1 implementation is validated
- **THEN** every non-blocked entry in `v1-tweak-matrix.md` SHALL map to a `TweakDefinition`, recommendation definition, or explicitly deferred Lab stub
- **AND** every blocked entry SHALL map to a denial/guardrail test.

#### Scenario: New tweak added
- **WHEN** a developer adds a tweak
- **THEN** tests SHALL fail unless source links, safety classification, backup strategy, verification strategy, and rollback strategy are present.

#### Scenario: Tweak standard ignored
- **WHEN** a tweak does not satisfy `tweak-definition-standard.md`
- **THEN** it SHALL not be eligible for implementation completion.

### Requirement: Applicability and Conflict Validation
The engine SHALL validate OS, hardware, driver, power-source, display, game-process, anti-cheat, and setting-conflict rules before a tweak can enter an apply plan.

#### Scenario: Conflicting GPU latency settings
- **WHEN** a plan contains conflicting latency, sync, frame-cap, frame-generation, or overlay settings
- **THEN** the engine SHALL block the plan and explain which settings conflict.

#### Scenario: Unsupported hardware or driver
- **WHEN** a tweak requires a capability that is unavailable on the current PC
- **THEN** the tweak SHALL be marked unavailable or recommendation-only, not applied.

#### Scenario: Display color side effect
- **WHEN** a capture, FSO, HAGS, VRR, or overlay tweak can affect HDR, ICC, or color-management behavior
- **THEN** the app SHALL disclose the side effect before apply.

### Requirement: Optimization Modes
Tweaks SHALL be classified as Safe, Competitive, or Lab.

#### Scenario: Safe mode run
- **WHEN** the user runs Safe mode
- **THEN** only Safe tweaks SHALL be eligible for automatic application.

#### Scenario: Competitive mode run
- **WHEN** the user enables Competitive mode
- **THEN** the app SHALL disclose security, heat, power, reboot, and compatibility tradeoffs before applying.

#### Scenario: Lab mode run
- **WHEN** the user enables Lab mode
- **THEN** the app SHALL require explicit opt-in per dangerous category and SHALL create backups before applying.

### Requirement: Dry Run First
The engine SHALL support dry-run planning for every tweak.

#### Scenario: Missing dry-run support
- **WHEN** a tweak cannot describe its planned changes without applying them
- **THEN** the tweak SHALL be blocked from Safe and Competitive modes.

### Requirement: Rollback Integrity
The engine SHALL store enough information to revert every applied tweak.

#### Scenario: Apply succeeds but verify fails
- **WHEN** verification fails after applying a tweak
- **THEN** the engine SHALL mark the tweak rollback-required and offer immediate rollback.

### Requirement: Dangerous Action Guardrails
The engine SHALL block known dangerous actions unless a Lab tweak explicitly owns them and the user opted in.

#### Scenario: System file rename
- **WHEN** a tweak attempts to rename, replace, or delete Windows system binaries
- **THEN** the engine SHALL deny the action by default.

#### Scenario: Anti-cheat bypass behavior
- **WHEN** a tweak attempts to disable driver signature enforcement, test-signing protections, kernel protections, or anti-cheat files
- **THEN** the engine SHALL deny the action.
