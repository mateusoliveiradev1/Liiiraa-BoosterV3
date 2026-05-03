# Optimizer UI/UX Overhaul Audit

Date: 2026-05-03

This folder contains the setup audit for tasks 1.1 through 1.4 of
`overhaul-optimizer-ui-ux`.

## Baseline Screenshots

Captured from the current desktop Vite app at `http://127.0.0.1:5174` with a
`1280x800` viewport and `deviceScaleFactor: 1`.

| Route | Hash | Screenshot |
| --- | --- | --- |
| Dashboard | `#dashboard` | `baseline-screenshots/1280x800/dashboard.png` |
| Scan | `#scan` | `baseline-screenshots/1280x800/scan.png` |
| Optimize | `#optimize` | `baseline-screenshots/1280x800/optimize.png` |
| Power | `#power` | `baseline-screenshots/1280x800/power.png` |
| NVIDIA | `#nvidia` | `baseline-screenshots/1280x800/nvidia.png` |
| PUBG | `#pubg` | `baseline-screenshots/1280x800/pubg.png` |
| Benchmarks | `#benchmarks` | `baseline-screenshots/1280x800/benchmarks.png` |
| Rollback | `#rollback` | `baseline-screenshots/1280x800/rollback.png` |
| Settings | `#settings` | `baseline-screenshots/1280x800/settings.png` |

## Audit Files

- `ui-string-inventory.md` inventories current hardcoded visible desktop copy
  candidates in `apps/desktop/src/**` and `packages/ui/src/**`.
- `design-token-inventory.md` inventories ad hoc visual values that should move
  into the design-token layer.
- `locale-decision.md` documents the first implementation pass locale default
  and fallback order.
