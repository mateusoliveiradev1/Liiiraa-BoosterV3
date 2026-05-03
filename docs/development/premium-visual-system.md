# Premium Visual System

This pass adapts the polish level of premium gaming optimizer references into
Liiiraa Booster without copying their identity. Use the references for product
logic, clarity, action hierarchy, and section rhythm; do not copy assets, exact
copy, screenshots, benchmark claims, testimonials, layout measurements, or brand
patterns.

## Shared Token Rule

Visible colors, surface hierarchy, spacing rhythm, radii, shadows, motion,
action sizing, badges, proof modules, category lanes, and drawer dimensions
start in `packages/ui/tokens/liiiraa.tokens.json`. Desktop maps those tokens in
`apps/desktop/src/designTokens.ts`; the static web landing consumes the checked
`apps/web/theme.generated.css` bridge before `apps/web/styles.css`.

Local CSS values are acceptable only when they describe layout mechanics such as
grid tracks, viewport constraints, or content-specific chart proportions. New
visual styling values should be promoted into the shared token contract first.

## Action Grammar

Each decision area should have one dominant primary action. Secondary actions
support review, customize, details, or waitlist/download alternatives. Ghost
actions are for low-emphasis navigation and compact toolbars. Destructive and
rollback actions use separate variants and must not sit visually inside the same
cluster as normal apply actions without confirmation context.

Buttons and icon buttons must cover hover, focus, loading, success, locked,
disabled, destructive, and rollback states. Icon-only controls need tooltips and
accessible labels. Labels should truncate or wrap within stable control
dimensions rather than resizing the layout.

## Surface Grammar

Use category lanes for optimizer groups such as Game Mode, System, Network,
GPU, Power, Startup/Services, Benchmarks, Rollback, and Settings. A lane needs a
title, short summary, status, trust or risk signal, one primary action, and a
detail entry point.

Use proof tiles for benchmark or trust evidence only when data is real, sourced,
or clearly labeled as example, preview, or coming soon. Benchmark deltas should
show metric names and context instead of broad absolute claims.

Use drawers, ledgers, inspectors, tabs, or expandable rows for dense tweak
details: before/after values, source, confidence, risk, reboot requirement, and
rollback capability.

## Reference Adaptation Guard

- Keep Liiiraa colors, logo, product screenshots, iconography, and copy.
- Preserve a direct product-first first viewport and clear one-click path.
- Separate proof, game support, safety, rollback, and advanced controls.
- Label demo proof as example or preview.
- Exclude external logos, screenshots, proprietary assets, exact phrases,
  exact section order, testimonials, and benchmark numbers.
- Review desktop and web screenshots for overlapping text, clipped controls,
  unclear primary action hierarchy, and unsupported claims before accepting a
  polish pass.
