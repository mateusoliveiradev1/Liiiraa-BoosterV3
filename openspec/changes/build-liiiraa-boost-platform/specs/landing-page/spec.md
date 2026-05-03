## ADDED Requirements

### Requirement: Product Landing Page
The platform SHALL include a planned landing page under `apps/web`.

#### Scenario: First viewport
- **WHEN** the landing page loads
- **THEN** the first viewport SHALL clearly show the Liiiraa Boost product, not a generic marketing card.

### Requirement: Proof-Oriented Marketing
The landing page SHALL emphasize real optimization proof, benchmarks, rollback safety, and PUBG/game focus.

#### Scenario: Claims shown
- **WHEN** performance claims are displayed
- **THEN** they SHALL be backed by benchmark visuals, methodology notes, or cautious wording.

### Requirement: Future Auth and Pricing
The landing page SHALL reserve sections for auth, pricing, affiliate, and purchase flows without implementing them in this change.

#### Scenario: User clicks purchase CTA
- **WHEN** auth/billing is not implemented
- **THEN** the CTA SHALL route to a placeholder/waitlist/contact path rather than a fake checkout.

### Requirement: Visual QA
The landing page SHALL be tested across desktop and mobile viewports.

#### Scenario: Responsive check
- **WHEN** Playwright visual checks run
- **THEN** text SHALL not overflow, overlap, or obscure product screenshots.
