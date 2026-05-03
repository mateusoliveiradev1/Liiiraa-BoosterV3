## ADDED Requirements

### Requirement: Performance Console Visual Direction
The desktop app SHALL use a Liiiraa-native performance console visual system that communicates optimization, trust, telemetry, rollback, and measured improvement without relying on generic neon dashboards or marketing-style composition.

#### Scenario: Desktop shell renders
- **WHEN** the desktop app shell is displayed
- **THEN** it SHALL show graphite/charcoal surfaces, disciplined accent colors, crisp borders, compact controls, and optimizer-specific state rather than oversized hero copy, decorative gradients, or generic stacked dashboard cards.

#### Scenario: Visual accents render
- **WHEN** success, active, warning, danger, lab, or neutral states are shown
- **THEN** accents SHALL be used as semantic indicators, strokes, chart lines, badges, and active states rather than large decorative page backgrounds.

### Requirement: Tokenized Design System
The desktop UI SHALL define and consume named design tokens for color, surface, border, typography, spacing, radius, motion, shadow, density, risk, and chart semantics.

#### Scenario: Component style is implemented
- **WHEN** a redesigned desktop component is added or updated
- **THEN** it SHALL use the shared token layer instead of new hardcoded one-off color, spacing, border, or shadow values.

#### Scenario: CSS theme is reviewed
- **WHEN** the visual QA pass reviews desktop CSS
- **THEN** it SHALL confirm that no single hue family dominates the UI and that accent usage follows semantic token roles.

### Requirement: Operational First Screen
The Dashboard SHALL prioritize actionable optimizer state over a welcome or marketing panel.

#### Scenario: Dashboard loads
- **WHEN** the Dashboard route becomes active
- **THEN** the first viewport SHALL show readiness, current bottleneck or next best action, scan/apply state, active optimization mode, rollback availability, benchmark delta, update/signing trust, and key hardware/game status.

#### Scenario: Dashboard has no scan result
- **WHEN** no scan result is available
- **THEN** the Dashboard SHALL still show a clear read-only scan call to action, trust/update state, rollback policy, and what information will be collected before any write is possible.

### Requirement: Diagnostic Route Composition
Each major desktop route SHALL use layout patterns appropriate to optimization work: command headers, status strips, ledgers, timelines, inspectors, charts, and action bars.

#### Scenario: Scan route renders
- **WHEN** the Scan route is active
- **THEN** it SHALL show scan scope, read-only progress, current module, completed modules, findings grouped by risk/impact, and plan-generation state.

#### Scenario: Optimize route renders
- **WHEN** the Optimize route is active
- **THEN** it SHALL show Safe, Competitive, Lab, and Blocked groups as an audited tweak ledger with exact change, expected impact, confidence/source, risk, reboot, rollback, and consent status.

#### Scenario: Apply or verify flow renders
- **WHEN** an apply or verify workflow is active
- **THEN** the UI SHALL show backup, apply, verify, benchmark prompt, failure, and rollback states as a timeline with current, completed, and pending changes.

#### Scenario: Rollback route renders
- **WHEN** the Rollback route is active
- **THEN** it SHALL show optimization sessions as a recovery timeline with before/after values where safe, restore actions, reboot markers, verification state, and exportable audit context.

### Requirement: Hardware And Game Optimization Surfaces
Power, NVIDIA, PUBG, and Benchmarks routes SHALL feel like dedicated optimizer tools rather than generic information pages.

#### Scenario: Power route renders
- **WHEN** the Power route is active
- **THEN** it SHALL show active Windows plan, Liiiraa plan options, laptop/desktop policy, power tradeoffs, rollback state, and what will change before applying.

#### Scenario: NVIDIA route renders
- **WHEN** the NVIDIA route is active
- **THEN** it SHALL show GPU/driver state, profile backup status, global/PUBG profile state, display/FPS policy, safety warnings, and rollback availability.

#### Scenario: PUBG route renders
- **WHEN** the PUBG route is active
- **THEN** it SHALL show install/config detection, BattlEye boundary state, DirectX benchmark choice, legacy launch option cleanup, competitive checklist, NVIDIA profile link, and benchmark call to action.

#### Scenario: Benchmarks route renders
- **WHEN** the Benchmarks route is active
- **THEN** it SHALL show before/after data, average FPS, 1% low, 0.1% low, p95 frame time, variance, capture metadata, and confidence warnings.

### Requirement: Risk And Trust Presentation
Risk, consent, reboot, rollback, privacy, update, and signing state SHALL be visible through text and shape/icon treatment, not color alone.

#### Scenario: Risky tweak is shown
- **WHEN** a Competitive, Lab, danger, blocked, or reboot-required tweak is displayed
- **THEN** it SHALL include a textual risk label, a distinguishable icon or shape, consent requirement, rollback or manual recovery status, and the reason it is not part of the default safe apply path.

#### Scenario: Trust state is shown
- **WHEN** Settings, Dashboard, or status surfaces show trust/update state
- **THEN** they SHALL include "Signed by Liiiraa", update channel/signature state, local data/privacy state, and no-secret/local-first messaging where relevant.

### Requirement: Visual QA Gate
The redesigned desktop UI SHALL pass automated and human-reviewable visual quality checks before the change is considered implementation-ready.

#### Scenario: Desktop visual smoke runs
- **WHEN** the desktop visual smoke test runs
- **THEN** it SHALL navigate all redesigned routes and assert that optimizer state, command navigation, status strip, at least one tweak ledger, benchmark proof, rollback state, and settings/trust content are visible.

#### Scenario: Responsive screenshot review runs
- **WHEN** screenshots are captured for supported desktop widths
- **THEN** they SHALL show no blank primary surface, no overlapping text or controls, no clipped primary actions, no color-only risk state, and no first-screen marketing hero replacing optimizer state.
