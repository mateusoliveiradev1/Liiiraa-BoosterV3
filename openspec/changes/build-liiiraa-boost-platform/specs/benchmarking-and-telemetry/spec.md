## ADDED Requirements

### Requirement: Benchmark Sessions
The app SHALL capture benchmark sessions before and after optimization.

#### Scenario: Benchmark run
- **WHEN** a benchmark is started
- **THEN** the app SHALL record metadata including Windows build, driver version, active power plan, active optimizer profile, game, and timestamp.

### Requirement: Frametime Metrics
The app SHALL calculate average FPS, 1% low, 0.1% low, p50/p95/p99 frametime, and dropped/delayed frame counts where available.

#### Scenario: Results shown
- **WHEN** benchmark results are shown
- **THEN** the UI SHALL emphasize stability and latency metrics, not only average FPS.

### Requirement: Metric Honesty
Benchmark reports SHALL distinguish measured native frames from generated/interpolated frames where tooling exposes that difference, and SHALL label latency metrics by their measurement source.

#### Scenario: Frame generation enabled
- **WHEN** frame generation, AFMF, DLSS Frame Generation, or driver interpolation is active
- **THEN** the benchmark SHALL mark FPS metrics as generated/interpolated when detectable and SHALL avoid comparing them as native FPS without disclosure.

#### Scenario: True end-to-end latency unavailable
- **WHEN** the app only has render-present, GPU busy, or software timing metrics
- **THEN** the UI SHALL label them as latency proxies instead of true click-to-photon latency.

### Requirement: Consent-Based Cloud Sync
The app SHALL sync telemetry or benchmark data to the cloud only after explicit user consent.

#### Scenario: User declines telemetry
- **WHEN** telemetry is disabled
- **THEN** benchmark data SHALL remain local unless the user manually exports it.

### Requirement: Local History
The app SHALL keep local benchmark history and optimization history.

#### Scenario: Offline usage
- **WHEN** the user is offline
- **THEN** scans, benchmarks, and rollbacks SHALL continue to work locally.
