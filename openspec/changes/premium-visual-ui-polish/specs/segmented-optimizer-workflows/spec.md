## ADDED Requirements

### Requirement: Separated optimization categories
The desktop optimizer UI SHALL organize optimizations into clearly separated category lanes for game, system, network, GPU, power, startup/services, benchmark, rollback, and settings concerns where applicable.

#### Scenario: Category lane scan
- **WHEN** a user opens the optimizer dashboard
- **THEN** each available category shows a distinct title, summary, status, risk/trust signal, primary action, and detail entry point

#### Scenario: Category detail entry
- **WHEN** a user opens a category detail
- **THEN** the UI shows the relevant tweaks, expected impact, safety status, rollback context, and any required reboot or permission notes for that category

### Requirement: One-click optimization path
The optimizer SHALL provide a clear one-click path for recommended safe changes while keeping review, customize, and advanced controls available as secondary actions.

#### Scenario: Safe recommendation apply
- **WHEN** a scan has produced safe recommendations
- **THEN** the UI presents a single primary apply action, a secondary review/customize action, and a clear summary of what will change

#### Scenario: No unsafe bulk apply
- **WHEN** high-risk, lab, blocked, or irreversible changes are present
- **THEN** those changes are excluded from the one-click path unless the user explicitly reviews and consents to them

### Requirement: Optimizer button grammar
The optimizer SHALL use consistent action labels and button variants for scan, optimize, customize, review, apply, benchmark, restore, revert, skip, lock, and advanced actions.

#### Scenario: Clear next action
- **WHEN** a user lands on any optimizer route
- **THEN** the primary button communicates the next useful action for that route without requiring the user to inspect unrelated panels

#### Scenario: Destructive action separation
- **WHEN** a reset, revert, delete, or rollback action is available
- **THEN** it uses destructive or rollback styling, confirmation copy, and sufficient spacing from normal optimization actions

### Requirement: Advanced controls without clutter
The optimizer SHALL expose advanced and pro-style controls through tabs, drawers, inspectors, or expandable rows that do not overwhelm the default view.

#### Scenario: Default simple mode
- **WHEN** a non-expert user opens an optimization category
- **THEN** the default view focuses on recommendation, impact, safety, and a small set of actions

#### Scenario: Advanced mode
- **WHEN** a user opens advanced details
- **THEN** the UI displays exact setting names, before/after values, source/confidence, reboot need, rollback availability, and warnings without changing the default recommendation

### Requirement: Backup and rollback visibility
The optimizer SHALL keep backup and rollback status visible before, during, and after optimization actions.

#### Scenario: Before apply
- **WHEN** a user is about to apply optimizations
- **THEN** the UI states whether a restore point or app-level backup exists and what will be rollback-capable

#### Scenario: After apply
- **WHEN** optimizations finish
- **THEN** the UI shows completion state, benchmark prompt where relevant, rollback entry point, and any reboot or verification requirement

### Requirement: Game mode and supported game clarity
The optimizer SHALL present game-focused optimization as a distinct flow with supported game status, detected configuration, launch/profile notes, and game-specific benchmark context.

#### Scenario: Supported game detected
- **WHEN** the app detects a supported game or selected game profile
- **THEN** the UI shows the game state, available optimizations, related GPU/profile settings, and a clear optimize or review action

#### Scenario: Game not detected
- **WHEN** a supported game is not installed or not found
- **THEN** the UI displays a polished empty state with detection guidance and no misleading optimize action
