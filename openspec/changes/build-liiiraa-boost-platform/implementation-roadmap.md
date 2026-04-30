# Implementation Roadmap

This roadmap is supporting guidance. The executable task source of truth is `tasks.md`, where every task has an ID, context pack, write scope, verification, and commit/push requirement.

Do not execute this roadmap directly in a fresh chat. Start from a `tasks.md` task ID such as `T041`, then use this roadmap only for phase orientation. If this roadmap and `tasks.md` disagree, follow `tasks.md` and update OpenSpec docs in a dedicated governance task before relying on the changed guidance.

This roadmap is intentionally micro-stepped so an AI agent can implement, verify, commit, and push without drifting.

Each row means:
1. implement only that step
2. run the listed verification or the closest available check
3. commit with the suggested Conventional Commit shape
4. push the branch when credentials are available

## Phase 0: Repository and Workflow

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 0.1 | initialize Git remote and protected repo docs | `git remote -v` | `chore(repo): configure repository remote` |
| 0.2 | add `.gitignore` and secret-safe baseline | `git status --ignored --short` | `chore(repo): add secret-safe gitignore` |
| 0.3 | scaffold pnpm workspace | `pnpm -v` and workspace install | `build(repo): initialize pnpm workspace` |
| 0.4 | add Turborepo pipeline | `pnpm turbo --version` | `build(repo): add turborepo pipeline` |
| 0.5 | add shared lint/type configs | `pnpm check` | `build(config): add shared project tooling` |
| 0.6 | add commitlint/Husky/lint-staged | invalid commit-msg test | `build(repo): enforce conventional commits` |
| 0.7 | add contributing and PR templates | markdown lint/check | `docs(repo): document contribution workflow` |
| 0.8 | add CI skeleton | workflow syntax check | `ci(repo): add baseline quality workflow` |

## Phase 1: Security and Release Foundations

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 1.1 | add security policy and threat model docs | doc review | `docs(security): define project threat model` |
| 1.2 | add GitHub Actions least-privilege permissions | CI dry run | `security(ci): restrict workflow permissions` |
| 1.3 | add dependency/secret scanning configs | scanner dry run | `security(repo): add supply chain scanning` |
| 1.4 | add Tauri updater design skeleton | config validation | `docs(release): define signed updater flow` |
| 1.5 | add release checklist | checklist review | `docs(release): add signed release checklist` |

## Phase 2: Desktop Shell and Visual Identity

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 2.1 | scaffold Tauri + React + Vite desktop app | desktop dev server starts | `feat(desktop): scaffold tauri app shell` |
| 2.2 | add route tree and app shell layout | typecheck | `feat(desktop): add command center navigation` |
| 2.3 | add visual tokens | visual smoke | `feat(ui): add liiiraa design tokens` |
| 2.4 | add logo/icon placeholder assets | build asset check | `feat(ui): add brand asset placeholders` |
| 2.5 | add dashboard static shell | screenshot review | `feat(desktop): add performance dashboard shell` |
| 2.6 | add accessibility and visual tests | Playwright smoke | `test(ui): add visual accessibility smoke tests` |

## Phase 3: Optimizer Domain Core

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 3.1 | create Rust workspace | `cargo test` | `build(rust): initialize optimizer workspace` |
| 3.2 | add tweak domain types | `cargo test -p optimizer-core` | `feat(optimizer): define tweak contract` |
| 3.3 | add Safe/Competitive/Lab policy | unit tests | `feat(optimizer): enforce optimization modes` |
| 3.4 | add dry-run planner | unit tests | `feat(optimizer): add dry run planning` |
| 3.5 | add backup/rollback abstractions | unit tests | `feat(optimizer): add rollback model` |
| 3.6 | add blocked action guardrails | negative tests | `security(optimizer): block dangerous actions` |

## Phase 4: Windows System Optimization

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 4.1 | add read-only scan adapters | mocked tests | `feat(windows): add system scan adapters` |
| 4.2 | add power plan read/create plan | Windows integration or mock | `feat(windows): add liiiraa power plan planner` |
| 4.3 | add Game DVR planner | tests | `feat(windows): add game capture optimization` |
| 4.4 | add startup/background scanner | tests | `feat(windows): add startup app scan` |
| 4.5 | add NTFS metadata tweaks | tests | `feat(windows): add ntfs optimization tweaks` |
| 4.6 | add network power-saving tweaks | tests | `feat(windows): add network power tuning` |
| 4.7 | add rollback recovery flow | failure tests | `test(windows): cover rollback recovery` |

## Phase 5: NVIDIA and PUBG

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 5.1 | NVAPI/NPI spike doc and adapter choice | spike result doc | `docs(nvidia): record profile integration decision` |
| 5.2 | NVIDIA detection | tests | `feat(nvidia): detect driver and gpu state` |
| 5.3 | NVIDIA profile backup/readback | tests/spike | `feat(nvidia): add profile backup flow` |
| 5.4 | global profile planner | tests | `feat(nvidia): plan global performance profile` |
| 5.5 | PUBG profile planner | tests | `feat(nvidia): plan pubg competitive profile` |
| 5.6 | PUBG install detection | tests | `feat(pubg): detect installation paths` |
| 5.7 | PUBG config recommendations | tests | `feat(pubg): add competitive recommendations` |
| 5.8 | anti-cheat safety denials | negative tests | `security(pubg): enforce anticheat boundaries` |

## Phase 6: Benchmarking

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 6.1 | benchmark session model | tests | `feat(benchmark): define session model` |
| 6.2 | PresentMon parser | parser fixtures | `feat(benchmark): parse presentmon captures` |
| 6.3 | before/after scoring | unit tests | `feat(benchmark): add comparison scoring` |
| 6.4 | chart downsampling | perf test | `perf(benchmark): downsample large captures` |
| 6.5 | UI benchmark screen | screenshot test | `feat(desktop): add benchmark comparison view` |

## Phase 7: Cloud and Persistence

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 7.1 | scaffold API app | API smoke | `feat(api): scaffold typed backend` |
| 7.2 | add Drizzle/Neon schema | migration check | `feat(db): add initial product schema` |
| 7.3 | add tRPC routers | contract tests | `feat(api): add typed product routers` |
| 7.4 | add local SQLite storage | tests | `feat(desktop): add local recovery storage` |
| 7.5 | add sync queue | offline tests | `feat(desktop): add offline sync queue` |

## Phase 8: Update System and Release

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 8.1 | configure updater plugin | config check | `feat(release): configure tauri updater` |
| 8.2 | configure updater signing docs/env | dry-run build | `security(release): document updater key handling` |
| 8.3 | add update manifest schema | tests | `feat(release): add update manifest contract` |
| 8.4 | add beta/stable channels | tests | `feat(release): add release channel model` |
| 8.5 | add rollback/kill switch | tests | `security(release): add catalog rollback controls` |

## Phase 9: Landing Page

| Step | Scope | Verification | Commit |
| --- | --- | --- | --- |
| 9.1 | scaffold web app | build | `feat(web): scaffold landing app` |
| 9.2 | add product hero | screenshot | `feat(web): add product first hero` |
| 9.3 | add proof sections | screenshot | `feat(web): add benchmark proof sections` |
| 9.4 | add trust/signed section | screenshot | `feat(web): add signed by liiiraa trust section` |
| 9.5 | add responsive visual QA | Playwright | `test(web): add landing visual checks` |
