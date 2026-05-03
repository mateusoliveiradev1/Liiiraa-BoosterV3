## Why

The current desktop UI technically exposes optimizer workflows, but the visual language still feels like a generic neon dashboard instead of a credible Windows performance optimizer. This matters now because the app's core promise is trust, speed, rollback safety, and measurable performance gains; the interface needs to communicate those qualities before deeper engine work becomes harder to present.

## What Changes

- Replace the current cyan/green neon dashboard treatment with a complete Liiiraa Booster performance-console art direction: engineered graphite surfaces, disciplined accent use, dense telemetry, sharp control hierarchy, and less decorative glow.
- Redesign Dashboard, Scan, Optimize, Power, NVIDIA, PUBG, Benchmarks, Rollback, and Settings around real optimizer jobs: diagnose, compare, choose a mode, apply, verify, benchmark, and undo.
- Introduce a design-token layer for colors, surfaces, typography, spacing, density, borders, state badges, charts, and motion so the desktop app does not depend on ad hoc CSS values.
- Add a stronger first-screen information architecture: system readiness, current bottleneck, next best action, live scan/apply status, rollback availability, and trust/update state must be visible without a marketing-style welcome panel.
- Rework tweak review UX into a more serious optimizer workflow with exact diffs, confidence, risk, reboot, rollback, source, and benchmark proof rather than broad card summaries.
- Make the interface localization-ready: current UI copy may remain English during the transition, but strings must move toward keyed copy with Brazilian Portuguese, English, and Spanish support planned.
- Preserve the existing safety model: no hidden privileged writes, no unsafe "optimize all", no color-only risk communication, and no regression away from rollback-first UX.
- No breaking changes to Windows optimization engines, Tauri IPC contracts, or public landing behavior are intended.

## Capabilities

### New Capabilities

- `optimizer-experience-visual-system`: Defines the complete desktop optimizer visual system, screen composition, component behavior, telemetry presentation, and visual QA gates.
- `optimizer-localization-readiness`: Defines how desktop UI copy is prepared for Portuguese, English, and Spanish without blocking the current English UI transition.

### Modified Capabilities

- None.

## Impact

- Affected code: `apps/desktop/src/App.tsx`, `apps/desktop/src/styles.css`, `apps/desktop/src/components/**`, `apps/desktop/src/routes/**`, `apps/desktop/src/adapters/**`, and shared typed UI data in `packages/ui/src/**`.
- Affected docs/specs: visual design guidance, UI/UX guidance, and desktop visual smoke expectations.
- Affected tests: desktop Playwright visual/smoke checks should assert layout, route surfaces, risk states, text fit, chart/metric rendering, and locale-ready strings.
- Potential dependencies: design-token structure and possibly a lightweight chart/rendering helper; prefer existing React, CSS, SVG, and lucide-react patterns unless a new dependency clearly reduces risk.
- Runtime constraints: keep the desktop local-first, avoid broad Tauri permissions, keep no-secret guarantees, and keep rollback/trust surfaces visible.
