## ADDED Requirements

### Requirement: Locale-Ready Desktop Copy
Redesigned desktop surfaces SHALL source user-facing copy from a locale-ready copy boundary instead of hardcoding new visible strings directly inside route components.

#### Scenario: Redesigned component renders copy
- **WHEN** a redesigned desktop component renders labels, headings, buttons, badges, empty states, warnings, tooltips, or helper text
- **THEN** the copy SHALL be read from typed locale keys with support for interpolation where dynamic values are needed.

#### Scenario: Missing locale key occurs
- **WHEN** a requested locale key is missing in the active locale
- **THEN** the UI SHALL fall back to the configured default locale and expose the missing key during development or tests.

### Requirement: Supported Locale Plan
The desktop app SHALL prepare for Brazilian Portuguese, English, and Spanish without requiring route rewrites when translations are filled.

#### Scenario: Locale catalog exists
- **WHEN** the locale system is introduced
- **THEN** it SHALL define `pt-BR`, `en-US`, and `es-ES` catalogs or placeholders with a documented default locale and fallback order.

#### Scenario: Current English UI remains during transition
- **WHEN** a string has not yet received final Portuguese or Spanish copy
- **THEN** the implementation MAY keep the current English text through the locale catalog, but SHALL NOT add new hardcoded English text in redesigned surfaces.

### Requirement: Optimization Terminology Consistency
Optimizer-specific terms SHALL use a shared glossary so risk, rollback, benchmark, scan, apply, and trust language remain consistent across locales.

#### Scenario: Tweak risk text renders
- **WHEN** Safe, Competitive, Lab, Blocked, rollback, reboot, benchmark, confidence, or source terms appear in the UI
- **THEN** they SHALL use glossary-backed locale keys so the same concept is translated consistently across Dashboard, Scan, Optimize, Benchmarks, Rollback, and Settings.

#### Scenario: Benchmark metric renders
- **WHEN** FPS, 1% low, 0.1% low, p95 frame time, latency, CPU, GPU, RAM, driver, power plan, or variance metrics appear
- **THEN** units and metric labels SHALL be locale-ready while preserving technical abbreviations that are standard across supported languages.

### Requirement: Locale Visual Fit
The UI SHALL account for longer Portuguese and Spanish labels without breaking layout.

#### Scenario: Non-English locale is tested
- **WHEN** a visual smoke test renders Portuguese or Spanish copy
- **THEN** primary navigation, buttons, status strip items, badges, tables, and cards SHALL wrap, truncate with accessible tooltip, or resize within stable layout constraints without overlapping adjacent content.

#### Scenario: Locale text appears in compact controls
- **WHEN** compact controls such as segmented modes, icon buttons, table headers, and risk badges render translated text
- **THEN** they SHALL preserve accessible labels/tooltips and stable control dimensions.
