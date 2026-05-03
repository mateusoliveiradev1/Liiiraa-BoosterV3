## ADDED Requirements

### Requirement: NVIDIA Detection
The app SHALL detect NVIDIA GPU presence, driver version, profile API availability, and relevant monitor refresh rates.

#### Scenario: No NVIDIA GPU
- **WHEN** no NVIDIA GPU is detected
- **THEN** NVIDIA optimization modules SHALL be hidden or marked unavailable.

### Requirement: Global NVIDIA Profile
The app SHALL provide a `Liiiraa Boost - Global Performance` NVIDIA profile for full-PC performance.

#### Scenario: Global profile apply
- **WHEN** the global profile is applied
- **THEN** the app SHALL back up current customized profiles, apply approved settings, verify them, and expose rollback.

### Requirement: PUBG NVIDIA Profile
The app SHALL provide a `Liiiraa Boost - PUBG Competitive` profile associated with PUBG executable paths.

#### Scenario: PUBG profile apply
- **WHEN** the PUBG profile is applied
- **THEN** the app SHALL apply settings to PUBG executables instead of forcing all settings globally.

### Requirement: NVIDIA Settings Safety
The app SHALL prefer documented or user-visible NVIDIA settings for default profiles.

#### Scenario: Hidden setting requested
- **WHEN** a hidden or undocumented NVIDIA setting is proposed
- **THEN** it SHALL be classified as Lab and require explicit opt-in and rollback.

### Requirement: NVIDIA Latency and VRR Policy
The app SHALL distinguish between NVIDIA Reflex, driver Low Latency Mode, Max Frame Rate, G-SYNC/VRR, and power-management mode.

#### Scenario: Reflex-supported game
- **WHEN** a game supports NVIDIA Reflex
- **THEN** the app SHALL prefer the in-game Reflex path and SHALL NOT blindly stack driver Ultra Low Latency as a default.

#### Scenario: VRR display detected
- **WHEN** a G-SYNC/VRR display is detected
- **THEN** the app SHALL recommend a profile-specific FPS cap below refresh rate and verify the cap source.

### Requirement: NVIDIA Resizable BAR
The app SHALL detect Resizable BAR status and NVIDIA driver support.

#### Scenario: ReBAR unsupported or disabled
- **WHEN** ReBAR is disabled or unsupported
- **THEN** the app SHALL recommend official BIOS/VBIOS/driver checks but SHALL NOT flash firmware.

#### Scenario: Hidden ReBAR override requested
- **WHEN** a hidden per-game ReBAR override is requested through NPI compatibility
- **THEN** it SHALL be Lab-only, benchmark-gated, and rollbackable because NVIDIA enables ReBAR per title after validation.

### Requirement: NVIDIA Overclocking Boundary
The app SHALL treat NVIDIA App automatic tuning or third-party GPU overclocking as Lab/advisory.

#### Scenario: GPU auto-tuning requested
- **WHEN** the user requests automatic GPU tuning
- **THEN** the app SHALL explain heat, stability, warranty, and rollback limits and SHALL NOT run firmware or voltage changes silently.

### Requirement: BattlEye Compatibility
The app SHALL avoid applying NVIDIA profile changes while PUBG or BattlEye processes are running.

#### Scenario: PUBG running
- **WHEN** PUBG or BattlEye is running
- **THEN** the app SHALL defer NVIDIA profile mutation until the game is closed.
