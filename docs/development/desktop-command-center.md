# Desktop Command Center

The desktop app launches from the repository root with one command:

```powershell
pnpm desktop:dev
```

This runs the Tauri shell for `apps/desktop` and lets Tauri start the desktop
Vite frontend automatically on `http://127.0.0.1:5174`. The public web landing
page is separate and is not required for desktop development.

The current command-center surfaces use local desktop state, mock adapters, and
Tauri IPC where it already exists. The cloud API is not required to open the
desktop shell. When API-backed flows are connected, keep the API in a separate
process and point `VITE_PUBLIC_API_BASE_URL` in `apps/desktop/.env.local` at
that API; do not add Neon credentials or service secrets to desktop files.

Useful package-level commands:

```powershell
pnpm --filter @liiiraa/desktop dev
pnpm --filter @liiiraa/desktop build
pnpm --filter @liiiraa/desktop typecheck
pnpm --filter @liiiraa/desktop test
pnpm --filter @liiiraa/desktop test:visual
```

Use `pnpm desktop:build` from the repository root for a Tauri production build.

## Visual verification

The desktop package has Playwright smoke coverage for the real Vite command
center. The visual spec navigates Dashboard, Scan, Optimize, Power, NVIDIA,
PUBG, Benchmarks, Rollback, and Settings, then checks the runtime status strip,
route-specific optimizer state, Optimize plan buckets, benchmark proof,
rollback recovery state, and Settings trust surfaces. It also captures
screenshot artifacts for every redesigned route at `1280x800`, `1440x900`, and
`1024x680` desktop viewports while checking for blank primary surfaces,
horizontal overflow, clipped text, clipped primary actions, obvious text
overlap, and regressions back to marketing-hero first screens.

The same visual spec runs locale-fit passes for `pt-BR` and `es-ES` through the
desktop locale boundary. Those passes verify longer labels in navigation,
buttons, badges, the status strip, tweak tables, and compact controls before
capturing review screenshots.

## Performance-console tokens

Desktop runtime styling is anchored in `apps/desktop/src/designTokens.ts` and
mapped through `apps/desktop/src/styles.css`. New visible colors, accent
surfaces, chart strokes, shadows, radii, motion values, and density values
should be added to that token module first, then consumed through CSS variables
or the exported `desktopToneCssVars` / `desktopChartCssVars` helpers.

Run the lightweight token guard with:

```powershell
pnpm --filter @liiiraa/desktop check:visual-tokens
```

The guard rejects the retired radial page washes and old one-off neon
cyan/green/purple literals so the desktop app stays in the graphite
performance-console direction.

## Locale workflow

Desktop copy uses the typed optimizer locale catalogs in
`packages/ui/src/localization.ts`. The current default locale remains `en-US`,
with `pt-BR` and `es-ES` partial catalogs falling back to `en-US` for keys that
are not translated yet.

For manual browser checks, add a locale query parameter before the route hash:

```powershell
http://127.0.0.1:5174/?locale=pt-BR#optimize
http://127.0.0.1:5174/?locale=es-ES#scan
```

The selected locale is stored under `liiiraa.optimizer.locale` for the desktop
frontend session. New redesigned copy should be added through locale keys, and
compact controls must keep stable dimensions with wrapping, truncation, or
tooltips when Portuguese or Spanish labels expand.

## Mock-only adapter status

The desktop state boundary is intentionally typed, but the UI still uses
mock-only adapters for the route data in this visual pass:

| Adapter | Current source | Replacement target |
| --- | --- | --- |
| Scan | `typed-mock` | Hydrate from `run_read_only_system_scan` and scan-progress IPC. |
| Plan | `typed-mock` | Hydrate from the tweak planner once engine contracts are connected. |
| Rollback | `typed-mock` | Hydrate from backup and restore session storage. |
| Benchmark | `typed-mock` | Hydrate from local benchmark captures and comparison metadata. |
| Trust | `typed-mock` | Hydrate from persisted privacy consent and signed catalog state. |
| Update | `typed-mock` | Hydrate from Tauri updater configuration and signed update checks. |
| Hardware | `typed-mock` | Hydrate from normalized power, GPU, PUBG, and benchmark discovery. |
