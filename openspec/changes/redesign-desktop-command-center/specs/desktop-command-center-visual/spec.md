## ADDED Requirements

### Requirement: Desktop Launch Surface
The desktop app SHALL launch into a dedicated optimizer command center rather than the public web landing page.

#### Scenario: Desktop app starts
- **WHEN** the user launches the Tauri desktop app in development or debug mode
- **THEN** the main window SHALL show desktop optimizer navigation, runtime status, and optimization workflow state.

#### Scenario: Landing page remains separate
- **WHEN** the user opens the public web landing page
- **THEN** it SHALL remain accessible without becoming the desktop app's primary runtime surface.

### Requirement: Command Center Layout
The desktop UI SHALL use a product-specific command-center layout with left navigation, top status strip, main diagnostic workspace, contextual inspector where useful, and bottom apply or rollback actions during state-changing flows.

#### Scenario: Dashboard renders
- **WHEN** the dashboard route is active
- **THEN** the UI SHALL prioritize system readiness, active mode, scan state, power plan, GPU state, PUBG readiness, benchmark delta, rollback availability, and trust state.

#### Scenario: User navigates routes
- **WHEN** the user selects Dashboard, Scan, Optimize, Power, NVIDIA, PUBG, Benchmarks, Rollback, or Settings
- **THEN** the main workspace SHALL update without showing marketing hero copy or an empty placeholder page.

### Requirement: Tweak Workflow Visibility
The desktop UI SHALL expose scan, plan, apply, verify, benchmark, and rollback states as separate workflow steps.

#### Scenario: Optimization plan is reviewed
- **WHEN** a generated or mock optimization plan is displayed
- **THEN** the UI SHALL group tweak candidates into Safe, Competitive, Lab, and Blocked sections with exact change summaries, expected impact, risk, reboot status, and rollback status.

#### Scenario: Apply flow is active
- **WHEN** the user enters an apply or verify flow
- **THEN** the UI SHALL show the current step, changes already completed, changes pending, failures, and rollback availability.

### Requirement: Risk And Safety Presentation
The desktop UI SHALL make risk, consent, reboot, and rollback requirements visible without relying on color alone.

#### Scenario: Competitive or Lab tweak appears
- **WHEN** a Competitive or Lab tweak is shown
- **THEN** the UI SHALL include a textual risk label, icon or shape distinction, consent requirement, and rollback or manual review status.

#### Scenario: Blocked tweak appears
- **WHEN** a blocked or denied tweak is shown
- **THEN** the UI SHALL explain why it is blocked and SHALL NOT provide an apply control for that tweak.

### Requirement: Game And Hardware Surfaces
The desktop UI SHALL include dedicated surfaces for Power, NVIDIA, PUBG, and Benchmarks that reflect the planned optimizer product areas.

#### Scenario: PUBG surface renders
- **WHEN** the PUBG route is active
- **THEN** the UI SHALL show install/config detection, BattlEye safety state, DX benchmark choice, launch option warnings, competitive checklist, NVIDIA profile link, and benchmark call to action.

#### Scenario: NVIDIA surface renders
- **WHEN** the NVIDIA route is active
- **THEN** the UI SHALL show GPU and driver state, global profile state, PUBG profile state, backup status, display/FPS policy, and rollback availability.

#### Scenario: Benchmarks surface renders
- **WHEN** the Benchmarks route is active
- **THEN** the UI SHALL show before and after metrics, FPS lows, frametime distribution, benchmark metadata, and confidence or variance warnings.

### Requirement: Desktop Visual Quality Gate
The desktop command center SHALL include verification that the app is not a static landing shell and that key visual surfaces render correctly.

#### Scenario: Desktop visual smoke runs
- **WHEN** the desktop visual smoke check runs
- **THEN** it SHALL verify command-center navigation, status strip, at least one tweak plan section, rollback state, and settings/trust surface.

#### Scenario: Screenshot review runs
- **WHEN** screenshot verification is available
- **THEN** it SHALL capture desktop viewports and SHALL fail or report if text overlaps, primary controls overflow, or the first screen lacks optimizer state.
