## ADDED Requirements

### Requirement: TDD for Optimizer Core
Optimizer core behavior SHALL be test-driven.

#### Scenario: New tweak
- **WHEN** a new tweak is implemented
- **THEN** tests SHALL cover detection, dry-run plan, backup metadata, mode validation, verify behavior, and rollback metadata.

### Requirement: E2E Coverage
The project SHALL include Playwright E2E tests for critical user flows.

#### Scenario: Optimize flow
- **WHEN** the E2E suite runs
- **THEN** it SHALL cover scan -> plan -> apply simulation -> verify -> rollback.

### Requirement: Contract Tests
The API, desktop IPC, and database schemas SHALL have contract validation.

#### Scenario: tRPC router change
- **WHEN** a router procedure changes
- **THEN** contract tests and TypeScript typecheck SHALL validate consumers.

### Requirement: Windows Integration Tests
Windows-specific adapters SHALL have integration tests gated for Windows runners.

#### Scenario: Non-Windows CI
- **WHEN** tests run on non-Windows hosts
- **THEN** Windows adapter integration tests SHALL be skipped with explicit reporting, not silently ignored.

### Requirement: Release Quality
Release builds SHALL verify Tauri build, updater configuration, signed artifacts, no embedded secrets, and rollback documentation.

#### Scenario: Release candidate
- **WHEN** a release candidate is built
- **THEN** release checks SHALL fail if updater signing, secret scanning, or rollback docs are missing.
