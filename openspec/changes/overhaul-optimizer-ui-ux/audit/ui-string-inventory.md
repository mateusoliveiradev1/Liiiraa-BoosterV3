# Hardcoded UI String Inventory

Date: 2026-05-03

Scope:

- `apps/desktop/src/**`
- `packages/ui/src/**`

Method: TypeScript AST scan of JSX text, visible JSX attributes
(`aria-label`, `title`, `alt`, `placeholder`), and object/string literal fields
commonly used for visible route data (`label`, `title`, `detail`, `summary`,
`value`, `state`, `eyebrow`, `tooltip`, `reason`, and related workflow fields).
The scan intentionally excludes imports, most class names, ids, data
attributes, and test assertion strings where detectable. Some candidates still
need human review because route fixtures and smoke HTML mix UI copy with
diagnostic/test metadata.

## Summary

| Area | Candidate count | Notes |
| --- | ---: | --- |
| `apps/desktop/src/**` | 458 | Shell copy, route copy, adapter fixtures, workflow components, benchmark charts, settings/trust surfaces. |
| `packages/ui/src/**` | 453 | Shared optimizer workflow fixture/render HTML, primitive labels/tooltips, settings trust copy. |
| Total | 911 | Use as migration backlog for locale keys. |

## Hotspots

| File | Candidates | Migration priority |
| --- | ---: | --- |
| `packages/ui/src/optimizationWorkflow.js` | 327 | High. Shared optimizer fixture copy, validation messages, and smoke HTML duplicate route terminology. Move reusable optimizer copy and glossary terms first. |
| `apps/desktop/src/components/OptimizationWorkflow.tsx` | 207 | High. Most Dashboard, Scan, Optimize, Rollback, Power, NVIDIA, PUBG, and shared workflow labels are rendered here. |
| `apps/desktop/src/adapters/desktopState.ts` | 84 | High. Navigation summaries, status strip values, route inspector text, adapter labels, and action labels are fixture-backed visible copy. |
| `apps/desktop/src/routes/BenchmarksRoute.tsx` | 67 | High. Benchmark labels, chart labels, table headings, privacy gate copy, and `toLocaleString("en-US")` usage should be locale-ready. |
| `packages/ui/src/primitives.ts` | 65 | Medium. Primitive labels, risk badge defaults, tooltips, and smoke fixtures should use glossary-backed keys. |
| `packages/ui/src/settingsTrust.js` | 61 | High. Trust, update, privacy, and local-data copy must align with Settings and status surfaces. |
| `apps/desktop/src/components/settings/SettingsTrustSurfaces.tsx` | 43 | High. Settings headings, button labels, section titles, aria labels, and subtitles are directly rendered. |
| `apps/desktop/src/commandCenter.ts` | 27 | Medium. Older command-center state still contains status and workflow text that should not drift from redesigned routes. |
| `apps/desktop/src/privacyConsent.ts` | 13 | Medium. Consent labels and blocked-upload messages should share privacy glossary keys. |
| `apps/desktop/src/crashReporting.ts` | 8 | Low for UI, but error messages and redaction demo strings should be separated from visible UI copy. |
| `apps/desktop/src/App.tsx` | 4 | High for shell accessibility: brand, nav, and runtime status labels. |
| `apps/desktop/src/catalogFetcher.ts` | 3 | Low. Mostly endpoints/cache keys; review before migrating. |
| `apps/desktop/src/routes/ScanRoute.tsx` | 2 | Medium. Action bar labels are visible. |

## Key Migration Groups

### Shell And Navigation

- `apps/desktop/src/App.tsx`: brand label, primary nav label, runtime status label.
- `apps/desktop/src/routes/index.tsx`: route labels for Dashboard, Scan, Optimize,
  Power, NVIDIA, PUBG, Benchmarks, Rollback, Settings.
- `apps/desktop/src/adapters/desktopState.ts`: navigation summaries and status
  strip labels/details.

Target keys:

- `shell.brand.name`
- `shell.navigation.dashboard.label`
- `shell.navigation.<route>.summary`
- `shell.status.<item>.label`
- `shell.status.<item>.detail`

### Optimizer Workflow Copy

- `apps/desktop/src/components/OptimizationWorkflow.tsx`: Dashboard hero/status,
  Scan scope/progress, Optimize group headings and ledger labels, rollback
  availability, power/NVIDIA/PUBG sections, action bars, badge text, risk labels,
  consent text, reboot text.
- `packages/ui/src/optimizationWorkflow.js`: shared optimizer fixture copy and
  smoke-rendered HTML.

Target keys:

- `workflow.dashboard.*`
- `workflow.scan.*`
- `workflow.optimize.*`
- `workflow.applyTimeline.*`
- `workflow.rollback.*`
- `workflow.power.*`
- `workflow.nvidia.*`
- `workflow.pubg.*`

### Benchmarks

- `apps/desktop/src/routes/BenchmarksRoute.tsx`: chart titles, legends, table
  headings, metric labels, result labels, privacy gate copy, row ranges, and
  hardcoded `en-US` number formatting.

Target keys:

- `benchmarks.metric.averageFps`
- `benchmarks.metric.onePercentLow`
- `benchmarks.metric.zeroPointOnePercentLow`
- `benchmarks.metric.p95FrameTime`
- `benchmarks.chart.<name>`
- `benchmarks.table.<column>`

### Settings, Trust, Privacy

- `apps/desktop/src/components/settings/SettingsTrustSurfaces.tsx`
- `packages/ui/src/settingsTrust.js`
- `apps/desktop/src/privacyConsent.ts`

Target keys:

- `settings.privacy.*`
- `settings.update.*`
- `settings.trust.*`
- `settings.localData.*`
- `settings.advancedGates.*`
- `privacyConsent.<kind>.*`

### Shared Glossary Terms

These should be centralized before route migration so the same concepts render
consistently across Dashboard, Scan, Optimize, Benchmarks, Rollback, and
Settings:

- Scan
- Apply
- Rollback
- Benchmark
- Risk
- Reboot
- Confidence
- Source
- Safe
- Competitive
- Lab
- Blocked
- Local first
- Signed by Liiiraa

## First Pass Rule

For redesigned surfaces, new visible copy should not be added directly inside
route components. Add or reuse typed locale keys, with `en-US` as the canonical
source text during the transition and `pt-BR`/`es-ES` placeholders allowed until
the translation pass is complete.
