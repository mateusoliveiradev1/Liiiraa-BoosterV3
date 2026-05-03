## ADDED Requirements

### Requirement: PUBG Discovery
The app SHALL detect PUBG installation, executable, config folder, launcher context, and BattlEye presence without modifying game binaries.

#### Scenario: PUBG installed through Steam
- **WHEN** PUBG is installed through Steam
- **THEN** the app SHALL locate `TslGame.exe` and relevant config paths.

### Requirement: Official Guidance Alignment
The app SHALL align PUBG optimizations with official support guidance where available.

#### Scenario: Launch options found
- **WHEN** legacy PUBG launch options are detected
- **THEN** the app SHALL recommend removing them rather than adding new unsupported launch flags.

#### Scenario: PUBG file corruption suspected
- **WHEN** crashes, missing files, or repeated stutter symptoms indicate possible corruption
- **THEN** the app SHALL recommend Steam/Epic verify/repair flows instead of deleting or modifying game content folders.

#### Scenario: Crash evidence requested
- **WHEN** the user chooses to collect crash evidence
- **THEN** the app MAY export relevant PUBG crash/log paths but SHALL avoid uploading them without consent.

### Requirement: DirectX Mode Benchmark
The app SHALL treat DX11 Enhanced vs DX11 as a benchmarked per-machine decision.

#### Scenario: DX mode recommendation
- **WHEN** the app recommends DX11 Enhanced or DX11
- **THEN** the recommendation SHALL include benchmark or stability rationale.

### Requirement: Anti-Cheat Boundaries
The app SHALL NOT modify PUBG game binaries, content folders, BattlEye files, game memory, process memory, kernel state, launch integrity, or anti-cheat behavior.

#### Scenario: Requested game file modification
- **WHEN** a tweak attempts to modify unsupported PUBG or BattlEye files
- **THEN** the app SHALL reject the tweak.

### Requirement: PUBG Competitive Preset
The app SHALL provide a PUBG competitive settings checklist focused on FPS stability, visibility, and latency.

#### Scenario: User opens PUBG profile
- **WHEN** the user opens PUBG profile
- **THEN** the UI SHALL show current settings, recommendations, risk, and whether each recommendation is automatic or manual.
