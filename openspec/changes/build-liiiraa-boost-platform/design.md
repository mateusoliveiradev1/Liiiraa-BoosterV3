# Design

## Architecture
```text
apps/desktop
  React + Vite + Tauri UI
        |
        | typed IPC via tauri-specta
        v
crates/optimizer-core
  pure tweak planning, policy, validation, risk model
        |
        +--> crates/windows-api
        |      registry, powercfg, services, scheduled tasks, adapters
        |
        +--> crates/cpu
        |      CPU topology, Intel/AMD capability detection, PPM policy, guardrails
        |
        +--> crates/gpu
        |      vendor-neutral GPU detection, VRR/ReBAR/SAM capability model
        |
        +--> crates/nvidia
        |      NVAPI/NPI profile backup, import, apply, verify
        |
        +--> crates/amd
        |      AMD Adrenalin feature planning, SAM/Anti-Lag/HYPR-RX guardrails
        |
        +--> crates/intel-gpu
        |      Intel GPU detection, driver guidance, PresentMon integration
        |
        +--> crates/pubg
        |      PUBG discovery, settings inspection, safe profile recommendations
        |
        +--> crates/benchmark
               PresentMon/FrameView-style capture, parsing, scoring

apps/api
  Fastify + tRPC + Drizzle
        |
        v
packages/db -> Neon Postgres
```

The desktop app never receives Neon credentials. It talks to the cloud API. The cloud API is the only service that connects to Neon.

## Monorepo
Use `pnpm workspaces` and `Turborepo`.

```text
apps/
  desktop/          Tauri 2 + React + Vite app
  api/              Fastify + tRPC backend
  web/              future landing page and marketing site
  workers/          async jobs, release checks, telemetry processing

packages/
  db/               Drizzle schema, migrations, seeds
  api-contract/     tRPC routers/types and shared Zod contracts
  ui/               shared design system and primitives
  validators/       Zod schemas and domain value objects
  config/           eslint, tsconfig, prettier, turbo presets
  logger/           structured logging helpers
  test-utils/       fixtures and test builders

crates/
  optimizer-core/   pure domain engine
  cpu/              CPU topology, Intel APO/DTT detection, AMD chipset/X3D checks
  gpu/              vendor-neutral GPU capability and display model
  windows-agent/    elevated service/process boundary
  windows-api/      Windows implementation adapters
  nvidia/           NVIDIA profile implementation
  amd/              AMD GPU profile planner and safe Adrenalin guidance
  intel-gpu/        Intel graphics detection and recommendations
  pubg/             PUBG-specific implementation
  benchmark/        benchmark capture and scoring
```

## Development Discipline
The repository uses Conventional Commits and micro-step delivery.

Default implementation loop:
```text
choose one micro-step
  -> implement
  -> run smallest relevant check
  -> conventional signed commit
  -> push active branch
  -> continue
```

Main branch protection should require signed commits, status checks, linear history, no force pushes, and no branch deletion. GitHub Actions should use least-privilege permissions, avoid unsafe shell interpolation, and pin third-party actions by SHA in release-sensitive workflows.

The implementation source of truth for micro-steps is `tasks.md`, with `implementation-roadmap.md` as supporting order guidance. The universal completion rule is `definition-of-done.md`: every task must be verified, committed, and pushed when credentials are available. V1 tweak scope is locked by `v1-tweak-matrix.md`.

## Stack Decisions
- Desktop shell: Tauri 2 for smaller app size, Rust access, secure permissions, updater, and signed desktop releases.
- UI: React + TypeScript + Vite for fast DX and mature component ecosystem.
- Routing: TanStack Router for type-safe routing and search params.
- Server state: TanStack Query with tRPC client.
- Styling: Tailwind CSS v4 with custom tokens and shadcn/ui primitives.
- Cloud API: Fastify + tRPC for end-to-end TypeScript inference.
- Database: Neon Postgres with Drizzle ORM and migration files in `packages/db`.
- Type safety: Zod at boundaries, Drizzle types for DB, tRPC for API, tauri-specta for Rust-to-TypeScript IPC.
- Tests: Vitest, cargo test, Playwright, contract tests, migration tests, and Windows integration tests.

## Runtime Boundaries
```text
User mode UI
  |
  | request optimization plan
  v
Optimizer core
  |
  | needs elevation?
  +---- no ----> apply via normal Tauri command
  |
  +---- yes ---> windows-agent elevated boundary
                    |
                    v
                 backup -> apply -> verify -> audit log
```

The UI must never call raw shell snippets directly. It calls typed commands. Commands delegate to domain services. Domain services call platform adapters.

## Tweak Engine Contract
Each tweak is a data-backed module with a stable ID.

The full tweak authoring source of truth is `tweak-definition-standard.md`. The V1 tweak inventory, mode, and blocked guardrail list are defined in `v1-tweak-matrix.md`; implementation must not add an optimization outside that matrix without updating research and the matrix first.

Required fields:
- `id`
- `title`
- `category`
- `mode`: `safe`, `competitive`, or `lab`
- `risk`: `low`, `medium`, `high`, or `critical`
- `requiresAdmin`
- `requiresReboot`
- `supportsDryRun`
- `sourceLinks`
- `antiCheatNotes`
- `detection`
- `precheck`
- `plan`
- `backup`
- `apply`
- `verify`
- `rollback`
- `do`
- `dont`

Required states:
```text
unknown -> detected -> planned -> backed_up -> applied -> verified
                         |                         |
                         v                         v
                       failed <---------------- rollback_required
```

## Optimization Modes
- Safe: low-risk, broadly reversible, suitable for default "Optimize PC" flow.
- Competitive: performance-priority changes with security, power, heat, or compatibility tradeoffs. Must show warnings.
- Lab: experimental or hardware/driver-sensitive changes. Must require explicit user opt-in and restore point/backup.

## Default Safety Policy
Default runs must:
- Create a plan first.
- Show what will change.
- Back up all previous values.
- Apply only safe tweaks unless the user chooses a higher mode.
- Preserve anti-cheat compatibility.
- Preserve system security unless the user explicitly chooses a tradeoff.
- Include a single rollback path.

Default runs must not:
- Disable Defender globally.
- Disable Windows Update globally.
- Disable UAC.
- Disable pagefile.
- Rename or replace system files.
- Disable driver signature enforcement, test-signing protections, kernel debugging protections, or anti-cheat dependencies.
- Modify PUBG game binaries, memory, or BattlEye files.

## NVIDIA Profile Design
Two profile layers are required:
- Global profile: `Liiiraa Boost - Global Performance`
- Game profile: `Liiiraa Boost - PUBG Competitive`

The app must prefer official NVIDIA Driver Settings API/NVAPI where practical. NVIDIA Profile Inspector `.nip` import/export is allowed as a compatibility path, not as an unvalidated tweak dump.

Every NVIDIA action must:
- Detect NVIDIA GPU and driver version.
- Back up current customized profiles before changes.
- Avoid applying while PUBG/BattlEye is running.
- Apply global settings conservatively.
- Apply PUBG settings per executable where possible.
- Verify settings after apply.
- Provide rollback to previous profile values.

## Power Plan Design
Create named plans:
- `Liiiraa Boost - Balanced`
- `Liiiraa Boost - Performance`
- `Liiiraa Boost - Competitive`

The engine must duplicate an existing Windows power scheme, rename it, adjust settings, store the previous active scheme, and restore it on rollback.

Desktop and laptop rules must differ. Laptop users need battery/heat warnings and a less aggressive default.

## Benchmarking
The app must measure "felt performance", not only average FPS.

Metrics:
- average FPS
- 1% low FPS
- 0.1% low FPS
- frametime p50/p95/p99
- dropped/delayed frames
- CPU busy vs GPU busy when available
- GPU temperature, utilization, clocks, power when available
- network jitter/ping for game sessions where available

Before/after comparisons must include run metadata:
- game
- map/session label if user provides it
- driver version
- Windows build
- active power plan
- optimization profile version
- timestamp

## Cloud and Neon
Neon stores durable product data:
- devices
- app releases
- tweak catalog versions
- benchmark sessions
- anonymized optimization results when user consents
- audit trails
- feature flags
- future user/license data

Local SQLite stores operational recovery data:
- local tweak backups
- rollback snapshots
- applied plan history
- offline benchmark captures
- pending sync queue

## Landing Page
The landing page is part of this change as a planned app under `apps/web`, but it ships after the product shell is stable.

The page must sell the real product:
- first viewport shows the Liiiraa Boost product clearly
- no fake gradient-only hero
- use real app screenshots or generated product visuals
- include before/after benchmark proof
- explain full PC optimization plus game/PUBG focus
- include pricing/auth placeholders for a future change

## Visual Identity
The visual source of truth is `visual-design.md`.

The app should feel like a premium Windows performance instrument signed by Liiiraa: strong, sharp, data-heavy, and trustworthy. It should avoid generic AI dashboard patterns, decorative gradient orbs, cartoon gaming visuals, and empty marketing cards.

Required visual assets:
- custom logo
- app icon
- tray icon
- favicon
- social preview
- product screenshots/mockups for the landing page

The UI should use a command-center layout with left rail navigation, top status strip, main diagnostic workspace, contextual inspector, and persistent rollback/action visibility during optimization flows.

## Security and Integrity
- All privileged actions require typed commands and policy checks.
- Every risky tweak requires backup and rollback.
- Release artifacts must be signed.
- Auto-updater must verify signatures.
- No secrets in desktop builds.
- Cloud API validates all input with Zod.
- Database migrations are reviewed and tested against Neon branches.

## Security Model
Threats to design against:
- compromised or buggy frontend code calling privileged commands
- malicious update bundles
- leaked cloud/database secrets
- command injection through shell arguments
- privilege escalation through the elevated agent
- unsafe remote content in the desktop webview
- telemetry collecting more data than the user consented to share
- anti-cheat distrust caused by suspicious kernel, driver, or game-memory behavior

Security controls:
- Tauri capabilities are minimal and window-specific.
- Tauri CSP blocks remote scripts and unnecessary remote content.
- Rust commands validate all inputs before touching Windows state.
- Elevated actions go through a narrow allowlist and audited request format.
- Shell execution is avoided where a structured Windows API exists.
- Any required shell call uses fixed binaries, structured arguments, timeouts, and no string-built command concatenation.
- Local secrets use OS-protected storage such as DPAPI-backed credential storage.
- Telemetry is opt-in, redacted, and exportable.
- Update bundles require Tauri updater signatures and Windows code signing.
- API security follows OWASP ASVS and OWASP API Security Top 10 as baseline checklists.

The deeper security checklist is `security-max-plan.md`.

## App Performance Model
The optimizer must not become the user's new background problem.

Performance budgets:
- cold dashboard usable target: under 2.5 seconds on a mid-range Windows gaming PC
- idle CPU target: under 1 percent after startup tasks settle
- idle memory target: under 250 MB for the UI process where practical
- scan UI responsiveness: no blocking main thread over 100 ms
- benchmark capture overhead: visible and measured; capture tooling must not run outside requested sessions
- cloud sync: batched, cancellable, retry-limited, and never required for local rollback

Implementation rules:
- scans run in background jobs with progress, cancellation, and concurrency limits
- expensive WMI/registry/service reads are batched and cached
- UI lists use virtualization when large
- graphs downsample data before rendering
- React state updates are batched and never driven by raw high-frequency benchmark streams
- Rust tasks expose progress events instead of blocking the UI
- startup avoids loading NVIDIA, benchmark, and game modules until needed

The deeper performance checklist is `performance-max-plan.md`.

## Testing Strategy
- TDD for optimizer-core: every tweak starts with tests for detection, planning, policy, and rollback metadata.
- Windows adapters get integration tests behind feature flags and on a Windows runner.
- UI gets Vitest + browser/component tests.
- Desktop flows get Playwright where possible.
- API gets tRPC contract tests and DB migration tests.
- Each unsafe tweak gets negative tests proving it is blocked from Safe mode.
- Security tests cover IPC permission denials, unsafe command rejection, secret scanning, updater metadata validation, and dangerous tweak blocking.
- Performance tests cover startup smoke timing, scan cancellation, large history rendering, and benchmark parsing throughput.
- Visual tests cover dashboard, optimization plan, NVIDIA profile, PUBG profile, benchmark comparison, rollback timeline, and landing hero across desktop/mobile widths.

## Open Questions
- Which brand name should be final: `Liiiraa Boost`, `Liiiraa Booster`, or another public name?
- Should cloud sync be required at MVP or optional after local mode?
- Should the first MVP include payments/auth or stay offline-first with a trial bypass?
- Which exact NVIDIA settings IDs should be locked after NVAPI/NPI spike confirms driver compatibility?
