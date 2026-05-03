## ADDED Requirements

### Requirement: Performance Budgets
The app SHALL define and test performance budgets for startup, idle CPU, memory, scan time, UI responsiveness, and benchmark overhead.

#### Scenario: Performance budget check
- **WHEN** performance smoke tests run
- **THEN** they SHALL fail or warn when agreed budgets are exceeded.

### Requirement: Maximum Practical Performance Baseline
The project SHALL use `performance-max-plan.md` as the baseline for app responsiveness, scan scheduling, benchmark overhead, and idle behavior.

#### Scenario: Performance-sensitive task
- **WHEN** a task adds scanning, charting, benchmark parsing, background work, polling, or startup code
- **THEN** it SHALL satisfy the relevant budget and implementation rule in `performance-max-plan.md`.

### Requirement: Non-Blocking UI
The desktop UI SHALL remain responsive during scans, optimization planning, apply, verify, rollback, and benchmark parsing.

#### Scenario: Long system scan
- **WHEN** a scan takes multiple seconds
- **THEN** the UI SHALL show progress and allow cancellation without blocking navigation.

### Requirement: Lazy Heavy Modules
Heavy modules SHALL load only when needed.

#### Scenario: User never opens NVIDIA page
- **WHEN** the user does not open NVIDIA optimization
- **THEN** NVIDIA profile tooling SHALL NOT be initialized during startup.

### Requirement: Scan Scheduling
System scans SHALL batch expensive Windows calls and limit concurrency.

#### Scenario: Full scan
- **WHEN** a full scan runs
- **THEN** WMI, registry, service, scheduled task, GPU, and storage reads SHALL be scheduled to avoid CPU spikes and UI stalls.

### Requirement: Efficient Benchmark Processing
Benchmark data SHALL be parsed and rendered without flooding the UI process.

#### Scenario: Large PresentMon capture
- **WHEN** a large benchmark capture is parsed
- **THEN** parsing SHALL run off the UI thread and chart data SHALL be downsampled before rendering.

### Requirement: Idle Discipline
The app SHALL not run continuous background optimization, polling, telemetry, or benchmarking without user intent.

#### Scenario: App idle
- **WHEN** the app is open and no scan/game/benchmark is active
- **THEN** CPU, disk, network, and GPU activity SHALL stay near idle.

### Requirement: Local-First Responsiveness
Critical flows SHALL not require cloud availability.

#### Scenario: API offline
- **WHEN** the cloud API is unavailable
- **THEN** scan, local optimization, rollback, and local benchmark history SHALL continue to work.
