# Build Liiiraa Boost Platform

## Summary
Build Liiiraa Boost as a production-grade desktop optimizer for Windows PCs, focused on full-system performance with dedicated gaming and PUBG optimization profiles. The product must be modular from day one: every optimization is a typed, reversible, auditable tweak with detection, backup, apply, verify, rollback, risk level, source links, and benchmark evidence where possible.

The platform includes the desktop app, Rust optimizer engine, cloud backend, persistent Neon Postgres data model, local recovery storage, test infrastructure, update pipeline, and a future landing page. Authentication and billing are intentionally deferred, but the architecture must reserve clean extension points for them.

## Problem
Most "FPS booster" apps are either dashboards, script bundles, or unsafe tweak packs. They often apply global registry changes without diagnosis, do not measure real improvement, do not separate safe/competitive/lab risk levels, and do not provide reliable rollback. The target user wants a PC that feels faster everywhere, with especially stable frametimes, lower stutter, lower latency, and better PUBG behavior.

## Goals
- Create a desktop optimizer that performs real Windows, GPU, power, service, network, and game-profile optimizations.
- Use a modern stack: Tauri 2, Rust, React, TypeScript, Vite, pnpm, Turborepo, tRPC, Fastify, Drizzle ORM, Neon Postgres, Tailwind v4, TanStack Router/Query, and shadcn/ui-based design system.
- Provide end-to-end type safety across UI, Tauri commands, cloud API, validation schemas, and database access.
- Make every tweak modular, testable, reversible, categorized, sourced, and benchmark-aware.
- Support full PC optimization plus game-specific profiles, with PUBG as the flagship profile.
- Include a strong, premium UI and a future landing page in the roadmap.
- Build with TDD, E2E tests, contract tests, Windows integration tests, and CI from the beginning.
- Treat app security, update integrity, least privilege, privacy, and app performance as first-class product requirements.
- Use Conventional Commits, signed commits, protected branches, secure CI, and micro-step pushes as the default development discipline.
- Establish a distinct Liiiraa visual identity with custom logo/icon assets, strong desktop art direction, and a dedicated visual design guide.

## Non-Goals
- Do not implement authentication, billing, affiliate tracking, or license enforcement in this change.
- Do not ship kernel drivers, cheats, game memory manipulation, game file tampering, or anti-cheat bypass behavior.
- Do not apply unsafe "optimize everything" bundles by default.
- Do not connect the desktop app directly to Neon with database credentials.
- Do not promise fixed FPS gains without measurement.

## Scope
In scope:
- Monorepo architecture and package boundaries.
- Desktop shell and optimizer workflow.
- Rust optimizer engine and elevated Windows service boundary.
- Tweak definition registry and first tweak catalog.
- NVIDIA global and PUBG profiles.
- Custom Liiiraa Boost power plans.
- PUBG detection, settings guidance, and safe profile application.
- Benchmarking and telemetry model using local captures and optional cloud sync.
- Neon-backed cloud API and persistent schema.
- Landing page plan and design requirements.
- Test strategy and implementation tasks.
- App threat model, secure Tauri permissions, elevated-agent security, supply-chain checks, signing, updater integrity, and privacy controls.
- App performance budgets, scan scheduling, UI responsiveness, resource limits, and benchmark overhead rules.
- Development workflow, commit rules, signed commits, branch protection, secure GitHub Actions, and release provenance.
- Visual identity, logo/icon requirements, design tokens, component style, motion, accessibility, and "Signed by Liiiraa" trust surfaces.

Out of scope for now:
- Auth providers, subscriptions, payments, license keys, admin dashboard, and affiliate program.
- Manual BIOS/overclock automation.
- Mobile apps.
- Mac/Linux optimization.

## Source Grounding
The proposal is grounded in research from:
- AtlasOS and ReviOS playbook patterns.
- XOS-related public repos and imribiy tweak collections, treated cautiously when not fully auditable.
- Microsoft Windows gaming/performance documentation.
- NVIDIA driver profile and latency documentation.
- PUBG official support/performance guidance.
- BattlEye anti-cheat compatibility notes.
- PresentMon/FrameView-style benchmark methodology.

## Success Criteria
- The app can run a pre-optimization scan and produce a typed plan without applying changes.
- The app can apply and rollback safe optimizations reliably.
- The app can create named Liiiraa Boost power plans and restore the previous active plan.
- The app can create/import/update NVIDIA profiles with backup and per-profile rollback.
- The app can detect PUBG, apply a PUBG-safe profile, and avoid anti-cheat-hostile behavior.
- The app can capture before/after metrics: average FPS, 1% low, 0.1% low, frametime percentiles, CPU/GPU bound indicators, and reboot-required state.
- The app can prove that privileged actions are allowlisted, audited, reversible, and inaccessible from arbitrary frontend code.
- The app meets defined startup, idle CPU, scan, memory, and UI responsiveness budgets.
- Release artifacts are signed, updater bundles are signature-verified, and desktop builds contain no cloud database secrets.
- Commits follow Conventional Commits and protected branches require signed commits/status checks.
- The app has a recognizable Liiiraa Booster visual identity, custom icons, and a non-generic command-center desktop experience.
- CI can run typecheck, unit tests, Rust tests, API contract tests, and Playwright flows.
- Specs are sufficiently divided so future AI agents can implement one module at a time without crossing boundaries.
