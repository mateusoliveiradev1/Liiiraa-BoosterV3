# @liiiraa/ui

Shared design tokens for Liiiraa Booster.

## Files

- `tokens/liiiraa.tokens.json` is the implementation source for color, type, spacing, radius, shadow, motion, and component tokens.
- `styles/theme.css` exposes the tokens as Tailwind v4 `@theme` variables and app-level CSS custom properties.
- `styles/tailwind.css` imports Tailwind and the Liiiraa theme for app entry points.
- `src/tokens.ts` exports the token object and token key types for TypeScript consumers.

## Verification

Run the token snapshot check:

```sh
pnpm --filter @liiiraa/ui test
```
