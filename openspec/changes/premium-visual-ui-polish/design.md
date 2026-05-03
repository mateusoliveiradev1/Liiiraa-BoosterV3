## Context

Liiiraa Booster already has desktop optimizer routes, shared UI tokens, locale scaffolding, and web landing assets, but the product still needs a final visual and UX pass that feels premium rather than merely functional. The user explicitly wants the clarity and polish of Hone: separated optimization areas, obvious action buttons, one-click entry points, proof, and a clean product presentation, adapted into Liiiraa's own identity.

The reference direction is not about cloning brand assets. Hone's public product flow highlights the patterns worth adapting: a direct gaming-performance promise, a visible app UI screenshot, one-click optimization framing, supported game proof, feature sections, benchmarks, backup/pro settings language, testimonials, FAQ, and download CTAs. The supplied videos also point at Hone's install/game-mode polish, so implementation should review those clips during visual QA.

Current constraints:

- Desktop is React/Vite/Tauri with shared UI helpers in `packages/ui`.
- The existing desktop visual direction is graphite performance-console, but polish still needs stronger spacing, hierarchy, and component finish.
- Web is static/Vite-style HTML/CSS under `apps/web`.
- Safety, rollback, and honest benchmark language remain product requirements.
- Existing OpenSpec changes for command-center and optimizer UI are complete or nearly complete; this change is the dedicated premium polish pass layered on top.

## Goals / Non-Goals

**Goals:**

- Make the product feel polished at first glance across desktop and web.
- Adapt the useful structure of Hone's optimizer UX: separated categories, clear CTAs, game/system flows, proof, and simple install/download messaging.
- Create a precise action grammar for buttons, icon buttons, segmented controls, tabs, toggles, destructive actions, and disabled/locked states.
- Reduce visual clutter by limiting each viewport to a small number of primary decisions and moving detail into ledgers, inspectors, drawers, or secondary sections.
- Improve the desktop optimizer flows so users can scan, choose a category, apply one-click recommendations, inspect advanced changes, and rollback without guessing.
- Improve the landing/product page so it shows the real app, supported use cases, measurable proof, trust, and download intent clearly.
- Add visual QA gates with screenshots, responsive checks, text-fit checks, and reviewer checklists.

**Non-Goals:**

- Copy Hone branding, logos, exact copy, screenshots, benchmark claims, icons, or proprietary assets.
- Add new optimizer engine behavior, privileged Windows operations, cloud accounts, payments, or telemetry services.
- Replace the whole desktop architecture or introduce a large design framework.
- Make invented performance claims when benchmark data is unavailable.

## Decisions

1. Adapt Hone's product logic, not its identity.

   The implementation should be allowed to mirror the level of separation and clarity from Hone: simple hero, strong app preview, one-click CTA, separated optimizer modules, game mode framing, backup/pro/advanced areas, benchmark proof, and FAQ/trust sections. The final look must remain Liiiraa-native through its logo, typography, color tokens, screenshots, iconography, copy, and product-specific optimization categories.

   Alternative considered: avoid close reference influence and only "clean up" current screens. That would likely preserve the exact problem the user is reacting to: a UI that is technically complete but not obviously premium.

2. Use a strict button and action grammar.

   Every screen should expose one primary action, a small number of secondary actions, and clear icon/tool actions. Button variants should cover primary apply/download, secondary customize/review, ghost navigation, icon-only tools, destructive rollback/reset, disabled/locked, loading, and success states. Primary buttons should be visually consistent in size, radius, hover, focus, icon placement, loading feedback, and label length.

   Alternative considered: tune each button locally. That increases polish in isolated areas but makes the product feel inconsistent route-to-route.

3. Separate optimizer content into category lanes.

   Desktop optimizer UX should group work into distinct lanes such as Game Mode, System, Network, GPU, Power, Startup/Services, Benchmarks, Rollback, and Settings. Each lane needs a compact summary, status, risk/trust signal, primary action, and detail entry point. Advanced controls should be accessible without competing with the one-click path.

   Alternative considered: keep one broad tweak ledger as the main structure. Ledgers are still useful for audit details, but they should not carry the entire first impression.

4. Treat proof and trust as UI primitives.

   Benchmark deltas, supported games, rollback availability, signed update state, backup state, and source/confidence should become repeatable visual patterns. The landing page can show proof modules only when claims are honest and sourced from product data, seeded demo data, or clearly labeled examples.

   Alternative considered: keep proof inside copy blocks. That makes the product feel less concrete and less similar to polished optimizer competitors.

5. Split the work into desktop polish, web polish, and QA polish.

   Desktop should prove the optimizer workflow. Web should prove the product promise and conversion flow. QA should catch the common failure modes: clutter, overlap, inconsistent buttons, no first-screen product signal, and claims without supporting data.

   Alternative considered: update only the desktop because optimizer use happens there. The landing page is still part of the first impression and should share the same polish standard.

6. Prefer local tokens and CSS over adding a heavy visual framework.

   Existing tokens and CSS are close enough to support a high-quality finish. Add dependencies only if they solve a specific hard problem such as charting or animation without bloating the product.

   Alternative considered: adopt a full component framework. That could speed up consistency, but it risks making the product look generic and fighting the existing desktop-specific layout.

## Risks / Trade-offs

- [Risk] Close reference adaptation could look derivative. -> Mitigation: maintain a reference adaptation checklist that bans copied assets, exact copy, exact layout measurements, and unsupported claims while allowing structure and UX lessons.
- [Risk] Separating many optimizer categories could create more clutter. -> Mitigation: use compact category lanes, progressive disclosure, one primary action per view, and drawers/inspectors for detail.
- [Risk] Polished proof sections could overstate performance. -> Mitigation: require real benchmark metadata, clearly labeled demo data, or no numeric claim.
- [Risk] Button consistency work can become a large component refactor. -> Mitigation: start with shared primitives for the most visible controls, then migrate route-by-route.
- [Risk] Visual QA can slow delivery. -> Mitigation: use screenshot artifacts and structural assertions instead of fragile pixel-perfect testing.
- [Risk] Desktop and web could drift apart visually. -> Mitigation: source shared color, radius, spacing, icon, and button rules from `packages/ui` where practical.

## Migration Plan

1. Capture reference notes from Hone pages/videos and current Liiiraa screenshots for desktop and web.
2. Add or refine shared visual tokens for button sizes, action variants, category lanes, proof modules, cards, panels, density, and motion.
3. Build shared component primitives for buttons, icon buttons, segmented controls, category lane cards, proof tiles, benchmark deltas, trust badges, drawers, and tooltips.
4. Polish desktop Dashboard and Optimize first, because they define the optimizer feel and action grammar.
5. Polish remaining desktop routes around the same category, proof, and rollback patterns.
6. Polish the web landing/product page around a real product screenshot, clean first viewport, supported games/optimization sections, honest proof, testimonials/trust, FAQ, and download CTAs.
7. Add screenshots and Playwright checks for desktop and web responsive states.
8. Run check/type/test commands and review screenshot artifacts before marking the change complete.

## Open Questions

- Should the public landing page lead with Portuguese copy immediately, or remain English until a full localization copy pass?
- Should the desktop category lanes include a locked/premium visual state now, or only support it once product packaging is defined?
- Should benchmark proof use current mock data with explicit labels, or hide numeric claims until real benchmark capture is wired?
