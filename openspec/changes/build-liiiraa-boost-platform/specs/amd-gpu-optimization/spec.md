## ADDED Requirements

### Requirement: AMD GPU Detection
The app SHALL detect AMD Radeon GPU presence, driver version, Adrenalin feature availability, display refresh/FreeSync state, Resizable BAR/Smart Access Memory readiness where available, and supported per-game profile features.

#### Scenario: No AMD GPU
- **WHEN** no AMD GPU is detected
- **THEN** AMD GPU optimization modules SHALL be hidden or marked unavailable.

### Requirement: AMD Feature Planner
The app SHALL provide safe AMD feature planning for HYPR-RX, Anti-Lag, Radeon Boost, Radeon Chill, Enhanced Sync, FreeSync, frame caps, Radeon Image Sharpening/RSR, AFMF/frame generation, and SAM/ReBAR readiness.

#### Scenario: Competitive game profile
- **WHEN** the user optimizes a competitive game such as PUBG
- **THEN** latency-first settings SHALL be preferred over frame-generation or dynamic-resolution features unless the user explicitly chooses a visual/single-player profile.

### Requirement: Smart Access Memory and ReBAR
The app SHALL detect and explain SAM/ReBAR requirements and SHALL guide the user to BIOS/driver updates where needed.

#### Scenario: SAM/ReBAR disabled or unavailable
- **WHEN** compatible hardware appears to have SAM/ReBAR disabled
- **THEN** the app SHALL provide a recommendation and benchmark plan but SHALL NOT flash BIOS/VBIOS or force unsupported driver flags.

### Requirement: AMD Lab Tweaks
AMD MPO, ULPS, and low-level registry experiments SHALL be Lab-only and issue-specific.

#### Scenario: MPO or ULPS tweak selected
- **WHEN** the user selects an AMD MPO/ULPS tweak
- **THEN** the app SHALL require explicit issue context, backup previous values, verify after reboot if needed, and provide rollback.

### Requirement: AMD Safety Guardrails
The app SHALL NOT disable AMD crash-protection services, import bulk AMD registry packs, or write undocumented values as a default optimization.

#### Scenario: Unsafe AMD registry pack
- **WHEN** a catalog or script proposes bulk undocumented AMD registry changes
- **THEN** the engine SHALL deny the action unless each setting is individually specified, sourced, classified, backed up, and placed in Lab.
