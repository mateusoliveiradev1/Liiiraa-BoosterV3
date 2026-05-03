## ADDED Requirements

### Requirement: Cloud API
The platform SHALL expose a Fastify + tRPC API for cloud-backed product data.

#### Scenario: Desktop cloud call
- **WHEN** the desktop app needs cloud data
- **THEN** it SHALL call the API and SHALL NOT connect directly to Neon.

### Requirement: Neon Postgres Persistence
The cloud backend SHALL use Neon Postgres with Drizzle ORM migrations.

#### Scenario: Migration
- **WHEN** schema changes are introduced
- **THEN** Drizzle migrations SHALL be generated, reviewed, and tested before deployment.

### Requirement: Persistent Data Model
The database SHALL support devices, app releases, tweak catalog versions, benchmark sessions, audit events, feature flags, and future auth/license entities.

#### Scenario: Auth added later
- **WHEN** authentication is added in a future change
- **THEN** the existing schema SHALL allow linking users to devices and benchmark history without rewriting optimizer data.

### Requirement: Connection Safety
The API SHALL use pooled Neon connections for runtime and direct connections for migrations where required.

#### Scenario: Serverless connection pressure
- **WHEN** API instances scale
- **THEN** pooled connections SHALL be used to avoid exhausting direct Postgres connections.
