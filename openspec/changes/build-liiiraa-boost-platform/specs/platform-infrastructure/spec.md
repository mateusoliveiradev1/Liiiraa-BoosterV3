## ADDED Requirements

### Requirement: Monorepo Workspace
The project SHALL use a pnpm workspace with Turborepo task orchestration.

#### Scenario: Shared package changes
- **WHEN** a shared package changes
- **THEN** Turborepo SHALL rebuild and retest only affected packages where possible.

#### Scenario: AI implementation boundary
- **WHEN** an implementation task targets one app, package, or crate
- **THEN** the repository structure SHALL make the write scope obvious and isolated.

### Requirement: Strict TypeScript Foundation
The TypeScript workspace SHALL use strict compiler settings and shared tsconfig packages.

#### Scenario: API contract change
- **WHEN** a backend contract changes
- **THEN** TypeScript SHALL surface impacted frontend call sites during typecheck.

### Requirement: Rust Workspace
The project SHALL use a Rust workspace for Windows optimizer modules.

#### Scenario: Windows adapter change
- **WHEN** a Windows API adapter changes
- **THEN** optimizer-core tests SHALL remain independent from live Windows state by using traits/fakes.

### Requirement: CI Pipeline
The repository SHALL include CI pipelines for TypeScript, Rust, database migrations, and E2E tests.

#### Scenario: Pull request
- **WHEN** a pull request is opened
- **THEN** CI SHALL run typecheck, lint, tests, and migration validation before merge.
