## ADDED Requirements

### Requirement: Desktop Shell
The desktop app SHALL be implemented with Tauri 2, React, TypeScript, and Vite.

#### Scenario: App startup
- **WHEN** the user launches the app
- **THEN** the app SHALL show system status, optimization state, and rollback availability without requiring authentication.

### Requirement: Typed IPC
The desktop app SHALL use generated TypeScript bindings for Rust commands.

#### Scenario: Command signature changes
- **WHEN** a Rust command input or output changes
- **THEN** the generated TypeScript bindings SHALL cause frontend call sites to typecheck against the new contract.

### Requirement: Optimization Flow
The desktop app SHALL separate scan, plan, apply, verify, benchmark, and rollback flows.

#### Scenario: User starts optimization
- **WHEN** the user requests optimization
- **THEN** the app SHALL produce a plan before applying any tweak.

#### Scenario: User reviews changes
- **WHEN** a plan contains Competitive or Lab tweaks
- **THEN** the UI SHALL display risk, reboot requirements, security tradeoffs, and rollback availability.

### Requirement: Strong Visual Identity
The desktop UI SHALL present a premium, technical, performance-focused visual style.

#### Scenario: Dashboard
- **WHEN** the dashboard renders
- **THEN** it SHALL prioritize scan score, performance state, active profile, before/after metrics, and rollback status over marketing copy.
