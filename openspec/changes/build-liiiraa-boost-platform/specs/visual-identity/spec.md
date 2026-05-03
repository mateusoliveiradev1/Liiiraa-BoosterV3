## ADDED Requirements

### Requirement: Liiiraa Brand Identity
The product SHALL have a distinct Liiiraa-branded visual identity and SHALL NOT rely on generic AI-generated SaaS styling.

#### Scenario: Public screen rendered
- **WHEN** a desktop or landing page screen is public-facing
- **THEN** it SHALL include polished Liiiraa visual identity through logo, typography, color, layout, iconography, and product-specific visual assets.

### Requirement: Visual Design Guide
The project SHALL maintain `visual-design.md` as the source of truth for app and landing page visuals.

#### Scenario: UI component added
- **WHEN** a new UI component is implemented
- **THEN** it SHALL follow the visual design guide for color, typography, spacing, icons, states, and accessibility.

### Requirement: UX Flow Spec
The project SHALL maintain `ui-ux-spec.md` as the source of truth for product flows.

#### Scenario: New screen implemented
- **WHEN** a desktop or landing screen is implemented
- **THEN** it SHALL satisfy the relevant flow, state, and safety rules in `ui-ux-spec.md`.

### Requirement: Logo and Icon Assets
The product SHALL include custom logo and icon assets suitable for app window, Windows tray, installer, favicon, and marketing usage.

#### Scenario: App packaged
- **WHEN** a desktop build is packaged
- **THEN** it SHALL use Liiiraa Booster app icons rather than default framework icons.

### Requirement: Performance-Oriented UI
The visual system SHALL communicate performance, trust, rollback safety, and real telemetry.

#### Scenario: Dashboard shown
- **WHEN** the dashboard loads
- **THEN** it SHALL show real system status, optimization readiness, benchmark deltas, and rollback state before marketing text.

### Requirement: Non-Generic Layout
The desktop app SHALL use a product-specific command-center layout rather than generic stacked dashboard cards.

#### Scenario: Main app shell rendered
- **WHEN** the shell renders
- **THEN** it SHALL include left rail navigation, top status strip, main diagnostic workspace, and contextual inspector/action area where appropriate.

### Requirement: Accessible High-Impact Design
The UI SHALL meet WCAG 2.2 AA expectations for essential text, controls, focus visibility, target sizing, and non-color-only state communication.

#### Scenario: Icon-only action
- **WHEN** an icon-only action is used
- **THEN** it SHALL include an accessible label and tooltip.

#### Scenario: Risk state shown
- **WHEN** a risk state is displayed
- **THEN** it SHALL use color plus text, icon, or shape.

### Requirement: Signed By Liiiraa Trust Signal
The app and landing page SHALL include a tasteful "Signed by Liiiraa" trust signal.

#### Scenario: Settings trust view
- **WHEN** the user opens trust/update settings
- **THEN** the app SHALL show signing, update, privacy, and "Signed by Liiiraa" information.
