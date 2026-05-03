## ADDED Requirements

### Requirement: Liiiraa-native premium visual direction
The product SHALL use a polished Liiiraa-native visual direction inspired by premium gaming optimizer references without copying external branding, assets, exact copy, screenshots, or unsupported claims.

#### Scenario: Reference adaptation review
- **WHEN** a reviewer compares the implemented UI against Hone references
- **THEN** the UI demonstrates similar polish, clarity, and separation while retaining Liiiraa branding, assets, colors, copy, and product-specific structure

#### Scenario: External asset protection
- **WHEN** the implementation introduces visual assets or copy
- **THEN** no Hone logo, screenshot, icon, exact phrase, benchmark claim, testimonial, or proprietary visual asset is used

### Requirement: Shared visual tokens
The system SHALL define and consume shared tokens for colors, typography, spacing, radius, shadow, motion, button dimensions, icon sizing, surface hierarchy, and semantic states across desktop and web.

#### Scenario: Tokenized visible styling
- **WHEN** a primary product surface uses a visible color, radius, shadow, spacing rhythm, or motion timing
- **THEN** the value comes from the shared token layer or from a documented component variant

#### Scenario: Cross-surface consistency
- **WHEN** desktop and web both render primary buttons, cards, proof tiles, badges, or section headers
- **THEN** they use compatible proportions, radii, spacing, contrast, and interaction states

### Requirement: Action component polish
The system SHALL provide consistent button, icon-button, segmented-control, tab, toggle, destructive-action, loading, disabled, and success states with accessible labels and professional hover/focus feedback.

#### Scenario: Primary action hierarchy
- **WHEN** a user views any main desktop route or landing-page viewport
- **THEN** there is no more than one visually dominant primary action in that decision area

#### Scenario: Button state coverage
- **WHEN** a button is hovered, focused, pressed, loading, disabled, successful, locked, or destructive
- **THEN** its visual state is distinct, accessible, and consistent with the shared action grammar

### Requirement: Controlled visual density
The UI SHALL avoid crowded mixed-purpose panels by using clear grouping, stable spacing, compact headings, and progressive disclosure for dense technical information.

#### Scenario: First-screen clarity
- **WHEN** a user opens the desktop app or landing page
- **THEN** the first viewport presents the product purpose, current state, and next action without overlapping text, nested-card clutter, or competing hero-scale headings

#### Scenario: Advanced detail disclosure
- **WHEN** technical details exceed the space needed for the main decision
- **THEN** the details appear in a ledger, inspector, drawer, table, tooltip, or secondary section instead of competing with the primary action

### Requirement: Motion and interaction finish
The UI SHALL use subtle motion and interaction feedback to communicate navigation, progress, state changes, and completion without distracting from optimizer tasks.

#### Scenario: Optimization progress feedback
- **WHEN** a scan, apply, benchmark, or rollback action is running
- **THEN** the UI shows stable progress feedback, button loading state, and route-level status without layout shift

#### Scenario: Reduced motion support
- **WHEN** the user or browser requests reduced motion
- **THEN** nonessential animations are removed or shortened while state changes remain understandable
