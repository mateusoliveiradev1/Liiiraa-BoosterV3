## Context

The repository already contains Tauri desktop Rust code and React/TypeScript desktop UI source under `apps/desktop/src`, but the runnable Tauri configuration currently points `devUrl` at the static web landing page on port 5173. The result is a valid desktop shell that does not expose the optimizer command center, tweak plan review, scan flow, or rollback surfaces described in the broader platform plan.

The redesign must turn the desktop into the primary product surface while preserving the security model: local-first desktop UI, narrow Tauri capabilities, typed IPC for Rust commands, no Neon credentials in desktop artifacts, and no hidden privileged writes.

## Goals / Non-Goals

**Goals:**

- Make the desktop app launch into a dedicated optimizer command center instead of the static landing page.
- Establish a desktop-first layout: left rail navigation, top status strip, diagnostic workspace, contextual inspector, and bottom apply/rollback bar where relevant.
- Render realistic scan, plan, apply, verify, benchmark, and rollback states even where engine integration needs temporary typed adapters.
- Keep Safe, Competitive, Lab, and Blocked tweak groups visually and behaviorally distinct.
- Provide screenshot/smoke coverage for desktop layout, text fit, navigation, and workflow state.

**Non-Goals:**

- Implement every Windows, GPU, PUBG, benchmark, or storage tweak engine as part of this visual redesign.
- Ship authentication, billing, cloud account management, or Neon access from the desktop app.
- Replace the public landing page; the web landing can keep its own static or future marketing surface.
- Add broad Tauri filesystem, shell, process, or arbitrary HTTP permissions.

## Decisions

1. Build a real desktop frontend package for `apps/desktop`.

   The desktop UI should own its own Vite/React entry, package scripts, dependencies, and build output. Tauri `devUrl` should point at the desktop dev server and `frontendDist` should point at the desktop build output. The static web app should remain separate.

   Alternative considered: keep Tauri pointed at `apps/web`. This keeps launch simple but makes the desktop indistinguishable from the landing page and prevents desktop-specific state, IPC, and navigation.

2. Use a typed desktop state adapter layer.

   UI routes should consume a small adapter boundary that can call Tauri IPC where available and return typed local mock data where the engine is not connected yet. The adapter names should match product concepts: scan status, plan buckets, tweak details, rollback sessions, benchmark summary, trust status, and update state.

   Alternative considered: hard-code route data directly in components. That is faster for a mockup but makes later IPC integration more expensive and hides missing engine contracts.

3. Keep the visual system dense and tool-like.

   The command center should use compact panels, thin separators, tabular metrics, icons with tooltips, risk badges, timelines, and charts. It should avoid landing-page hero composition, nested cards, decorative copy, and empty dashboard blocks.

   Alternative considered: polish the current landing layout. That improves first impressions but does not create the product workflows needed to test optimization behavior.

4. Treat visual QA as a required gate.

   Desktop work should include Playwright or equivalent browser-level screenshots against the Vite app, plus a Tauri launch smoke where practical. Checks should validate that text does not overflow, routes navigate, and workflow states are visible at the target desktop sizes.

   Alternative considered: rely only on typecheck and manual inspection. That misses the exact problem that triggered this change: the app technically runs but shows the wrong surface.

## Risks / Trade-offs

- [Risk] The desktop UI can drift from real engine capability if mock adapters become permanent. -> Mitigation: name adapters after real contracts, mark mock-only data clearly in code, and add tasks to replace each adapter with IPC/API integration.
- [Risk] Adding a desktop package can overlap with existing workspace scripts. -> Mitigation: keep package scope `@liiiraa/desktop`, wire it through pnpm workspace, and update Tauri config intentionally.
- [Risk] Dense UI can become visually noisy. -> Mitigation: use the existing visual design rules: compact but structured panels, no nested cards, no decorative blobs, and clear risk segmentation.
- [Risk] Screenshot checks may require browser dependencies on a fresh machine. -> Mitigation: keep a lightweight static/Vite smoke as the minimum and document any heavier Playwright dependency.

## Migration Plan

1. Add or complete `apps/desktop/package.json`, Vite config, TypeScript config, and desktop scripts.
2. Point Tauri development and build output at the desktop frontend instead of `apps/web`.
3. Move current desktop React source into a runnable command-center app and remove any reliance on the web landing for desktop launch.
4. Implement route-level visual surfaces and typed state adapters.
5. Add screenshot/smoke verification and update root scripts as needed.
6. Keep the static web landing available at its own URL and package.

## Open Questions

- Should the first implementation use only local mock data, or should `run_read_only_system_scan` be wired into the Scan/Dashboard route immediately?
- Should the desktop dev server keep port 5173, or should web and desktop use separate fixed ports to avoid ambiguity?
- Which chart library, if any, should be adopted for benchmark and frametime visuals?
