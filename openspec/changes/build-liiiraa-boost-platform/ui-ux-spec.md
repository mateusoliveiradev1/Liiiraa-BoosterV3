# UI/UX Spec

The UI must feel like a real performance workstation, not a generic admin dashboard.

## Primary User Jobs
- "Make my PC faster without breaking it."
- "Optimize for PUBG and competitive games."
- "Show me what changed and why."
- "Prove the result with benchmarks."
- "Undo everything if something feels wrong."

## App Navigation
```text
Dashboard
Scan
Optimize
Power
NVIDIA
Games
  PUBG
Benchmarks
Rollback
Settings
```

## Dashboard
Must show:
- system readiness score
- active optimization mode
- active power plan
- GPU driver state
- PUBG readiness
- last benchmark delta
- rollback availability
- update/signing trust state

Must not show:
- generic welcome hero
- marketing copy as primary content
- decorative cards with no action

## Scan Flow
Steps:
1. choose scan scope
2. run scan with progress and cancel
3. group findings by impact and risk
4. generate optimization plan

States:
- idle
- scanning
- partial result
- complete
- failed with retry
- cancelled

## Optimization Plan Flow
Plan is grouped by:
- Safe
- Competitive
- Lab
- Blocked/educational

Each tweak row shows:
- exact change
- expected impact
- risk
- rollback
- reboot
- source confidence
- why recommended

Actions:
- apply safe only
- include competitive
- inspect lab
- export plan
- cancel

## Apply Flow
The apply screen must behave like an installer with safety:
- backup
- apply
- verify
- benchmark prompt
- rollback if needed

The user must always be able to see:
- current step
- what is changing now
- what already changed
- what failed
- how to rollback

## NVIDIA Screen
Must show:
- GPU and driver version
- current global profile state
- Liiiraa global profile state
- PUBG profile state
- backup status
- warning if PUBG/BattlEye is running
- recommended refresh/FPS cap logic

## PUBG Screen
Must show:
- install detection
- config detection
- BattlEye safety status
- DX11 vs DX11 Enhanced benchmark choice
- launch options warning
- competitive checklist
- NVIDIA profile link
- benchmark CTA

## Rollback Screen
Must show:
- timeline of optimization sessions
- tweak-level rollback state
- restore all for a session
- exact before/after values where safe to show
- reboot required markers

## Settings
Must show:
- privacy and telemetry
- update channel
- signing/update trust
- local data export/delete
- "Signed by Liiiraa"
- advanced/lab feature gates

## UX Safety Rules
- No scary irreversible action without exact disclosure.
- No hidden "optimize all" that includes Competitive/Lab tweaks.
- No fake progress.
- No average-FPS-only success state.
- No benchmark comparison without metadata.
- No risk state shown by color alone.
- No icon-only action without accessible label and tooltip.

## Performance-Console Rules
- The first screen must show readiness, current bottleneck or next best action, scan/apply state, active mode, rollback availability, benchmark delta, hardware/game state, and signing/update trust.
- The shell uses graphite surfaces, steel borders, compact density, and semantic accents. Avoid large neon/radial backgrounds and oversized marketing panels.
- Use ledgers for tweak review, timelines for apply and rollback, inspectors for state detail, and charts/tables for benchmark proof.
- Primary actions must remain visible and unclipped at `1024x680`, `1280x800`, and `1440x900`.
- Every route must render a nonblank optimizer surface with route-specific state.

## Locale Workflow
- The default desktop locale is `en-US`.
- `pt-BR` and `es-ES` catalogs are present as partial catalogs and fall back to `en-US` for missing keys.
- New redesigned shell, shared primitive, action, badge, table, and compact-control copy must be added through typed locale keys.
- Manual checks can use `?locale=pt-BR#route` or `?locale=es-ES#route`.
- Portuguese and Spanish visual checks must cover navigation, buttons, badges, status strip items, tweak tables, and compact controls for expansion, wrapping, truncation, and tooltip behavior.

## Verification Gate
- Run `pnpm --filter @liiiraa/desktop typecheck`.
- Run `pnpm --filter @liiiraa/desktop check:visual-tokens`.
- Run `pnpm --filter @liiiraa/desktop test:visual`.
- Review screenshot artifacts for blank surfaces, overlap, clipped actions, color-only risk states, and marketing-hero regressions.
