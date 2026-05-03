# Development Workflow

## Repository
- Remote: `https://github.com/mateusoliveiradev1/Liiiraa-BoosterV3.git`
- Default branch: `main`
- Integration branch pattern: `feat/<short-scope>` or `chore/<short-scope>`
- Work style: micro-steps with small commits and frequent pushes.

## Commit Standard
Use Conventional Commits 1.0.0.

Format:
```text
<type>(<scope>): <short imperative summary>

[optional body]

[optional footer(s)]
```

Allowed types:
- `feat`: user-facing or product feature
- `fix`: bug fix
- `perf`: performance improvement
- `refactor`: behavior-preserving code change
- `test`: tests only
- `docs`: documentation/spec/design changes
- `build`: dependency, bundler, package, or build system changes
- `ci`: CI/CD and GitHub Actions changes
- `chore`: maintenance with no product behavior change
- `style`: formatting-only changes
- `security`: security hardening, secret handling, signing, permission, or vulnerability work
- `revert`: revert a previous commit

Recommended scopes:
- `openspec`
- `repo`
- `desktop`
- `api`
- `web`
- `db`
- `ui`
- `security`
- `performance`
- `optimizer`
- `windows`
- `nvidia`
- `pubg`
- `benchmark`
- `release`

Examples:
```text
docs(openspec): define optimizer platform proposal
build(repo): scaffold pnpm turborepo workspace
security(ci): restrict github token permissions
feat(windows): add power plan detection
perf(desktop): lazy load benchmark charts
test(optimizer): cover safe mode policy
```

Breaking changes:
```text
feat(api)!: replace benchmark session contract

BREAKING CHANGE: desktop clients must send profileVersion.
```

## Micro-Step Policy
Each implementation micro-step should:
1. Change one coherent thing.
2. Run the smallest relevant verification.
3. Commit with a Conventional Commit message.
4. Push after the commit when the workspace is in a coherent state.

Micro-step examples:
- create workspace config
- add one package
- add one crate
- add one domain type
- add one tweak definition
- add one UI screen shell
- add one test file
- add one CI job

Avoid commits that combine unrelated layers, for example UI redesign + database migration + Windows registry adapter.

## Push Policy
- Push every completed micro-step to the active branch.
- Do not push broken code unless the branch name and commit message clearly mark it as an intentional WIP spike.
- Prefer `git push origin <branch>` after each green micro-step.
- Never force-push `main`.
- If force-push is needed on a feature branch, use `--force-with-lease`.

## Commit Security
Required:
- signed commits using GPG or SSH signing
- signed tags for releases
- `.gitignore` for secrets and build output
- secret scanning before push
- no `.env`, Neon credentials, signing keys, private updater keys, or certificates committed
- no binary release artifacts committed to source unless explicitly justified

Recommended local config:
```bash
git config commit.gpgsign true
git config tag.gpgsign true
git config gpg.format ssh
git config user.signingkey ~/.ssh/<your-signing-key>.pub
```

Use GPG instead of SSH signing if that is how Liiiraa's GitHub account is configured.

## Repository Protection
Enable GitHub branch protection or repository rules for `main`:
- require signed commits
- require pull request before merge once more contributors exist
- require status checks
- require linear history
- block force pushes
- block deletions
- require conversation resolution
- require CodeQL/security checks when available

## Commit Hooks
Use:
- `commitlint` with Conventional Commits
- Husky `commit-msg` hook
- Husky `pre-commit` hook for lint-staged fast checks
- optional `pre-push` hook for targeted tests

Hooks are developer feedback, not the only gate. CI remains authoritative.

## CI Security
GitHub Actions must:
- set default `permissions: read-all` or explicit minimal permissions
- pin third-party actions to full-length commit SHAs for stable/release workflows
- avoid `pull_request_target` unless there is a reviewed reason
- avoid interpolating untrusted GitHub context directly in shell scripts
- use environments/reviewers for signing and deployment secrets
- use Dependabot, secret scanning, CodeQL, cargo audit, and pnpm audit where practical
- produce artifact attestations for release builds when supported

## Release Commit Policy
Release commits/tags must:
- be signed
- include changelog
- include build provenance or artifact attestation where supported
- include Tauri updater signature
- include Windows code-signing status
- reference the exact tweak catalog version

## AI Agent Rules
Any AI implementation agent must:
- keep commits small and conventional
- push after coherent micro-steps if credentials are available
- never commit secrets
- never bypass tests to make progress look faster
- include verification notes before committing risky modules
- keep visual changes aligned with `visual-design.md`
