## 1. Desktop Runtime Wiring

- [x] 1.1 Add a runnable `@liiiraa/desktop` frontend package with Vite, React, TypeScript, and dev/build/typecheck scripts.
- [x] 1.2 Point Tauri development and build config at the desktop frontend instead of the static web landing page.
- [x] 1.3 Ensure the desktop app launches from one documented command with web/API dependencies clear.

## 2. Command Center Foundation

- [x] 2.1 Build the desktop shell layout with left rail navigation, top status strip, main workspace, and route switching.
- [x] 2.2 Add shared desktop state adapters for scan, plan, rollback, benchmark, trust, update, and hardware status.
- [x] 2.3 Replace placeholder route content with dense optimizer-first panels on every primary route.

## 3. Tweak Workflow Surfaces

- [x] 3.1 Implement Dashboard and Scan surfaces showing readiness, scan scope, progress, findings, and next action.
- [x] 3.2 Implement Optimize surface with Safe, Competitive, Lab, and Blocked tweak sections.
- [x] 3.3 Implement apply/verify/benchmark/rollback workflow states with backup, failure, reboot, and rollback visibility.

## 4. Hardware, Game, And Trust Routes

- [x] 4.1 Implement Power and NVIDIA surfaces with plan/profile state, backup state, warnings, and rollback affordances.
- [x] 4.2 Implement PUBG surface with install/config detection, BattlEye safety, DX benchmark choice, checklist, and NVIDIA link.
- [x] 4.3 Implement Benchmarks surface with FPS lows, frametime, metadata, variance warnings, and comparison visuals.
- [x] 4.4 Implement Settings and trust surfaces for privacy, telemetry, update channel, signed update status, and local data controls.

## 5. Visual QA And Verification

- [x] 5.1 Add desktop visual smoke coverage for navigation, status strip, tweak plan sections, rollback state, and settings/trust.
- [x] 5.2 Add screenshot verification for desktop viewports and check for overflow, overlap, and missing optimizer state.
- [x] 5.3 Run `pnpm check` and a Tauri desktop launch smoke, then document any remaining mock-only adapters.
