## Context

The desktop app now has a real command-center shell, but the current visual layer still leans heavily on cyan/green neon gradients, glowing cards, and broad dashboard blocks. That makes the product feel closer to a generic gaming dashboard than a serious optimization instrument.

The product domain needs a sharper story. PC optimization apps that feel credible make the same few things obvious: what is being measured, what can safely change, what has risk, what improved, and how to undo it. Reference research points in that direction:

- VoltAgent `awesome-design-md` is useful as a design-system reference format, especially Raycast's product-as-chrome density, Linear's restrained surface ladder, and NVIDIA's disciplined accent-as-signal approach.
- Razer Cortex emphasizes game/system boosting, automatic activation, auto-restore, expert tuning, and "does not overclock" trust framing.
- NVIDIA App centers a unified GPU control surface, real-time FPS/performance overlays, GPU tuning, and game/app optimization.
- Intel Application Optimization exposes per-application enable/disable controls and explains real-time resource direction.
- AMD Software: Adrenalin presents one-click presets alongside advanced metrics, fan, power, GPU, and memory tuning.

The existing app is English-heavy, while the product direction needs Portuguese now or soon and English/Spanish later. This change should not block UI redesign on a full translation platform, but it must stop new screens from hardcoding every user-facing string directly inside components.

## Goals / Non-Goals

**Goals:**

- Make Liiiraa Booster look like a premium Windows performance console: technical, fast, trustworthy, dense, and reversible.
- Establish a first-viewport app experience that immediately shows optimizer state, not a welcome hero.
- Replace scattered CSS values with named design tokens for surfaces, borders, typography, charts, risk states, and motion.
- Rework major routes into diagnostic tools: Dashboard, Scan, Optimize, Power, NVIDIA, PUBG, Benchmarks, Rollback, and Settings.
- Make tweak review and apply flows feel like an audited change ledger with diffs, risk, confidence, reboot, rollback, and proof.
- Prepare copy for Brazilian Portuguese, English, and Spanish with typed locale keys and a simple fallback path.
- Preserve the local-first, rollback-first, narrow-permission safety model.

**Non-Goals:**

- Implement new Windows, GPU, CPU, storage, or PUBG optimization engine behavior.
- Add cloud sync, auth, billing, or external analytics.
- Clone any reference brand identity from `awesome-design-md`; references are directional, not source branding.
- Replace the public landing page in this change, except where shared visual tokens need to remain compatible.
- Add a heavy i18n framework unless implementation proves the typed dictionary path is insufficient.

## Decisions

1. Build a Liiiraa-native "performance console" visual system.

   The redesign should synthesize references instead of copying them. Use Raycast as a lesson in letting product UI become the visual identity, Linear as a lesson in restrained dark surfaces and sparse accent, NVIDIA as a lesson in treating green as a signal instead of a background, and optimization apps as a lesson in telemetry, per-app control, one-click presets, advanced tuning, and rollback language.

   Alternative considered: apply a single `awesome-design-md` style directly. That would improve taste quickly, but it would make Liiiraa feel derivative and could clash with the optimizer trust/safety domain.

2. Replace broad neon surfaces with tokenized graphite surfaces and controlled accents.

   Add a token layer that maps to CSS variables and component-level variants. The palette should use graphite/charcoal base layers, steel borders, near-white text, and limited accents: performance green, telemetry cyan, caution amber, danger red, and sparing violet for lab/advanced. Accent colors must appear in indicators, strokes, small badges, chart lines, and active states, not as giant page washes.

   Alternative considered: keep the existing palette and only tune spacing. That would keep the same "generic booster" impression the user is reacting to.

3. Change the Dashboard from welcome panel to operational cockpit.

   The first screen should show readiness, current bottleneck, active mode, next best action, scan/apply state, rollback readiness, benchmark delta, update/signing trust, and hardware/game state. The current "Ready to boost" hero pattern should be replaced with a compact command header plus diagnostic grid/ledger.

   Alternative considered: keep a large hero and add more metrics below it. That still makes the first impression feel like marketing instead of a utility app.

4. Move workflow UI toward ledgers, timelines, and inspectors.

   Tweak review should be row-first and audit-first: each row needs exact change, before/after where available, expected impact, confidence/source, risk, reboot, rollback, and action status. Apply should look like a verified installer timeline. Rollback should look like a recovery log. Benchmarks should show 1% low, 0.1% low, p95 frame time, variance, and metadata rather than average FPS alone.

   Alternative considered: keep repeated metric cards for every surface. Cards are fine for repeated entities, but they should not be the dominant structure for every workflow.

5. Introduce locale-ready copy without blocking the redesign.

   Start with a typed dictionary module and `t(key, params)` helper for desktop strings. Use `en-US` as the current default only if that matches implementation state, but create the structure so `pt-BR` and `es-ES` can be filled without changing component APIs. Tests should fail when new visible strings bypass the dictionary in redesigned surfaces.

   Alternative considered: adopt a full i18n package immediately. That may be useful later, but this proposal only needs a small, controlled copy boundary.

6. Treat visual QA as a real implementation gate.

   Add or update Playwright checks for desktop widths, route navigation, first-screen optimizer state, text fit, no overlapping controls, risk state labels, locale rendering, and screenshot review artifacts. Use stable route/test selectors and avoid brittle exact pixel comparisons except for obvious blank/overflow checks.

   Alternative considered: rely on manual review. The existing problem is visual/product fit, so the redesign needs repeatable guards.

## Risks / Trade-offs

- [Risk] Dense performance UI can become noisy. -> Mitigation: define information hierarchy, density tiers, consistent row heights, and collapse advanced details into inspectors/drawers.
- [Risk] A dark technical UI can drift into one-note cyan/green styling again. -> Mitigation: restrict accents by semantic role and review CSS for dominant hue balance before acceptance.
- [Risk] Locale-ready copy can slow implementation. -> Mitigation: start with a thin typed dictionary and migrate only redesigned desktop surfaces in this change.
- [Risk] Mock telemetry can look dishonest. -> Mitigation: label adapter/mock states clearly and keep proof/benchmark UI tied to available metadata.
- [Risk] Visual screenshot tests can be brittle. -> Mitigation: use structural assertions for required surfaces and screenshot artifacts for human review, not fragile full-page pixel thresholds.
- [Risk] Referencing external brands can lead to a copied identity. -> Mitigation: document the Liiiraa-specific synthesis and avoid importing reference tokens wholesale.

## Migration Plan

1. Inventory redesigned desktop surfaces and hardcoded user-facing strings.
2. Add a design-token module and CSS variable map, then replace current global neon/radial background treatment.
3. Refactor shared UI primitives for status strip, command header, metric readout, risk badge, tweak ledger row, diff panel, apply timeline, benchmark chart, and rollback session log.
4. Rebuild Dashboard first so the first screen proves the new direction before route-by-route migration.
5. Rework Scan, Optimize, Apply/Verify, Benchmarks, Rollback, Power, NVIDIA, PUBG, and Settings with the same tokens and component vocabulary.
6. Add locale dictionary scaffolding and migrate visible redesigned desktop copy into keys.
7. Update visual/design docs and Playwright smoke checks.
8. Run desktop typecheck, tests, and screenshot review before implementation is considered complete.

## Open Questions

- Should `pt-BR` become the default locale immediately, or should `en-US` remain default until the Portuguese copy pass is complete?
- Should benchmark charts stay custom SVG/CSS for now, or should a chart library be adopted when real telemetry volume increases?
- Should the desktop app expose a theme density toggle, or should compact density be the only V1 target?
