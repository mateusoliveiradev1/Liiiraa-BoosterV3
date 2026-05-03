## ADDED Requirements

### Requirement: System Scan
The app SHALL scan Windows version, build, CPU, RAM, GPU, storage, network adapters, services, scheduled tasks, startup apps, power plan, VBS/HVCI state, and reboot-required state.

#### Scenario: Scan completed
- **WHEN** scan completes
- **THEN** the app SHALL produce a typed system report without applying changes.

### Requirement: Liiiraa Boost Power Plans
The app SHALL create named Windows power plans for Balanced, Performance, and Competitive profiles.

#### Scenario: Competitive plan apply
- **WHEN** the user applies the Competitive power plan
- **THEN** the app SHALL store the previous active scheme, apply the Liiiraa Boost plan, verify active scheme, and expose rollback.

#### Scenario: Laptop detected
- **WHEN** the device is a laptop
- **THEN** the app SHALL show battery and heat warnings and use less aggressive defaults unless the user opts in.

### Requirement: Safe Windows Tweaks
The app SHALL support safe Windows optimizations including Game DVR/capture toggles, background app controls, startup controls, NTFS metadata options, network adapter power saving, USB selective suspend, PCIe link state, and power throttling.

#### Scenario: Safe tweak apply
- **WHEN** a safe Windows tweak is applied
- **THEN** the app SHALL back up the previous value and verify the new state.

### Requirement: Display and Present Path Safety
The app SHALL inspect display refresh rate, VRR, HDR, ICC/color profile usage, overlays, capture state, HAGS, and windowed-game optimization state before graphics-present tweaks.

#### Scenario: Color-sensitive display setup detected
- **WHEN** a GameDVR, FSO, HAGS, VRR, capture, or overlay tweak may affect color/HDR/present behavior
- **THEN** the app SHALL warn the user and prefer benchmarked per-game changes.

### Requirement: Network Tuning Safety
The app SHALL treat adapter power saving and Energy Efficient Ethernet separately from RSC/offload/interrupt/buffer tuning.

#### Scenario: Advanced network tweak requested
- **WHEN** RSC, offloads, interrupt moderation, buffers, TCP packs, or Jumbo Frames are requested
- **THEN** the app SHALL classify them as Lab or blocked unless adapter-specific support and benchmark evidence exist.

### Requirement: Storage Tuning Safety
The app SHALL use Windows-supported storage checks and cleanup paths.

#### Scenario: Unsupported storage driver hack requested
- **WHEN** a tweak attempts unsupported NVMe/server-driver registry hacks or blind game-content deletion
- **THEN** the app SHALL deny the action.

### Requirement: Security Tradeoff Tweaks
The app MAY support VBS, Memory Integrity, and Virtual Machine Platform changes only as explicit Competitive/Lab tradeoffs.

#### Scenario: VBS tweak selected
- **WHEN** the user selects a VBS/HVCI/VMP performance tradeoff
- **THEN** the app SHALL explain the security impact, reboot requirement, and recovery path before apply.

### Requirement: Forbidden Defaults
The app SHALL NOT disable Defender, Windows Update, UAC, pagefile, driver signature enforcement, kernel protections, CPU security mitigations, Smart App Control, or Windows system binaries in default optimization.

#### Scenario: Safe mode plan includes forbidden default
- **WHEN** a Safe mode plan includes a forbidden default action
- **THEN** validation SHALL fail before apply.
