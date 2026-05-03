## ADDED Requirements

### Requirement: Reference adaptation checklist
The change SHALL include a checklist that compares the implemented UI against the selected references for polish, separation, action clarity, and product proof while verifying that Liiiraa identity remains original.

#### Scenario: Reference checklist completion
- **WHEN** the implementation is ready for review
- **THEN** the checklist records how the UI adapts reference patterns and confirms no external brand assets, exact copy, exact layout measurements, or unsupported claims were copied

### Requirement: Desktop visual screenshot gates
Desktop implementation SHALL produce screenshot artifacts for supported desktop widths and primary routes showing that surfaces are nonblank, organized, text-safe, and action-clear.

#### Scenario: Desktop screenshot review
- **WHEN** desktop visual tests run
- **THEN** screenshots exist for dashboard, optimize, scan, benchmark, rollback, settings, and at least one game/GPU-related route at supported widths

#### Scenario: Desktop route acceptance
- **WHEN** a reviewer inspects desktop screenshots
- **THEN** no primary route has overlapping controls, clipped primary actions, unreadable labels, empty main content, or inconsistent button hierarchy

### Requirement: Web visual screenshot gates
Web implementation SHALL produce screenshot artifacts for landing-page desktop and mobile viewports showing product-first presentation, clean sections, and CTA consistency.

#### Scenario: Landing screenshot review
- **WHEN** web visual tests run
- **THEN** screenshots exist for the first viewport, product proof section, feature/optimization section, trust/FAQ area, and final CTA area on desktop and mobile

#### Scenario: Landing acceptance
- **WHEN** a reviewer inspects landing screenshots
- **THEN** the page shows real product evidence, polished section separation, consistent CTAs, and no horizontal overflow or text collisions

### Requirement: Responsive and locale text fit
The UI SHALL verify that important labels, buttons, status items, and cards fit across supported desktop, web, mobile, and configured locale states.

#### Scenario: Long label check
- **WHEN** Portuguese, English, or Spanish strings render inside compact navigation, buttons, badges, status strips, tables, or cards
- **THEN** text wraps, truncates, or resizes according to component rules without overlapping adjacent content

#### Scenario: Minimum viewport check
- **WHEN** desktop or web UI is viewed at the minimum supported viewport
- **THEN** the layout preserves primary actions, product signal, and readable optimizer state without horizontal scrolling unless the component is an intentional data table

### Requirement: Automated polish checks
The project SHALL include automated checks that catch blank screens, console errors, missing routes, horizontal overflow, primary action absence, and obvious layout collisions.

#### Scenario: Automated visual smoke
- **WHEN** the visual smoke suite runs
- **THEN** it navigates the key desktop and web states, asserts required landmarks/actions, checks for console errors, and records screenshot artifacts

#### Scenario: Failing polish gate
- **WHEN** a check detects blank content, missing primary action, horizontal overflow, or visible text collision
- **THEN** the implementation is not accepted until the issue is fixed or explicitly documented with a narrow exception
