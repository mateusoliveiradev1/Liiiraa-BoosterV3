# Tasks

This file is the executable source of truth for clean-context implementation. Each row defines the full contract for one task ID: what to read, what may be edited, how to verify, and which Conventional Commit message to use before push.

The user can start a new chat with one task ID, for example: `execute T041`. Unless the user explicitly asks for a phase or range, agents must execute exactly one task ID and then stop.

Use [chat-execution-guide.md](chat-execution-guide.md) for copy/paste prompts when starting a new implementation chat.

## Non-Negotiable Execution Rules
- `tasks.md` is authoritative for implementation sequencing. Supporting docs such as `implementation-roadmap.md` provide orientation only and must not expand a task's scope.
- One task ID per chat unless the user explicitly asks to continue.
- Read the listed context files before editing.
- Only edit the listed write scope. If a needed dependency sits outside that scope, stop and report the blocker instead of editing around it.
- Run the smallest relevant verification before commit.
- Treat the `Verify` and `Commit and Push` columns as required task work, not follow-up work.
- Commit and push after every task ID. Never save commits for the end.
- Use Conventional Commits.
- Use signed commits when local signing is configured.
- If push/auth/signing is blocked, report the blocker and exact command attempted.
- Do not mark a task done without verification status, commit hash, and push status.
- Every tweak task must obey `tweak-definition-standard.md` and `v1-tweak-matrix.md`.
- Every UI task must obey `visual-design.md` and `ui-ux-spec.md`.
- Every security task must obey `security-max-plan.md`.
- Every performance task must obey `performance-max-plan.md`.
- Every tweak implementation must also obey `tweak-hardening-review.md`.

## Per-Task Done Template
Each task completion note must include:
- `Verification: <command/result>`
- `Commit: <hash or blocked reason>`
- `Push: <remote/branch or blocked reason>`
- `Files changed: <list>`

## Required Commit/Push Pattern
For every task ID:
```text
git status --short
<run checks>
git add <task files>
git commit -S -m "<type(scope): message>"   # use -S only when signing works
git push origin <active-branch>
```

If signing is not configured yet, complete T001 before normal implementation commits.

## 0. Repo Governance

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T000 | Lock OpenSpec handoff rules and make this task map the source of truth. | `tasks.md`, `definition-of-done.md`, `implementation-roadmap.md`, `chat-execution-guide.md` | OpenSpec docs only | `rg -n "Commit and Push|v1-tweak-matrix|one task" openspec/changes/build-liiiraa-boost-platform` | `docs(openspec): lock task execution protocol`, then push |
| T001 | Configure Git identity, commit signing guide, branch naming, and local commit safety. | `development-workflow.md`, `security-max-plan.md` | `.gitignore`, `CONTRIBUTING.md`, `.github/*`, package config as needed | `git config --list --show-origin`, commitlint dry run | `chore(repo): enforce signed conventional workflow`, then push |
| T002 | Add commitlint, Husky, lint-staged, and Conventional Commit validation. | `development-workflow.md` | root package files, `.husky/`, commitlint config | `pnpm commitlint --from HEAD~1 --to HEAD` or documented dry run | `chore(repo): add conventional commit gates`, then push |
| T003 | Add CI skeleton with least-privilege permissions and pinned action policy comments. | `security-max-plan.md`, `development-workflow.md` | `.github/workflows/*` | YAML validation plus `rg -n "permissions:" .github/workflows` | `ci(repo): add guarded quality workflow`, then push |
| T004 | Scaffold pnpm workspace and Turborepo pipeline. | `design.md`, `specs/platform-infrastructure/spec.md` | `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `packages/config/*` | `pnpm install --lockfile-only`, `pnpm turbo --dry=json` | `chore(workspace): scaffold pnpm turbo monorepo`, then push |
| T005 | Scaffold Rust workspace crates and shared quality config. | `design.md`, `specs/platform-infrastructure/spec.md` | `Cargo.toml`, `crates/*` | `cargo metadata`, `cargo test --workspace` | `chore(rust): scaffold optimizer workspace`, then push |
| T006 | Add env examples and no-secret guardrails. | `specs/cloud-backend-persistence/spec.md`, `security-max-plan.md` | `.env.example`, app/package env examples, docs | secret scan command or documented local fallback | `chore(config): add safe environment templates`, then push |

## 1. Product Design and Visual Identity

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T010 | Convert visual design into implementable tokens. | `visual-design.md`, `ui-ux-spec.md` | `packages/ui`, Tailwind/theme files | token unit snapshot or style build | `feat(ui): add liiiraa design tokens`, then push |
| T011 | Create logo, app icon, tray icon, installer icon, favicon, and social preview placeholders/finals. | `visual-design.md`, `specs/visual-identity/spec.md` | `apps/desktop/src/assets`, `apps/web/public`, `packages/ui/assets` | asset dimensions script or manual checklist | `feat(brand): add liiiraa booster identity assets`, then push |
| T012 | Build reusable UI primitives with accessibility states. | `visual-design.md`, `ui-ux-spec.md` | `packages/ui/src/*` | Story/test render plus a11y smoke if available | `feat(ui): add accessible app primitives`, then push |
| T013 | Build desktop app shell and command-center layout. | `ui-ux-spec.md`, `specs/desktop-optimizer-app/spec.md` | `apps/desktop/src/*` | `pnpm --filter desktop typecheck` | `feat(desktop): add command center shell`, then push |
| T014 | Build dashboard, scan, optimize, and rollback views. | `ui-ux-spec.md`, `v1-tweak-matrix.md` | `apps/desktop/src/routes/*`, `apps/desktop/src/components/*`, `packages/ui/src/*` | Playwright smoke screenshot | `feat(desktop): add optimization workflow screens`, then push |
| T015 | Build NVIDIA, PUBG, Power, and Benchmark views. | `ui-ux-spec.md`, `specs/nvidia-profile-optimization/spec.md`, `specs/pubg-game-optimization/spec.md`, `v1-tweak-matrix.md` | `apps/desktop/src/routes/*`, `apps/desktop/src/components/*`, `packages/ui/src/*` | Playwright smoke screenshot | `feat(desktop): add gaming optimization surfaces`, then push |
| T016 | Build settings, privacy, update, and "Signed by Liiiraa" trust surfaces. | `update-system.md`, `security-max-plan.md`, `visual-design.md` | `apps/desktop/src/routes/settings*`, `apps/desktop/src/components/settings/*`, `packages/ui/src/*` | UI test or screenshot | `feat(desktop): add trust and update settings`, then push |
| T017 | Build landing page shell with product-first hero and app visual. | `visual-design.md`, `specs/landing-page/spec.md` | `apps/web/*` | Playwright desktop/mobile screenshots | `feat(web): add liiiraa booster landing page`, then push |

## 2. Security and Privilege Boundary

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T020 | Write threat model for frontend compromise, privileged agent abuse, updater, API, telemetry, and anti-cheat trust. | `security-max-plan.md`, `research.md` | `docs/security/*` | `rg -n "frontend compromise|updater|anti-cheat" docs/security` | `docs(security): add product threat model`, then push |
| T021 | Configure minimal Tauri capabilities, CSP, and deny-by-default permissions. | `security-max-plan.md`, Tauri docs in `research.md` | `apps/desktop/src-tauri/*` | Tauri config validation/build | `feat(security): restrict tauri capabilities`, then push |
| T022 | Implement typed IPC validation and command allowlist skeleton. | `specs/app-security/spec.md`, `design.md` | `crates/optimizer-core`, `apps/desktop/src-tauri` | Rust unit tests | `feat(security): add typed ipc allowlist`, then push |
| T023 | Implement elevated agent boundary skeleton with audit logging. | `design.md`, `security-max-plan.md` | `crates/windows-agent`, `crates/windows-api`, local storage | Rust tests for unknown command denial | `feat(agent): add privileged command boundary`, then push |
| T024 | Add local data protection for backups/tokens where applicable. | `security-max-plan.md`, `specs/app-security/spec.md`, `specs/desktop-optimizer-app/spec.md` | `crates/optimizer-core`, `crates/windows-agent`, `packages/local-store` or equivalent local persistence crate/package | unit test for protected storage adapter | `feat(security): protect local sensitive data`, then push |
| T025 | Add API baseline controls: validation, CORS, rate limit, error redaction, least-privilege logging. | `security-max-plan.md`, `specs/cloud-backend-persistence/spec.md` | `apps/api`, `packages/api-contract` | API unit tests | `feat(api): add baseline security controls`, then push |
| T026 | Add privacy consent gates for telemetry, crash reports, and benchmark sync. | `security-max-plan.md`, `specs/benchmarking-and-telemetry/spec.md`, `ui-ux-spec.md` | `apps/desktop/src/*`, `apps/api/src/*`, telemetry packages/modules | consent tests | `feat(privacy): add telemetry consent gates`, then push |

## 3. Core Tweak Engine

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T030 | Define domain types: `TweakDefinition`, plans, backups, results, rollback, risk/mode/category. | `tweak-definition-standard.md`, `v1-tweak-matrix.md` | `crates/optimizer-core` | `cargo test -p optimizer-core` | `feat(core): define tweak engine contracts`, then push |
| T031 | Implement registry/catalog loader with schema version and source validation. | `v1-tweak-matrix.md`, `update-system.md` | `crates/optimizer-core`, catalog fixtures | schema tests | `feat(core): load validated tweak catalog`, then push |
| T032 | Implement dry-run plan builder and dependency ordering. | `tweak-definition-standard.md` | `crates/optimizer-core` | plan ordering tests | `feat(core): add dry run planning`, then push |
| T033 | Implement backup and rollback interfaces. | `tweak-definition-standard.md`, `definition-of-done.md` | `crates/optimizer-core`, `crates/windows-api` | rollback fixture tests | `feat(core): add backup rollback contracts`, then push |
| T034 | Implement mode policy guardrails: Safe excludes Competitive/Lab; blocked actions denied. | `v1-tweak-matrix.md`, `security-max-plan.md` | `crates/optimizer-core` | policy denial tests | `feat(core): enforce tweak safety policy`, then push |
| T035 | Implement local SQLite model for snapshots, audit events, pending sync, and benchmark captures. | `specs/desktop-optimizer-app/spec.md`, `specs/benchmarking-and-telemetry/spec.md` | local persistence crate/package | migration/storage tests | `feat(storage): add local optimizer persistence`, then push |

## 4. Windows V1 Safe Optimizations

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T040 | Implement read-only system scan: OS, CPU, RAM, GPU, storage, network, services, startup, VBS/HVCI, active power plan. | `v1-tweak-matrix.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, desktop scan bindings | Rust tests with fixtures; manual scan dry run | `feat(windows): add read only system scan`, then push |
| T041 | Implement Liiiraa power plans with desktop/laptop rules and rollback. | `v1-tweak-matrix.md`, `research.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, `crates/optimizer-core` | mocked `powercfg` tests or Windows dry run | `feat(power): add liiiraa power plans`, then push |
| T042 | Implement Game DVR, capture, Game Bar overlay, and Focus Assist/notification safe controls. | `v1-tweak-matrix.md`, `research.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, `crates/optimizer-core`, desktop plan UI if needed | registry fixture tests | `feat(windows): add safe gaming capture controls`, then push |
| T043 | Implement startup/background app inspection and recommendation-only apply model. | `v1-tweak-matrix.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, `crates/optimizer-core`, desktop plan UI if needed | fixture tests | `feat(windows): add startup app optimizer`, then push |
| T044 | Implement Storage Sense/temp cleanup/trim/DirectStorage readiness checks. | `v1-tweak-matrix.md`, `research.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, `crates/optimizer-core`, storage-related UI if needed | fixture tests | `feat(storage): add safe cleanup readiness checks`, then push |
| T045 | Implement Defender performance-safe actions: schedule awareness and narrow exclusions only with warning. | `v1-tweak-matrix.md`, `research.md`, `security-max-plan.md` | `crates/windows-api`, `crates/optimizer-core`, security warning UI if needed | policy tests proving no global Defender disable | `feat(windows): add defender-safe performance options`, then push |
| T046 | Implement Windows Update/Delivery Optimization controls that avoid global disable. | `v1-tweak-matrix.md`, `research.md`, `security-max-plan.md` | `crates/windows-api`, `crates/optimizer-core`, update settings UI if needed | tests proving global disable is blocked | `feat(windows): add update bandwidth controls`, then push |
| T047 | Implement NIC power-saving and EEE/Green Ethernet detection with adapter-specific rollback. | `v1-tweak-matrix.md`, `research.md`, `specs/windows-system-optimization/spec.md` | `crates/windows-api`, `crates/optimizer-core`, network plan UI if needed | adapter property fixture tests | `feat(network): add adapter power optimizer`, then push |

## 5. Windows Competitive and Lab Tweaks

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T050 | Implement VBS/HVCI/VMP tradeoff detection and explicit-consent plan, with reboot and rollback. | `v1-tweak-matrix.md`, `security-max-plan.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core`, security warning UI if needed | tests proving not Safe/default | `feat(windows): add security tradeoff planner`, then push |
| T051 | Implement MMCSS and Win32PrioritySeparation as benchmarked Competitive tweaks. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core` | registry fixture tests | `feat(windows): add scheduler competitive tweaks`, then push |
| T052 | Implement HAGS, VRR, windowed optimizations, graphics preference detection/planning. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core`, graphics plan UI if needed | tests plus UI plan | `feat(windows): add graphics setting planner`, then push |
| T053 | Implement Search indexing and SysMain conditional planner only; no system binary renames. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core` | tests proving SearchApp rename blocked | `feat(windows): add conditional service planner`, then push |
| T054 | Implement NTFS last-access and 8.3 behavior with compatibility warnings. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core` | fsutil fixture tests | `feat(storage): add ntfs metadata tweaks`, then push |
| T055 | Implement advanced NIC settings as Lab/benchmark-only: RSS/RSC/offloads/interrupt moderation. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core` | tests proving defaults stay conservative | `feat(network): add lab network tuning planner`, then push |
| T056 | Implement Lab stubs for timer resolution and memory compression with no default apply. | `v1-tweak-matrix.md`, `research.md` | `crates/windows-api`, `crates/optimizer-core` | tests proving opt-in required | `feat(windows): add lab tweak guards`, then push |
| T057 | Implement CPU Intel/AMD platform planner: topology, throttling, Intel APO/DTT, AMD chipset/X3D, CPPC, PPM audit. | `v1-tweak-matrix.md`, `specs/cpu-platform-optimization/spec.md`, `research.md` | `crates/cpu`, `crates/windows-api`, `crates/optimizer-core` | CPU fixture tests proving no unsafe default | `feat(cpu): add intel amd platform planner`, then push |
| T058 | Implement CPU guardrail denials for E-core disable, SMT disable, CPU mitigation disable, realtime priority, hard affinity, and auto OC. | `v1-tweak-matrix.md`, `specs/cpu-platform-optimization/spec.md`, `security-max-plan.md` | `crates/cpu`, `crates/optimizer-core` | denial tests | `security(cpu): block unsafe cpu tweaks`, then push |

## 6. GPU Profiles

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T060 | Implement GPU/driver detection for NVIDIA, AMD, Intel. | `v1-tweak-matrix.md`, `research.md`, `specs/nvidia-profile-optimization/spec.md` | `crates/nvidia`, `crates/optimizer-core`, GPU vendor modules | fixture tests | `feat(gpu): add vendor driver detection`, then push |
| T061 | Implement NVIDIA profile backup/readback via NVAPI/Driver Settings API or validated NPI compatibility. | `specs/nvidia-profile-optimization/spec.md`, `v1-tweak-matrix.md` | `crates/nvidia` | profile backup tests | `feat(nvidia): add profile backup bridge`, then push |
| T062 | Implement `Liiiraa Boost - Global Performance` with conservative global settings. | `v1-tweak-matrix.md`, `specs/nvidia-profile-optimization/spec.md` | `crates/nvidia`, catalog fixtures | readback tests | `feat(nvidia): add global performance profile`, then push |
| T063 | Implement `Liiiraa Boost - PUBG Competitive` for `TslGame.exe`, including FPS cap, Reflex/LLM, G-SYNC/VRR, and ReBAR policy. | `v1-tweak-matrix.md`, `specs/nvidia-profile-optimization/spec.md`, `specs/pubg-game-optimization/spec.md` | `crates/nvidia`, `crates/pubg` | profile fixture tests | `feat(nvidia): add pubg competitive profile`, then push |
| T064 | Implement AMD profile planner for HYPR-RX, Anti-Lag/Anti-Lag 2 policy, Boost, Chill, FRTC, Enhanced Sync, FreeSync, AFMF, RIS/RSR, and SAM/ReBAR. | `v1-tweak-matrix.md`, `specs/amd-gpu-optimization/spec.md`, `research.md` | `crates/amd`, `crates/gpu`, `crates/optimizer-core`, desktop AMD UI if needed | capability tests | `feat(amd): add radeon profile planner`, then push |
| T065 | Implement Intel graphics/PresentMon-friendly detection and safe recommendations. | `v1-tweak-matrix.md`, `research.md` | `crates/intel-gpu`, `crates/gpu`, `crates/optimizer-core`, benchmark integration if needed | fixture tests | `feat(intel): add graphics recommendation planner`, then push |
| T066 | Implement GPU profile rollback UI and engine integration. | `specs/nvidia-profile-optimization/spec.md`, `ui-ux-spec.md`, `v1-tweak-matrix.md` | GPU modules/crates, `apps/desktop/src/*` | rollback tests/flow | `feat(gpu): add profile rollback flow`, then push |
| T067 | Implement vendor-neutral GPU platform checks: driver age, display refresh, VRR, ReBAR/SAM, frame-generation policy, shader cache state, and clean-driver recommendations. | `v1-tweak-matrix.md`, `specs/nvidia-profile-optimization/spec.md`, `specs/amd-gpu-optimization/spec.md` | `crates/gpu`, `crates/nvidia`, `crates/amd`, `crates/intel-gpu` | GPU capability fixture tests | `feat(gpu): add platform capability planner`, then push |

## 7. PUBG Optimization

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T070 | Detect PUBG install paths, `TslGame.exe`, configs, Steam/Epic metadata, and BattlEye presence. | `v1-tweak-matrix.md`, `specs/pubg-game-optimization/spec.md` | `crates/pubg` | fixture tests | `feat(pubg): detect installation and anticheat`, then push |
| T071 | Read PUBG config safely and snapshot before suggestions. | `specs/pubg-game-optimization/spec.md` | `crates/pubg`, local storage | parser tests | `feat(pubg): add safe config snapshot`, then push |
| T072 | Detect legacy launch options and recommend removal instead of forcing flags. | `v1-tweak-matrix.md` | `crates/pubg`, desktop UI | fixture tests | `feat(pubg): add launch option cleanup planner`, then push |
| T073 | Build DX11 vs DX11 Enhanced benchmark flow with no universal forced default. | `v1-tweak-matrix.md`, benchmark specs | `crates/pubg`, `crates/benchmark`, UI | E2E flow | `feat(pubg): add dx mode benchmark flow`, then push |
| T074 | Build PUBG performance/visibility checklist and NVIDIA/Windows cross-plan. | `ui-ux-spec.md`, `v1-tweak-matrix.md` | desktop PUBG route, catalog | UI smoke | `feat(pubg): add competitive settings checklist`, then push |
| T075 | Enforce anti-cheat guardrails: no game memory, no BE files, no kernel/test-signing/debug tweaks. | `research.md`, `security-max-plan.md`, `v1-tweak-matrix.md` | `crates/optimizer-core`, `crates/pubg` | denial tests | `feat(pubg): enforce anticheat safety rules`, then push |

## 8. Benchmarking and Telemetry

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T080 | Integrate PresentMon-compatible capture path and session lifecycle. | `specs/benchmarking-and-telemetry/spec.md`, `research.md` | `crates/benchmark` | capture parser tests | `feat(benchmark): add presentmon capture sessions`, then push |
| T081 | Parse metrics: FPS, 1%/0.1% lows, p50/p95/p99 frametime, dropped frames, CPU/GPU busy where available. | benchmark spec | `crates/benchmark` | CSV fixture tests | `feat(benchmark): parse frametime metrics`, then push |
| T082 | Implement before/after scoring with confidence/variance warnings. | `performance-max-plan.md` | `crates/benchmark`, UI summary | scoring tests | `feat(benchmark): add comparison scoring`, then push |
| T083 | Build benchmark UI charts with virtualization/downsampling. | `ui-ux-spec.md`, `performance-max-plan.md` | desktop benchmark route | Playwright screenshot/perf smoke | `feat(desktop): add benchmark result charts`, then push |
| T084 | Add cloud sync for benchmark sessions behind explicit consent. | `specs/cloud-backend-persistence/spec.md`, privacy docs | API, local pending sync | API and consent tests | `feat(telemetry): sync benchmark sessions with consent`, then push |

## 9. Cloud, Neon, and Catalog

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T090 | Scaffold Fastify + tRPC API with typed contracts. | `design.md`, `specs/cloud-backend-persistence/spec.md` | `apps/api`, `packages/api-contract` | API typecheck/tests | `feat(api): scaffold typed backend`, then push |
| T091 | Add Drizzle + Neon schema/migrations for devices, releases, tweak catalog, benchmarks, audit events, feature flags. | cloud spec | `packages/db`, migrations | migration check | `feat(db): add neon schema migrations`, then push |
| T092 | Implement signed/validated remote tweak catalog delivery without arbitrary scripts. | `update-system.md`, `v1-tweak-matrix.md` | API, catalog package, desktop fetcher | signature/integrity tests | `feat(catalog): add signed remote tweak catalog`, then push |
| T093 | Add release channel and feature flag APIs: dev, beta, stable. | `update-system.md` | API/db/contracts | API tests | `feat(api): add release channels and flags`, then push |
| T094 | Add auth-ready boundaries without shipping auth yet. | proposal/design | API contracts, docs | tests proving public/private separation | `chore(api): prepare auth boundaries`, then push |

## 10. Updates, Release, and Operations

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T100 | Configure Tauri updater for signed artifacts and channels. | `update-system.md`, release spec | Tauri config, updater module | build/config validation | `feat(updater): configure signed app updates`, then push |
| T101 | Add remote kill switch and bad catalog rollback strategy. | `update-system.md`, `specs/release-operations/spec.md` | `apps/api/src/*`, catalog package/module, desktop updater module | rollback tests | `feat(catalog): add remote rollback controls`, then push |
| T102 | Add release workflow with least privilege, signed tags, attestations, changelog, no-secret checks. | `development-workflow.md`, `security-max-plan.md` | `.github/workflows`, scripts | workflow lint/dry run | `ci(release): add signed release workflow`, then push |
| T103 | Document Windows code signing, installer signing, and SmartScreen reputation path. | release ops spec | `docs/release/*` | doc checklist | `docs(release): document windows signing path`, then push |
| T104 | Add crash/error reporting with opt-in and PII redaction. | `security-max-plan.md`, `specs/app-security/spec.md`, `specs/benchmarking-and-telemetry/spec.md` | `apps/desktop/src/*`, `apps/api/src/*`, telemetry packages/modules | redaction tests | `feat(ops): add privacy safe crash reporting`, then push |

## 11. Final Quality Gates

| ID | Chat Goal | Read First | Write Scope | Verify | Commit and Push |
| --- | --- | --- | --- | --- | --- |
| T110 | Add full `pnpm check` and `cargo check` orchestration. | testing spec | root scripts, CI | `pnpm check`, `cargo test --workspace` | `chore(test): add full quality command`, then push |
| T111 | Add Windows integration tests for registry/power/services using safe mocks and guarded live mode. | testing spec, tweak standard | `crates/windows-api/tests` | integration suite | `test(windows): add optimizer integration coverage`, then push |
| T112 | Add Playwright E2E for scan -> plan -> apply simulation -> rollback simulation. | UI specs | `apps/desktop/tests`, `apps/web/tests` | Playwright run | `test(e2e): add optimization workflow coverage`, then push |
| T113 | Add performance budgets and smoke measurements for startup, idle CPU, memory, scan time, UI responsiveness, benchmark overhead. | `performance-max-plan.md` | test scripts/docs | perf smoke result | `test(perf): add app performance budgets`, then push |
| T114 | Add documentation gate proving every tweak has source links, do/dont, backup, verify, rollback, risk, anti-cheat notes, applicability, conflicts, and side effects. | `tweak-definition-standard.md`, `v1-tweak-matrix.md`, `tweak-hardening-review.md` | validation script/tests | validation passes | `test(tweaks): enforce tweak documentation completeness`, then push |
| T115 | Run release-candidate dry run and freeze V1 scope. | all OpenSpec docs | docs/checklists only unless bug fixes | all checks plus release checklist | `chore(release): freeze v1 release candidate scope`, then push |
