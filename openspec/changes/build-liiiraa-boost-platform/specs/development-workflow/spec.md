## ADDED Requirements

### Requirement: Conventional Commits
The repository SHALL use Conventional Commits 1.0.0 for every commit.

#### Scenario: Commit message validation
- **WHEN** a developer creates a commit
- **THEN** commitlint SHALL validate the commit message against the project convention.

### Requirement: Micro-Step Commits and Pushes
Implementation SHALL be split into coherent micro-steps with one focused commit per step and a push after each coherent step when credentials are available.

#### Scenario: Task completed
- **WHEN** a task or subtask reaches a coherent verified state
- **THEN** the developer or AI agent SHALL commit and push that micro-step before starting unrelated work.

#### Scenario: Work is intentionally experimental
- **WHEN** a spike or WIP commit is pushed
- **THEN** the branch name and commit message SHALL clearly identify it as experimental and not production-ready.

### Requirement: Signed Commits
The repository SHALL require signed commits for protected branches.

#### Scenario: Unsigned commit on protected branch
- **WHEN** an unsigned commit is pushed to a protected branch
- **THEN** repository rules SHALL reject it.

### Requirement: Branch Protection
The `main` branch SHALL use branch protection or repository rules.

#### Scenario: Direct unsafe push
- **WHEN** a direct push attempts to bypass required checks, signed commits, or linear history
- **THEN** GitHub SHALL reject it.

### Requirement: Secure GitHub Actions
GitHub Actions workflows SHALL follow least-privilege and supply-chain hardening rules.

#### Scenario: Workflow uses GitHub token
- **WHEN** a workflow uses `GITHUB_TOKEN`
- **THEN** permissions SHALL be explicitly scoped to the minimum required.

#### Scenario: Third-party action used in release workflow
- **WHEN** a third-party action is used in a stable/release workflow
- **THEN** it SHALL be pinned to a full-length commit SHA or documented as an approved exception.

### Requirement: Secret-Free Commits
The repository SHALL prevent secrets from entering commits.

#### Scenario: Secret-like value staged
- **WHEN** a secret-like value is staged
- **THEN** hooks or CI secret scanning SHALL fail before merge.

### Requirement: Release Provenance
Release builds SHALL include signed tags and build provenance or artifact attestations where available.

#### Scenario: Stable release
- **WHEN** a stable release is created
- **THEN** the release SHALL reference a signed tag, updater signature, Windows signing status, and artifact provenance status.

### Requirement: AI Agent Commit Discipline
AI agents working on this repository SHALL follow the same commit, verification, and push policy.

#### Scenario: AI completes a micro-step
- **WHEN** an AI agent completes a micro-step
- **THEN** it SHALL run relevant checks, create a Conventional Commit, and push if authentication is available.

### Requirement: Task Definition of Done
Every task SHALL inherit the repository Definition of Done.

#### Scenario: Task checkbox marked complete
- **WHEN** a task checkbox is marked complete
- **THEN** the implementation SHALL have verification evidence, a Conventional Commit, and a push status unless the user explicitly paused commits or push credentials were unavailable.

### Requirement: Task-ID Execution Map
Implementation SHALL follow `tasks.md` when bootstrapping the product.

#### Scenario: Agent starts a new chat
- **WHEN** an AI agent starts implementation in a fresh chat
- **THEN** it SHALL execute exactly one requested task ID from `tasks.md`
- **AND** it SHALL use that task's context pack, write scope, verification, and commit/push instruction.

#### Scenario: Agent needs phase orientation
- **WHEN** an AI agent needs broader phase orientation
- **THEN** it MAY consult `implementation-roadmap.md`
- **BUT** `tasks.md` SHALL remain the executable source of truth.
