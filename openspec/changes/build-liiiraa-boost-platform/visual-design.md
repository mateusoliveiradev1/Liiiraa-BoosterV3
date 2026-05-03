# Visual Design

## Brand
Product name: `Liiiraa Booster`

Signature:
```text
Signed by Liiiraa
```

The product should feel like a high-end Windows performance instrument: sharp, technical, confident, fast, and trustworthy. It must not look like a generic AI SaaS dashboard, a cloned gaming launcher, or a noisy RGB booster.

## Design Pillars
- Precision: every surface should feel measured, audited, and reversible.
- Power: visual energy comes from contrast, motion, telemetry, and performance data, not decorative clutter.
- Trust: risky actions must look serious and understandable.
- Game-readiness: PUBG and competitive modes should feel intense without becoming childish.
- Premium utility: dense enough for power users, clean enough for normal users.

## Visual Direction
The desktop app uses a graphite performance-console base with high-contrast
telemetry and restrained semantic accents. Accent colors are signals for active,
success, warning, danger, lab, trust, rollback, and benchmark states; they must
not become page-wide decorative washes.

Avoid:
- generic purple/blue gradient SaaS hero look
- fake glassmorphism everywhere
- huge empty cards
- rounded blobs/orbs as decoration
- cartoon gaming visuals
- emoji-based UI
- decorative text explaining obvious controls

Prefer:
- crisp panels
- compact controls
- thin separators
- real graphs
- hardware/status indicators
- risk badges
- command-style timelines
- icon-first toolbars with tooltips
- strong first screen with actual optimization state

Final desktop rules:
- first viewport shows optimizer state, not a welcome hero
- route composition favors command headers, status strips, ledgers, timelines, inspectors, charts, and action bars
- no card-inside-card nesting
- repeated cards are allowed only for repeated entities or framed tools
- risk, reboot, rollback, trust, privacy, update, and signing states use text plus icon or shape, never color alone
- new colors, shadows, radii, spacing, density, and motion values start in the desktop design-token layer

## Logo Direction
The logo should be custom and recognizable at tray-icon size.

Concept:
- A stylized `L` mark built from speed-line geometry and a shield/bolt negative-space hint.
- Works in monochrome, full color, and Windows tray sizes.
- No generic rocket, lightning-only, skull, crosshair-only, or esports mascot.

Deliverables:
- `logo.svg`
- `logo-mark.svg`
- `app-icon.ico`
- `tray-icon.ico`
- `favicon.svg`
- `social-preview.png`

Logo rules:
- keep the mark readable at 16px
- preserve clearspace equal to 25 percent of mark width
- never place on low-contrast backgrounds
- never distort, glow excessively, or add uncontrolled shadows

## Color System
Base:
- background: near-black graphite, not pure black
- surface: layered charcoal
- border: subtle steel
- text primary: near-white
- text secondary: muted cool gray

Accents:
- performance green for verified improvements
- electric cyan for active telemetry
- amber for caution/competitive tradeoffs
- red for high-risk/lab/danger
- violet is allowed only as a secondary accent, never the dominant theme

Rules:
- meet WCAG 2.2 AA contrast for text and essential controls
- state colors must not rely on color alone
- risk levels must include label/icon/shape differences
- charts must have accessible legends and tooltips

## Typography
Use a modern sans-serif for UI and a technical monospace for metrics.

Suggested direction:
- UI: Inter, Geist, or similar
- Metrics/code: JetBrains Mono or Geist Mono

Rules:
- no negative letter spacing
- no viewport-width font scaling
- compact panels use compact type
- metrics must be tabular where numbers compare
- long labels must wrap or truncate gracefully with tooltip

## Iconography
Use a consistent icon family, preferably lucide-react for app UI.

Icon usage:
- power plan: gauge/zap/cpu
- rollback: rotate-ccw/history
- security: shield/lock
- NVIDIA/GPU: microchip/monitor
- PUBG/game: crosshair/gamepad/target where appropriate
- benchmark: chart/noise/activity
- risk: triangle-alert/octagon-alert

Rules:
- icon-only buttons require accessible label and tooltip
- common commands should use icons before text-heavy buttons
- no mixed icon styles
- no hand-drawn one-off SVGs unless they are brand assets

## Layout
Primary layout:
```text
left rail navigation
top status strip
main diagnostic workspace
right contextual inspector when needed
bottom action/rollback bar for apply flows
```

Key screens:
- Dashboard: system score, active profile, last benchmark delta, rollback state, next best action.
- Scan: hardware, OS, services, GPU, network, storage, security, game readiness.
- Optimize: plan review with Safe/Competitive/Lab segmentation.
- Power: Liiiraa plans, active Windows plan, laptop warnings.
- NVIDIA: global profile, PUBG profile, backup state, driver info.
- PUBG: install detection, DX benchmark, config recommendations, BattlEye-safe boundaries.
- Benchmarks: before/after charts, frametime, 1% low, notes.
- Rollback: timeline of changes, restore points, audit log.
- Settings: privacy, telemetry, updates, signing/trust info.

## Components
Core components:
- `StatusStrip`
- `SystemScoreGauge`
- `MetricTile`
- `FrametimeChart`
- `RiskBadge`
- `TweakPlanList`
- `TweakDiffPanel`
- `RollbackTimeline`
- `ProfileSelector`
- `ModeSegmentedControl`
- `DriverStatusPanel`
- `PowerPlanCard`
- `BenchmarkComparison`
- `SecurityBoundaryNotice`

Component rules:
- no card-inside-card nesting
- cards only for repeated items or genuine framed tools
- toolbars use icons with tooltips
- risky actions require confirmation UI with exact changes listed
- destructive/rollback actions must be visually distinct but not melodramatic

## Motion
Motion should communicate system activity, not decorate.

Use:
- progress scan sweep
- chart line reveal
- small apply/verify state transitions
- subtle status pulse for active benchmark capture

Avoid:
- endless background animation
- flashy RGB motion
- heavy blur animation
- animation during benchmark capture that could affect performance perception

Respect reduced-motion preferences.

## Landing Page Visual
The landing page must use actual product visuals.

Hero:
- product name or direct offer as H1
- real app screenshot/mockup as primary visual
- no split generic card layout
- no generic gradient-only background
- show hint of the next section in first viewport

Sections:
- proof/benchmarks
- PC-wide optimization
- PUBG focus
- safety and rollback
- signed by Liiiraa trust section
- future pricing/auth placeholder

## Quality Bar
Before visual work is accepted:
- desktop screenshots at supported widths must be reviewed
- text cannot overflow or overlap
- contrast must pass AA for essential content
- icon-only buttons must have labels/tooltips
- app must still feel fast with dummy benchmark data
- no placeholder logo in final public-facing screens
- no blank primary route surfaces
- no clipped primary actions
- no marketing hero replacing optimizer state
- Portuguese and Spanish locale-fit screenshots must keep navigation, buttons, badges, status strip, tables, and compact controls stable

## Desktop Locale Workflow
Desktop UI copy should flow through typed locale keys in
`packages/ui/src/localization.ts`. `en-US` is the default locale for the current
implementation pass, while `pt-BR` and `es-ES` partial catalogs fall back to
`en-US` until final translations are filled.

Manual visual checks can switch locale with `?locale=pt-BR` or `?locale=es-ES`
before the hash route. New compact labels need a tooltip, wrap behavior, or
stable truncation strategy before they are accepted.
