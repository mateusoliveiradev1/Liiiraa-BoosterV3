# @liiiraa/ui

Shared design tokens and primitive contracts for Liiiraa Booster.

## Files

- `tokens/liiiraa.tokens.json` is the implementation source for color, type, spacing, radius, shadow, motion, and component tokens.
- `styles/theme.css` exposes the tokens as Tailwind v4 `@theme` variables and app-level CSS custom properties.
- `styles/tailwind.css` imports Tailwind and the Liiiraa theme for app entry points.
- `src/tokens.ts` exports the token object and token key types for TypeScript consumers.
- `src/primitives.ts` exports framework-neutral definitions for action buttons, icon buttons, segmented controls, tabs,
  toggles, drawers, cards, category lanes, proof tiles, trust badges, benchmark deltas, and state badges.

## Premium Visual Rules

- Use one primary action per decision area. Secondary, ghost, rollback, destructive, locked, loading, success, and disabled
  states should use shared button variants instead of local color choices.
- Dense optimizer details belong in ledgers, drawers, inspectors, expandable rows, or secondary cards; the first screen
  should stay focused on category state and the next action.
- Category lanes, proof tiles, trust badges, and benchmark deltas should use shared component tokens so desktop and web
  keep compatible spacing, radii, shadows, and state colors.

## Verification

Run the token snapshot check:

```sh
pnpm --filter @liiiraa/ui test
```
