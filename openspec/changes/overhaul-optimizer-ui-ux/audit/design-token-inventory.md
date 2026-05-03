# Design Token Inventory

Date: 2026-05-03

Scope:

- `apps/desktop/src/**`
- `packages/ui/src/**`

The repository already has a shared token source at
`packages/ui/tokens/liiiraa.tokens.json`, exposed by
`packages/ui/src/tokens.ts`. The current desktop CSS still defines its own
`:root` variables and many direct values, so the next tasks should map the
desktop app onto a single token boundary instead of continuing parallel token
systems.

## Automated Scan Summary

| Value type | Total matches | Unique values | Main files |
| --- | ---: | ---: | --- |
| Hex colors | 144 | 53 | `apps/desktop/src/styles.css`, `packages/ui/src/settingsTrust.js`, `packages/ui/src/optimizationWorkflow.js`, `apps/desktop/src/routes/BenchmarksRoute.tsx`, SVG assets |
| `rgb()` / `rgba()` colors | 76 | 59 | Mostly `apps/desktop/src/styles.css` |
| Gradients | 16 | 16 | All in `apps/desktop/src/styles.css` |
| Shadows/glows | 27 | 24 | Mostly `apps/desktop/src/styles.css` |
| Radius values | 39 | 17 | `styles.css`, inline React styles, shared primitives/smoke HTML |
| Spacing/layout values | 391 | 223 | `styles.css`, `BenchmarksRoute.tsx`, `OptimizationWorkflow.tsx`, shared smoke HTML |
| Breakpoints/media queries | 6 | 6 | `styles.css`, shared smoke HTML |

## Values To Move Into Tokens

### Desktop Root Palette

Current values live in `apps/desktop/src/styles.css:4-24`:

- Backgrounds: `#010204`, `#030507`, `#04070b`
- Surfaces: `#070b11`, `#0b1119`, `#101923`
- Borders: `#142230`, `#243a50`
- Text: `#f7fbff`, `#c3d2df`, `#8295a8`
- Accents/status: `#13d8ff`, `#35ff8f`, `#ffb13d`, `#ff4d6a`, `#b08cff`, `#94a6b8`

These overlap conceptually with `packages/ui/tokens/liiiraa.tokens.json` but do
not share the same names or exact values. The desktop pass should define the
chosen graphite/steel/status palette once and consume it through CSS variables.

### Neon And Radial Backgrounds

The main ad hoc glow treatments are in `apps/desktop/src/styles.css`:

- Body radial cyan/green washes at lines 42-44.
- Shell cyan/green linear washes at lines 56-58.
- Sidebar cyan wash and cyan border at lines 72-76.
- Active navigation cyan/green gradient and glow at lines 139-143.
- Dashboard hero cyan/green gradients at lines 294-298.
- Inspector green wash at lines 597-600.

These should be replaced with restrained graphite surfaces and semantic accent
strokes, not large decorative page washes.

### Shadows And Glow

Current shadows mix panel depth with colored neon glow:

- `--shadow-panel` in `styles.css:24`.
- Brand drop shadow and text shadow in `styles.css:91` and `styles.css:99`.
- Focus ring in `styles.css:134`.
- Active nav, icon, status, metric, hero, button, marker, and chart glow values
  across `styles.css`.

Move to token categories:

- `shadow.panel`
- `shadow.focus`
- `shadow.inset`
- `shadow.accent.active`
- `shadow.accent.success`
- `shadow.chartLine`

### Radii

Current repeated values:

- `8px`
- `999px`
- `50%`
- `0.2rem`
- `var(--radius-card)`

Map to:

- `radius.sm`
- `radius.md`
- `radius.card`
- `radius.pill`
- `radius.round`

Keep cards at 8px or less unless the design system changes that explicitly.

### Layout, Density, And Breakpoints

Values that should become named density/layout tokens:

- Rail widths: `minmax(13.5rem, 15rem)`, `5rem`, `2.85rem`, `1.95rem`.
- Status strip: `3.35rem`, `repeat(4, minmax(8.5rem, 1fr))`, status item
  columns and padding.
- Workspace: `76rem` max content width and `1.35rem` page padding.
- Dashboard hero and grid tracks, especially the first-viewport layout.
- Metric tile min heights and grid tracks.
- Table minimum widths such as `32rem` and benchmark table widths such as
  `43rem`.
- Breakpoints: `1180px`, `760px`, plus smoke HTML breakpoints `900px`,
  `860px`, and `640px`.

Target buckets:

- `layout.rail.width`
- `layout.rail.compactWidth`
- `layout.content.maxWidth`
- `density.pagePadding`
- `density.controlHeight`
- `density.statusStripHeight`
- `breakpoint.desktopCompact`
- `breakpoint.mobile`

### Chart And Benchmark Styling

`apps/desktop/src/routes/BenchmarksRoute.tsx` defines a local `toneAccent`
palette and direct SVG colors:

- `#27d7ff`, `#ff5a67`, `#9b7cff`, `#9aa8b8`, `#3af28f`, `#ffbd5a`
- Grid stroke `#344252`
- Axis label `#9aa8b8`
- Inline chart/table spacing and bar track values.

Move these to chart/status tokens and reuse the same color mapping as metric
tiles, benchmark proof, and risk badges.

### Shared Smoke HTML

`packages/ui/src/optimizationWorkflow.js` and `packages/ui/src/settingsTrust.js`
embed smoke-render CSS with duplicated colors, spacing, borders, radius, and
shadow values. These are not the desktop runtime CSS, but they should stay in
sync with token choices or snapshot tests will drift from the product UI.

## Values That Can Remain Local

- SVG path geometry and intrinsic logo dimensions can remain in assets, though
  brand colors should reference the final brand token values when practical.
- Data-derived chart coordinates and row calculations should remain computed.
- Test-only redaction sample strings in crash-reporting tests do not need
  visual tokens.

## Tokenization Priority

1. Establish desktop token names and CSS variables for background, surface,
   border, text, status/risk/mode, chart, spacing, radius, shadow, motion, and
   density.
2. Map `apps/desktop/src/styles.css` root variables to the shared token source.
3. Replace decorative radial/neon washes with graphite surfaces and semantic
   accent strokes.
4. Replace local `toneAccent` maps and inline visual values in React components.
5. Update shared smoke HTML to consume the same token values or generated CSS
   variable names.
