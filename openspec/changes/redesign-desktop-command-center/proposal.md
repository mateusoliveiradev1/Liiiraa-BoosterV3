## Why

The current desktop app can launch through Tauri, but it loads the static web landing experience instead of the planned optimizer command center. This blocks meaningful product testing because users cannot scan, review tweak plans, apply safe changes, inspect risk, or rollback from the desktop UI.

## What Changes

- Replace the temporary static landing surface inside Tauri with a dedicated desktop command-center UI.
- Introduce a desktop-first visual system for dense optimizer workflows: left rail, status strip, diagnostic workspace, contextual inspector, and apply/rollback action bar.
- Add interactive UI flows for scan, optimization planning, safe/competitive/lab tweak review, apply/verify progress, benchmark prompts, and rollback state.
- Add dedicated surfaces for Power, NVIDIA, PUBG, Benchmarks, Rollback, Settings, privacy, updates, and "Signed by Liiiraa" trust.
- Integrate the desktop UI with existing Rust IPC where available and use typed mock/state adapters where engine work is not yet connected.
- Add screenshot and smoke verification so the desktop cannot regress into a marketing page or empty shell.
- No breaking changes to the existing API or tweak engine contracts are intended.

## Capabilities

### New Capabilities

- `desktop-command-center-visual`: Defines the desktop command-center visual, interaction, and state requirements needed for a usable optimizer UI.

### Modified Capabilities

- None.

## Impact

- Affected code: `apps/desktop/src/*`, `apps/desktop/src-tauri/*`, `packages/ui/*`, and desktop test/screenshot tooling.
- Affected runtime: Tauri desktop development must load a real desktop app surface instead of the static web landing page.
- Affected verification: desktop smoke checks must validate visual layout, navigation, tweak plan states, rollback state, and responsive text behavior.
- Dependencies may include Vite, React, TypeScript, lucide icons, and lightweight chart/rendering utilities if they are not already present.
