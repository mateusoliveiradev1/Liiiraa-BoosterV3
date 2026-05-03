## 1. Audit And Setup

- [x] 1.1 Capture current desktop screenshots for Dashboard, Scan, Optimize, Power, NVIDIA, PUBG, Benchmarks, Rollback, and Settings as redesign baselines.
- [x] 1.2 Inventory hardcoded desktop UI strings in `apps/desktop/src/**` and shared visible strings in `packages/ui/src/**`.
- [x] 1.3 Identify current ad hoc CSS colors, gradients, shadows, radii, and layout values that must move to design tokens.
- [x] 1.4 Document the chosen default locale and fallback order for the first implementation pass.

## 2. Design Token Foundation

- [x] 2.1 Add a desktop design-token module for graphite surfaces, steel borders, typography, spacing, density, radius, motion, state colors, and chart colors.
- [x] 2.2 Map design tokens into CSS variables consumed by `apps/desktop/src/styles.css`.
- [x] 2.3 Replace current radial/neon background treatments with restrained performance-console surfaces.
- [x] 2.4 Add semantic token usage for active, success, warning, danger, lab, neutral, trust, rollback, and benchmark states.
- [x] 2.5 Add lintable or reviewable CSS guidance to prevent new one-off dominant cyan/green/purple styling.

## 3. Localization Readiness

- [x] 3.1 Add typed locale catalogs or placeholders for `pt-BR`, `en-US`, and `es-ES`.
- [x] 3.2 Add a typed translation helper with interpolation and default-locale fallback.
- [x] 3.3 Add a shared optimizer glossary for scan, apply, rollback, benchmark, risk, reboot, confidence, source, Safe, Competitive, Lab, and Blocked terms.
- [x] 3.4 Migrate shell navigation, status strip, common action buttons, risk badges, and tooltips to locale keys.
- [x] 3.5 Add a development/test signal for missing locale keys in redesigned surfaces.

## 4. Core UI Primitives

- [x] 4.1 Build a compact command header that shows route title, next best action, primary controls, and trust/update context.
- [x] 4.2 Rework `StatusStrip` into a stable telemetry strip with compact labels, accessible tooltips, and no text overflow.
- [x] 4.3 Build metric readout, risk badge, mode segmented control, icon toolbar, and trust badge primitives using the new tokens.
- [x] 4.4 Build tweak ledger rows with exact change, impact, confidence/source, risk, reboot, rollback, and consent status.
- [x] 4.5 Build diff, apply timeline, rollback session log, and benchmark chart primitives.
- [x] 4.6 Ensure icon-only controls use lucide icons with accessible names and tooltips.

## 5. Route Redesign

- [x] 5.1 Redesign Dashboard as an operational cockpit with readiness, bottleneck, next action, scan/apply state, rollback, benchmark, trust, and hardware/game status in the first viewport.
- [x] 5.2 Redesign Scan around read-only scope, current module progress, completed modules, findings, and generate-plan state.
- [x] 5.3 Redesign Optimize around the audited tweak ledger for Safe, Competitive, Lab, and Blocked groups.
- [x] 5.4 Redesign apply/verify surfaces to show backup, apply, verify, benchmark prompt, failure, and rollback timeline states.
- [x] 5.5 Redesign Power with active plan, Liiiraa plan options, laptop/desktop policy, tradeoffs, and rollback state.
- [x] 5.6 Redesign NVIDIA with GPU/driver state, profile backups, global/PUBG profile state, display/FPS policy, warnings, and rollback.
- [x] 5.7 Redesign PUBG with install/config detection, BattlEye boundary, DirectX benchmark choice, launch option cleanup, competitive checklist, NVIDIA profile link, and benchmark CTA.
- [x] 5.8 Redesign Benchmarks with average FPS, 1% low, 0.1% low, p95 frame time, variance, metadata, and confidence warnings.
- [x] 5.9 Redesign Rollback with session timeline, before/after values where safe, restore actions, reboot markers, verification state, and audit export context.
- [x] 5.10 Redesign Settings with privacy, telemetry, update channel, signing trust, local data controls, advanced gates, and "Signed by Liiiraa" visibility.

## 6. Verification And Documentation

- [x] 6.1 Update desktop Playwright smoke tests to navigate all redesigned routes and assert required optimizer state.
- [x] 6.2 Add screenshot checks for supported desktop widths and verify no blank primary surfaces, overlap, clipped primary actions, or marketing hero regressions.
- [x] 6.3 Add locale visual checks for Portuguese and Spanish text expansion on navigation, buttons, badges, status strip, tables, and compact controls.
- [x] 6.4 Run desktop typecheck, lint/check scripts, and visual tests.
- [x] 6.5 Update visual design and UI/UX docs with the final Liiiraa performance-console rules and locale workflow.
