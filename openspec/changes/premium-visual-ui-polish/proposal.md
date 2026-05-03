## Why

The product now has optimizer surfaces, but the visual execution still does not feel as polished, separated, or immediately convincing as best-in-class gaming optimizer products such as Hone. This change exists to raise Liiiraa Booster to a premium visual and UX standard before the remaining UI work is treated as finished.

## What Changes

- Establish a Hone-caliber, Liiiraa-native visual direction for desktop and web: sharp dark surfaces, disciplined accent use, stronger product screenshots, clean spacing, fewer competing blocks, and more intentional CTAs.
- Rework optimizer UX around clearly separated optimization groups, one-click entry points, per-category controls, advanced/pro sections, backup/rollback context, and direct buttons that make the next action obvious.
- Replace crowded mixed-purpose panels with deliberate sections: scan state, game/system optimization, network, GPU, power, startup/services, benchmark proof, restore points, and settings.
- Polish the component system for buttons, segmented controls, tabs, toggles, cards, ledgers, drawers, tooltips, empty states, loading states, disabled states, and danger/rollback confirmation states.
- Update the landing/product presentation so the first viewport shows the actual product and value clearly, with proof sections for games, benchmarks, supported optimizations, trust, testimonials, FAQ, and download CTAs.
- Add visual QA gates so implementation cannot be accepted while layouts look cluttered, text overlaps, buttons feel inconsistent, primary actions are unclear, or the product still reads as generic.
- Keep the identity Liiiraa-specific: references can influence structure, rhythm, polish, and interaction quality, but no Hone assets, logos, exact copy, benchmark claims, or brand identity should be imported.
- No breaking changes to optimizer engine behavior, Tauri IPC contracts, API contracts, or safety/rollback guarantees are intended.

## Capabilities

### New Capabilities

- `premium-visual-system`: Defines the polished Liiiraa visual language, layout rhythm, component styling, motion, density, icon usage, and cross-surface finish level.
- `segmented-optimizer-workflows`: Defines optimizer UX patterns for separated optimization categories, one-click flows, per-category controls, advanced details, proof, backup, and rollback.
- `product-presentation-experience`: Defines the polished web/product presentation patterns for the landing page, feature sections, benchmark proof, game support, trust, and CTAs.
- `visual-quality-assurance`: Defines visual review, screenshot, responsiveness, text-fit, contrast, polish, and reference-adaptation gates.

### Modified Capabilities

- None.

## Impact

- Affected code: `apps/desktop/src/**`, `apps/desktop/tests/**`, `apps/web/**`, `apps/web/tests/**`, `packages/ui/src/**`, `packages/ui/styles/**`, `packages/ui/tokens/**`, and shared visual assets in `packages/ui/assets/**`.
- Affected design docs/specs: desktop visual design, web landing guidance, UI token guidance, optimizer workflow documentation, screenshot baselines, and QA acceptance checklists.
- Affected tests: desktop and web Playwright visual smoke checks, responsive layout checks, locale/text expansion checks, component-state checks, and screenshot artifact review.
- Potential dependencies: prefer existing React, CSS, lucide icons, and local token infrastructure; add new visual, animation, or chart dependencies only when they clearly improve polish without adding heavy runtime or maintenance cost.
- Runtime constraints: preserve local-first desktop behavior, clear rollback affordances, no hidden privileged writes, no unsafe optimize-all behavior, and no dishonest benchmark/testimonial claims.
