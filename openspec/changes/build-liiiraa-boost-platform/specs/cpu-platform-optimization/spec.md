## ADDED Requirements

### Requirement: CPU Platform Detection
The app SHALL detect CPU vendor, model, generation, topology, hybrid-core status, SMT/Hyper-Threading status, cache/CCD hints where available, chipset driver signals, thermal throttling signals, power throttling signals, and active processor power-management settings.

#### Scenario: CPU scan completed
- **WHEN** the system scan runs
- **THEN** the report SHALL identify Intel, AMD, or unknown CPU platform capabilities without applying changes.

### Requirement: Intel CPU Optimization Policy
The app SHALL support Intel CPU optimization through detection, Windows PPM settings, Intel APO/DTT readiness checks, and scheduler guardrails.

#### Scenario: Intel APO capable platform
- **WHEN** an Intel CPU supports Intel Application Optimization or limited Advanced Mode
- **THEN** the app SHALL detect BIOS/DTT/driver/Windows requirements and recommend the official Intel path instead of forcing affinity or disabling E-cores.

#### Scenario: Intel hybrid CPU detected
- **WHEN** Intel P-core/E-core topology is detected
- **THEN** the app SHALL prefer OS/Thread Director-friendly behavior and SHALL NOT disable E-cores by default.

### Requirement: AMD CPU Optimization Policy
The app SHALL support AMD CPU optimization through chipset driver checks, CPPC/preferred-core readiness, X3D scheduling readiness, Windows Game Mode dependency checks, and safe power-management planning.

#### Scenario: AMD X3D multi-CCD CPU detected
- **WHEN** a multi-CCD AMD X3D CPU is detected
- **THEN** the app SHALL verify AMD chipset/X3D scheduling components and Game Mode readiness and SHALL recommend official driver repair/update before any scheduler workaround.

#### Scenario: AMD Precision Boost/PBO requested
- **WHEN** the user asks for Precision Boost Overdrive, Curve Optimizer, or Ryzen Master tuning
- **THEN** the app SHALL classify it as Lab/advisory and SHALL explain warranty, stability, heat, and motherboard dependency.

### Requirement: CPU Overclocking Guardrails
The app SHALL NOT automatically overclock, undervolt, disable SMT/Hyper-Threading, disable E-cores, disable security mitigations, force realtime priority, or apply BIOS/firmware changes.

#### Scenario: Unsafe CPU tweak proposed
- **WHEN** a tweak attempts automatic OC/undervolt, SMT disable, E-core disable, CPU mitigation disable, or firmware mutation
- **THEN** the engine SHALL deny it outside explicit Lab advisory flows.

### Requirement: App CPU Friendliness
The app SHALL avoid becoming a performance problem while optimizing.

#### Scenario: Game is running
- **WHEN** a game or benchmark session is active
- **THEN** Liiiraa Booster background work SHALL use cancellation, concurrency limits, and low-impact scheduling so scans do not steal CPU from the game.
