# V1 Release Candidate Freeze Checklist

Task: `T115`
Change: `build-liiiraa-boost-platform`
Dry run timestamp: `2026-05-02T22:44:32-03:00`
Base branch: `main`
Base commit: `6ecaefa`
Dry-run tag used for metadata checks: `v0.0.0-rc.1`

## Freeze Decision

V1 scope is frozen to the OpenSpec artifacts in
`openspec/changes/build-liiiraa-boost-platform`, with
`v1-tweak-matrix.md` as the optimizer scope lock and `tasks.md` as the
execution map.

After this checkpoint:

- Do not add new V1 tweak IDs, product surfaces, release behaviors, telemetry
  fields, API procedures, privileged commands, or updater/catalog capabilities
  without a new OpenSpec task or change.
- Release-candidate work may fix defects found by the gates below, but it must
  not expand the product promise.
- Blocked guardrails remain part of V1 scope. Removing a denial is a scope
  change and needs a new OpenSpec decision.
- Auth, billing, payments, affiliate flows, admin dashboards, automatic
  overclocking, kernel drivers, anti-cheat bypass behavior, BIOS/firmware
  mutation, and direct desktop-to-Neon access remain out of scope.

## Read-First Coverage

T115 read scope was `all OpenSpec docs`. The dry run reviewed the change
metadata, proposal, design, task map, roadmap, definition of done, research,
security, performance, update, UI/UX, visual identity, tweak standard,
hardening review, tweak catalog, V1 tweak matrix, and every spec under
`openspec/changes/build-liiiraa-boost-platform/specs`.

## Quality Gate Results

| Gate | Command | Result |
| --- | --- | --- |
| Aggregate check | `pnpm check` | Blocked locally after JS gates passed because `cargo` is not available on PATH. |
| JavaScript workspace check | `pnpm check:js` | Passed: Turbo reported 32 successful tasks. |
| Rust workspace tests | `cargo test --workspace` | Blocked locally: PowerShell could not resolve `cargo`. |
| JavaScript tests | `pnpm test` | Passed: Turbo reported 8 successful test tasks. |
| Build smoke | `pnpm build` | Passed: Turbo reported 8 successful build tasks; cache-output warnings only. |
| Release workflow validation | `node scripts/validate-release-workflow.mjs` | Passed. |
| Release secret scan | `node scripts/check-release-no-secrets.mjs` | Passed across 235 text files. |
| Release changelog dry run | `node scripts/generate-release-changelog.mjs --tag v0.0.0-rc.1 --output "$env:TEMP\\liiiraa-rc-changelog.md" --allow-missing-tag` | Passed; changelog generated in the local temp directory. |
| Performance budget smoke | `node scripts/check-performance-budgets.mjs` | Passed all 10 budget checks. |
| Tweak documentation gate | `node scripts/validate-tweak-documentation.mjs` | Passed for 195 tweak rows across 13 sections. |
| Commitlint dry run | `pnpm commitlint --from HEAD~1 --to HEAD` | Passed with a warning that the previous `tweaks` scope is outside the recommended scope list. |

## Release Checklist

| Area | RC status | Notes |
| --- | --- | --- |
| V1 scope lock | Passed | Scope is frozen to OpenSpec docs and `v1-tweak-matrix.md`. |
| Task discipline | Passed | T115 changed checklist documentation only and did not start T116 or any future task. |
| JS quality | Passed | `pnpm check:js`, `pnpm test`, and `pnpm build` passed. |
| Rust quality | Blocked locally | Install or expose Rust/Cargo, then rerun `pnpm check` and `cargo test --workspace`. |
| Release workflow | Passed | Release workflow has dry-run dispatch, least-privilege permissions, signed-tag checks, no-secret scan, changelog generation, and attestation step validation. |
| Secret handling | Passed | No release secret scan findings in tracked or untracked releasable text files. |
| Tweak documentation | Passed | The matrix and hardening docs satisfy source, do/dont, backup, verify, rollback, risk, anti-cheat, applicability, conflicts, and side-effect gates. |
| Performance budgets | Passed | Local smoke measurements stayed within startup, idle, scan, UI responsiveness, and benchmark-overhead budgets. |
| Signed tag | Not executed | Dry run used a missing RC tag with `--allow-missing-tag`; stable promotion still requires a signed Git tag. |
| Windows signing | Not executed | No Windows release artifacts were built in this task. Stable promotion still requires Authenticode-signed app, installer, uninstaller, and helper executables. |
| Tauri updater signing | Not executed | No updater artifacts were built in this task. Stable promotion still requires signed updater metadata/artifacts and no private updater key in repo or desktop output. |
| Catalog rollback | Passed by tests | Catalog tests covered signed catalogs, tamper rejection, revoked catalogs, disabled versions, and rollback catalog behavior through JS gates. |

## Stable Promotion Blockers

- Rust/Cargo is not available in the local execution environment, so
  `cargo check --workspace --all-targets` and `cargo test --workspace` could
  not run here.
- A stable release tag must be created and signed before publish.
- Windows installer, app executable, uninstaller, helper executables, and Tauri
  updater artifacts must be built, signed, and verified before stable
  distribution.

## RC Verdict

The V1 scope is frozen. JavaScript, documentation, release-script, security,
performance, and catalog gates passed in the dry run. Stable promotion is not
green from this workstation until Rust tooling is available and the signed
Windows/updater artifact gates are executed.
