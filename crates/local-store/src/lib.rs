//! SQLite local persistence for optimizer history, audit events, pending sync,
//! and benchmark captures.

use std::{
    error,
    fmt::{self, Write as _},
    path::Path,
};

use rusqlite::{params, Connection, OptionalExtension, Row};

/// Current local SQLite schema version.
pub const SCHEMA_VERSION: i64 = 1;

/// Snapshot type used for read-only PUBG config captures before recommendations.
pub const PUBG_CONFIG_SNAPSHOT_TYPE: &str = "pubg.config";

/// Payload schema version for PUBG config snapshots.
pub const PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION: &str = "pubg-config-v1";

/// Pending sync record kind for benchmark session cloud sync.
pub const BENCHMARK_SESSION_SYNC_RECORD_KIND: &str = "benchmark_session";

/// Payload schema version for minimized benchmark session sync payloads.
pub const BENCHMARK_SESSION_SYNC_SCHEMA_VERSION: &str = "benchmark-session-sync-v1";

/// Initial schema for local optimizer persistence.
pub const MIGRATION_001: &str = r#"
BEGIN;

CREATE TABLE IF NOT EXISTS optimizer_snapshots (
    id TEXT PRIMARY KEY,
    snapshot_type TEXT NOT NULL,
    created_at_utc TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    CHECK (length(trim(id)) > 0),
    CHECK (length(trim(snapshot_type)) > 0),
    CHECK (length(trim(created_at_utc)) > 0),
    CHECK (length(trim(schema_version)) > 0),
    CHECK (length(trim(payload_json)) > 0)
);

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    occurred_at_utc TEXT NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target TEXT NOT NULL,
    outcome TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    CHECK (length(trim(id)) > 0),
    CHECK (length(trim(occurred_at_utc)) > 0),
    CHECK (length(trim(actor)) > 0),
    CHECK (length(trim(action)) > 0),
    CHECK (length(trim(target)) > 0),
    CHECK (length(trim(outcome)) > 0),
    CHECK (length(trim(metadata_json)) > 0)
);

CREATE TABLE IF NOT EXISTS benchmark_captures (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    phase TEXT NOT NULL,
    captured_at_utc TEXT NOT NULL,
    game TEXT NOT NULL,
    windows_build TEXT NOT NULL,
    driver_version TEXT NOT NULL,
    active_power_plan TEXT NOT NULL,
    active_optimizer_profile TEXT NOT NULL,
    measurement_source TEXT NOT NULL,
    average_fps REAL NOT NULL,
    one_percent_low_fps REAL NOT NULL,
    zero_point_one_percent_low_fps REAL NOT NULL,
    frametime_p50_ms REAL NOT NULL,
    frametime_p95_ms REAL NOT NULL,
    frametime_p99_ms REAL NOT NULL,
    dropped_frames INTEGER NOT NULL,
    delayed_frames INTEGER NOT NULL,
    generated_frames_detected INTEGER NOT NULL,
    latency_proxy INTEGER NOT NULL,
    CHECK (length(trim(id)) > 0),
    CHECK (length(trim(session_id)) > 0),
    CHECK (phase IN ('before', 'after', 'single')),
    CHECK (length(trim(captured_at_utc)) > 0),
    CHECK (length(trim(game)) > 0),
    CHECK (length(trim(windows_build)) > 0),
    CHECK (length(trim(driver_version)) > 0),
    CHECK (length(trim(active_power_plan)) > 0),
    CHECK (length(trim(active_optimizer_profile)) > 0),
    CHECK (length(trim(measurement_source)) > 0),
    CHECK (average_fps >= 0),
    CHECK (one_percent_low_fps >= 0),
    CHECK (zero_point_one_percent_low_fps >= 0),
    CHECK (frametime_p50_ms >= 0),
    CHECK (frametime_p95_ms >= 0),
    CHECK (frametime_p99_ms >= 0),
    CHECK (dropped_frames >= 0),
    CHECK (delayed_frames >= 0),
    CHECK (generated_frames_detected IN (0, 1)),
    CHECK (latency_proxy IN (0, 1))
);

CREATE INDEX IF NOT EXISTS idx_benchmark_captures_session
    ON benchmark_captures(session_id, captured_at_utc);

CREATE TABLE IF NOT EXISTS pending_sync (
    id TEXT PRIMARY KEY,
    record_kind TEXT NOT NULL,
    record_id TEXT NOT NULL,
    created_at_utc TEXT NOT NULL,
    consent_granted_at_utc TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at_utc TEXT,
    last_error TEXT,
    CHECK (length(trim(id)) > 0),
    CHECK (length(trim(record_kind)) > 0),
    CHECK (length(trim(record_id)) > 0),
    CHECK (length(trim(created_at_utc)) > 0),
    CHECK (length(trim(consent_granted_at_utc)) > 0),
    CHECK (length(trim(payload_json)) > 0),
    CHECK (attempts >= 0),
    UNIQUE (record_kind, record_id)
);

CREATE INDEX IF NOT EXISTS idx_pending_sync_created
    ON pending_sync(created_at_utc);

PRAGMA user_version = 1;

COMMIT;
"#;

/// Result type returned by local persistence operations.
pub type LocalStoreResult<T> = Result<T, LocalStoreError>;

/// File or in-memory SQLite store for local optimizer data.
pub struct LocalStore {
    connection: Connection,
}

impl LocalStore {
    /// Opens a SQLite database at `path` and applies migrations.
    pub fn open(path: impl AsRef<Path>) -> LocalStoreResult<Self> {
        let store = Self {
            connection: Connection::open(path)?,
        };
        store.migrate()?;
        Ok(store)
    }

    /// Opens an in-memory SQLite database and applies migrations.
    pub fn open_in_memory() -> LocalStoreResult<Self> {
        let store = Self {
            connection: Connection::open_in_memory()?,
        };
        store.migrate()?;
        Ok(store)
    }

    /// Applies all local persistence migrations.
    pub fn migrate(&self) -> LocalStoreResult<()> {
        self.connection.execute_batch(MIGRATION_001)?;
        Ok(())
    }

    /// Returns SQLite `PRAGMA user_version`.
    pub fn schema_version(&self) -> LocalStoreResult<i64> {
        Ok(self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    /// Stores a snapshot of optimizer state, such as scan, plan, or rollback data.
    pub fn insert_snapshot(&self, snapshot: &OptimizerSnapshot) -> LocalStoreResult<()> {
        validate_snapshot(snapshot)?;

        self.connection.execute(
            "INSERT INTO optimizer_snapshots (
                id, snapshot_type, created_at_utc, schema_version, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                snapshot.id.as_str(),
                snapshot.snapshot_type.as_str(),
                snapshot.created_at_utc.as_str(),
                snapshot.schema_version.as_str(),
                snapshot.payload_json.as_str(),
            ],
        )?;

        Ok(())
    }

    /// Loads one optimizer snapshot by ID.
    pub fn snapshot(&self, id: &str) -> LocalStoreResult<Option<OptimizerSnapshot>> {
        validate_identifier("id", id)?;

        self.connection
            .query_row(
                "SELECT id, snapshot_type, created_at_utc, schema_version, payload_json
                 FROM optimizer_snapshots
                 WHERE id = ?1",
                [id],
                map_snapshot,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Lists optimizer snapshots matching one snapshot type.
    pub fn snapshots_by_type(
        &self,
        snapshot_type: &str,
    ) -> LocalStoreResult<Vec<OptimizerSnapshot>> {
        validate_identifier("snapshot_type", snapshot_type)?;

        let mut statement = self.connection.prepare(
            "SELECT id, snapshot_type, created_at_utc, schema_version, payload_json
             FROM optimizer_snapshots
             WHERE snapshot_type = ?1
             ORDER BY created_at_utc, id",
        )?;
        let rows = statement.query_map([snapshot_type], map_snapshot)?;

        collect_rows(rows)
    }

    /// Lists stored PUBG config snapshots captured before recommendations.
    pub fn pubg_config_snapshots(&self) -> LocalStoreResult<Vec<OptimizerSnapshot>> {
        self.snapshots_by_type(PUBG_CONFIG_SNAPSHOT_TYPE)
    }

    /// Stores an audit event for local accountability and rollback trails.
    pub fn insert_audit_event(&self, event: &AuditEvent) -> LocalStoreResult<()> {
        validate_audit_event(event)?;

        self.connection.execute(
            "INSERT INTO audit_events (
                id, occurred_at_utc, actor, action, target, outcome, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id.as_str(),
                event.occurred_at_utc.as_str(),
                event.actor.as_str(),
                event.action.as_str(),
                event.target.as_str(),
                event.outcome.as_str(),
                event.metadata_json.as_str(),
            ],
        )?;

        Ok(())
    }

    /// Lists audit events in timestamp order.
    pub fn audit_events(&self) -> LocalStoreResult<Vec<AuditEvent>> {
        let mut statement = self.connection.prepare(
            "SELECT id, occurred_at_utc, actor, action, target, outcome, metadata_json
             FROM audit_events
             ORDER BY occurred_at_utc, id",
        )?;
        let rows = statement.query_map([], map_audit_event)?;

        collect_rows(rows)
    }

    /// Stores one benchmark capture with environment metadata and metrics.
    pub fn insert_benchmark_capture(
        &self,
        capture: &BenchmarkCapture,
    ) -> LocalStoreResult<()> {
        validate_benchmark_capture(capture)?;

        self.connection.execute(
            "INSERT INTO benchmark_captures (
                id, session_id, phase, captured_at_utc, game, windows_build,
                driver_version, active_power_plan, active_optimizer_profile,
                measurement_source, average_fps, one_percent_low_fps,
                zero_point_one_percent_low_fps, frametime_p50_ms, frametime_p95_ms,
                frametime_p99_ms, dropped_frames, delayed_frames,
                generated_frames_detected, latency_proxy
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
            )",
            params![
                capture.id.as_str(),
                capture.session_id.as_str(),
                capture.phase.as_str(),
                capture.captured_at_utc.as_str(),
                capture.game.as_str(),
                capture.windows_build.as_str(),
                capture.driver_version.as_str(),
                capture.active_power_plan.as_str(),
                capture.active_optimizer_profile.as_str(),
                capture.measurement_source.as_str(),
                capture.metrics.average_fps,
                capture.metrics.one_percent_low_fps,
                capture.metrics.zero_point_one_percent_low_fps,
                capture.metrics.frametime_p50_ms,
                capture.metrics.frametime_p95_ms,
                capture.metrics.frametime_p99_ms,
                capture.metrics.dropped_frames,
                capture.metrics.delayed_frames,
                bool_to_sql(capture.generated_frames_detected),
                bool_to_sql(capture.latency_proxy),
            ],
        )?;

        Ok(())
    }

    /// Lists benchmark captures for a session in capture order.
    pub fn benchmark_captures(
        &self,
        session_id: &str,
    ) -> LocalStoreResult<Vec<BenchmarkCapture>> {
        validate_identifier("session_id", session_id)?;

        let mut statement = self.connection.prepare(
            "SELECT
                id, session_id, phase, captured_at_utc, game, windows_build,
                driver_version, active_power_plan, active_optimizer_profile,
                measurement_source, average_fps, one_percent_low_fps,
                zero_point_one_percent_low_fps, frametime_p50_ms, frametime_p95_ms,
                frametime_p99_ms, dropped_frames, delayed_frames,
                generated_frames_detected, latency_proxy
             FROM benchmark_captures
             WHERE session_id = ?1
             ORDER BY captured_at_utc, id",
        )?;
        let rows = statement.query_map([session_id], map_benchmark_capture)?;

        collect_rows(rows)
    }

    /// Adds an item to the cloud-sync queue only after explicit consent.
    pub fn enqueue_pending_sync(
        &self,
        item: &PendingSyncItem,
        consent: SyncConsent,
    ) -> LocalStoreResult<()> {
        if consent != SyncConsent::Granted {
            return Err(LocalStoreError::consent_required("pending_sync"));
        }

        validate_pending_sync(item)?;

        self.connection.execute(
            "INSERT INTO pending_sync (
                id, record_kind, record_id, created_at_utc, consent_granted_at_utc,
                payload_json, attempts, next_attempt_at_utc, last_error
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id.as_str(),
                item.record_kind.as_str(),
                item.record_id.as_str(),
                item.created_at_utc.as_str(),
                item.consent_granted_at_utc.as_str(),
                item.payload_json.as_str(),
                item.attempts,
                item.next_attempt_at_utc.as_deref(),
                item.last_error.as_deref(),
            ],
        )?;

        Ok(())
    }

    /// Builds and queues a minimized benchmark-session sync payload after consent.
    pub fn enqueue_benchmark_session_sync(
        &self,
        request: &BenchmarkSessionSyncRequest,
        consent: SyncConsent,
    ) -> LocalStoreResult<PendingSyncItem> {
        if consent != SyncConsent::Granted {
            return Err(LocalStoreError::consent_required("benchmark_session_sync"));
        }

        validate_benchmark_session_sync_request(request)?;
        let captures = self.benchmark_captures(&request.session_id)?;
        if captures.is_empty() {
            return Err(LocalStoreError::invalid_value(
                "session_id",
                "benchmark session has no captures to sync",
            ));
        }

        let item = PendingSyncItem {
            id: request.id.clone(),
            record_kind: BENCHMARK_SESSION_SYNC_RECORD_KIND.to_owned(),
            record_id: request.session_id.clone(),
            created_at_utc: request.created_at_utc.clone(),
            consent_granted_at_utc: request.consent_granted_at_utc.clone(),
            payload_json: benchmark_session_sync_payload_json(&request.session_id, &captures),
            attempts: 0,
            next_attempt_at_utc: None,
            last_error: None,
        };

        self.enqueue_pending_sync(&item, SyncConsent::Granted)?;
        Ok(item)
    }

    /// Lists queued sync items in creation order.
    pub fn pending_sync_items(&self) -> LocalStoreResult<Vec<PendingSyncItem>> {
        let mut statement = self.connection.prepare(
            "SELECT id, record_kind, record_id, created_at_utc, consent_granted_at_utc,
                payload_json, attempts, next_attempt_at_utc, last_error
             FROM pending_sync
             ORDER BY created_at_utc, id",
        )?;
        let rows = statement.query_map([], map_pending_sync_item)?;

        collect_rows(rows)
    }
}

/// Snapshot of local optimizer state.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizerSnapshot {
    /// Stable snapshot ID.
    pub id: String,
    /// Snapshot class, such as `scan`, `plan`, or `rollback`.
    pub snapshot_type: String,
    /// UTC timestamp for snapshot creation.
    pub created_at_utc: String,
    /// Schema or catalog version that produced the payload.
    pub schema_version: String,
    /// JSON payload owned by the snapshot producer.
    pub payload_json: String,
}

/// Local audit trail event.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEvent {
    /// Stable audit event ID.
    pub id: String,
    /// UTC timestamp for the audited action.
    pub occurred_at_utc: String,
    /// Actor responsible for the action.
    pub actor: String,
    /// Stable action key.
    pub action: String,
    /// Logical target affected by the action.
    pub target: String,
    /// Stable outcome key.
    pub outcome: String,
    /// JSON metadata for local diagnostics.
    pub metadata_json: String,
}

/// Benchmark phase stored in local history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkPhase {
    /// Baseline capture before optimization.
    Before,
    /// Capture after optimization.
    After,
    /// Standalone capture with no before/after pair.
    Single,
}

impl BenchmarkPhase {
    /// Returns the stable SQLite value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
            Self::Single => "single",
        }
    }

    fn from_str(value: &str) -> LocalStoreResult<Self> {
        match value {
            "before" => Ok(Self::Before),
            "after" => Ok(Self::After),
            "single" => Ok(Self::Single),
            _ => Err(LocalStoreError::invalid_value("phase", "unknown phase")),
        }
    }
}

/// Numeric metrics captured from benchmark tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkMetrics {
    /// Average frames per second.
    pub average_fps: f64,
    /// One percent low frames per second.
    pub one_percent_low_fps: f64,
    /// Zero point one percent low frames per second.
    pub zero_point_one_percent_low_fps: f64,
    /// P50 frametime in milliseconds.
    pub frametime_p50_ms: f64,
    /// P95 frametime in milliseconds.
    pub frametime_p95_ms: f64,
    /// P99 frametime in milliseconds.
    pub frametime_p99_ms: f64,
    /// Dropped frame count when available.
    pub dropped_frames: i64,
    /// Delayed frame count when available.
    pub delayed_frames: i64,
}

/// Benchmark capture with environment metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCapture {
    /// Stable capture ID.
    pub id: String,
    /// Session grouping before and after captures.
    pub session_id: String,
    /// Benchmark phase.
    pub phase: BenchmarkPhase,
    /// UTC capture timestamp.
    pub captured_at_utc: String,
    /// Game or workload label.
    pub game: String,
    /// Windows build observed for the run.
    pub windows_build: String,
    /// GPU driver version observed for the run.
    pub driver_version: String,
    /// Active Windows power plan.
    pub active_power_plan: String,
    /// Active optimizer profile.
    pub active_optimizer_profile: String,
    /// Tooling or source that produced latency and frame metrics.
    pub measurement_source: String,
    /// Benchmark metrics.
    pub metrics: BenchmarkMetrics,
    /// Whether generated or interpolated frames were detected.
    pub generated_frames_detected: bool,
    /// Whether latency values are only a proxy rather than true end-to-end latency.
    pub latency_proxy: bool,
}

/// Request to queue one stored benchmark session for cloud sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSessionSyncRequest {
    /// Stable pending sync item ID.
    pub id: String,
    /// Stored benchmark session ID to sync.
    pub session_id: String,
    /// UTC timestamp when the queue item was created.
    pub created_at_utc: String,
    /// UTC timestamp when benchmark sync consent was granted.
    pub consent_granted_at_utc: String,
}

/// Local queue item waiting for consent-approved cloud sync.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSyncItem {
    /// Stable queue item ID.
    pub id: String,
    /// Type of local record being synced.
    pub record_kind: String,
    /// Local record ID being synced.
    pub record_id: String,
    /// UTC timestamp when the queue item was created.
    pub created_at_utc: String,
    /// UTC timestamp when the user granted sync consent.
    pub consent_granted_at_utc: String,
    /// JSON payload to send when connectivity and policy allow.
    pub payload_json: String,
    /// Number of sync attempts already made.
    pub attempts: i64,
    /// Optional UTC timestamp for retry scheduling.
    pub next_attempt_at_utc: Option<String>,
    /// Last sync error, if any.
    pub last_error: Option<String>,
}

/// Explicit user consent state for cloud sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncConsent {
    /// User has granted telemetry or benchmark sync consent.
    Granted,
    /// User has declined telemetry or benchmark sync consent.
    Denied,
}

/// Reason a local store operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalStoreErrorReason {
    /// SQLite returned an error.
    Sqlite,
    /// A required field was empty or malformed.
    InvalidValue,
    /// Cloud sync was requested without explicit consent.
    ConsentRequired,
}

impl LocalStoreErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::InvalidValue => "invalid_value",
            Self::ConsentRequired => "consent_required",
        }
    }
}

/// Structured error from local SQLite persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalStoreError {
    reason: LocalStoreErrorReason,
    field: Option<&'static str>,
    detail: Option<String>,
}

impl LocalStoreError {
    fn sqlite(detail: impl Into<String>) -> Self {
        Self {
            reason: LocalStoreErrorReason::Sqlite,
            field: None,
            detail: Some(detail.into()),
        }
    }

    fn invalid_value(field: &'static str, detail: impl Into<String>) -> Self {
        Self {
            reason: LocalStoreErrorReason::InvalidValue,
            field: Some(field),
            detail: Some(detail.into()),
        }
    }

    fn consent_required(field: &'static str) -> Self {
        Self {
            reason: LocalStoreErrorReason::ConsentRequired,
            field: Some(field),
            detail: Some("explicit user consent is required before cloud sync".to_owned()),
        }
    }

    /// Returns the failure reason.
    #[must_use]
    pub const fn reason(&self) -> LocalStoreErrorReason {
        self.reason
    }

    /// Returns the field associated with validation failures.
    #[must_use]
    pub const fn field(&self) -> Option<&'static str> {
        self.field
    }

    /// Returns extra diagnostic detail.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for LocalStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.reason.as_str())?;

        if let Some(field) = self.field {
            write!(formatter, " ({field})")?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, ": {detail}")?;
        }

        Ok(())
    }
}

impl error::Error for LocalStoreError {}

impl From<rusqlite::Error> for LocalStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::sqlite(error.to_string())
    }
}

fn map_snapshot(row: &Row<'_>) -> rusqlite::Result<OptimizerSnapshot> {
    Ok(OptimizerSnapshot {
        id: row.get(0)?,
        snapshot_type: row.get(1)?,
        created_at_utc: row.get(2)?,
        schema_version: row.get(3)?,
        payload_json: row.get(4)?,
    })
}

fn map_audit_event(row: &Row<'_>) -> rusqlite::Result<AuditEvent> {
    Ok(AuditEvent {
        id: row.get(0)?,
        occurred_at_utc: row.get(1)?,
        actor: row.get(2)?,
        action: row.get(3)?,
        target: row.get(4)?,
        outcome: row.get(5)?,
        metadata_json: row.get(6)?,
    })
}

fn map_benchmark_capture(row: &Row<'_>) -> rusqlite::Result<BenchmarkCapture> {
    let phase: String = row.get(2)?;

    Ok(BenchmarkCapture {
        id: row.get(0)?,
        session_id: row.get(1)?,
        phase: BenchmarkPhase::from_str(&phase).map_err(to_sql_conversion_error)?,
        captured_at_utc: row.get(3)?,
        game: row.get(4)?,
        windows_build: row.get(5)?,
        driver_version: row.get(6)?,
        active_power_plan: row.get(7)?,
        active_optimizer_profile: row.get(8)?,
        measurement_source: row.get(9)?,
        metrics: BenchmarkMetrics {
            average_fps: row.get(10)?,
            one_percent_low_fps: row.get(11)?,
            zero_point_one_percent_low_fps: row.get(12)?,
            frametime_p50_ms: row.get(13)?,
            frametime_p95_ms: row.get(14)?,
            frametime_p99_ms: row.get(15)?,
            dropped_frames: row.get(16)?,
            delayed_frames: row.get(17)?,
        },
        generated_frames_detected: sql_to_bool(row.get(18)?),
        latency_proxy: sql_to_bool(row.get(19)?),
    })
}

fn map_pending_sync_item(row: &Row<'_>) -> rusqlite::Result<PendingSyncItem> {
    Ok(PendingSyncItem {
        id: row.get(0)?,
        record_kind: row.get(1)?,
        record_id: row.get(2)?,
        created_at_utc: row.get(3)?,
        consent_granted_at_utc: row.get(4)?,
        payload_json: row.get(5)?,
        attempts: row.get(6)?,
        next_attempt_at_utc: row.get(7)?,
        last_error: row.get(8)?,
    })
}

fn collect_rows<T>(
    rows: impl Iterator<Item = rusqlite::Result<T>>,
) -> LocalStoreResult<Vec<T>> {
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn to_sql_conversion_error(error: LocalStoreError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(error),
    )
}

fn validate_snapshot(snapshot: &OptimizerSnapshot) -> LocalStoreResult<()> {
    validate_identifier("id", &snapshot.id)?;
    validate_identifier("snapshot_type", &snapshot.snapshot_type)?;
    validate_required("created_at_utc", &snapshot.created_at_utc)?;
    validate_required("schema_version", &snapshot.schema_version)?;
    validate_json("payload_json", &snapshot.payload_json)
}

fn validate_audit_event(event: &AuditEvent) -> LocalStoreResult<()> {
    validate_identifier("id", &event.id)?;
    validate_identifier("actor", &event.actor)?;
    validate_identifier("action", &event.action)?;
    validate_required("occurred_at_utc", &event.occurred_at_utc)?;
    validate_required("target", &event.target)?;
    validate_identifier("outcome", &event.outcome)?;
    validate_json("metadata_json", &event.metadata_json)
}

fn validate_benchmark_capture(capture: &BenchmarkCapture) -> LocalStoreResult<()> {
    validate_identifier("id", &capture.id)?;
    validate_identifier("session_id", &capture.session_id)?;
    validate_required("captured_at_utc", &capture.captured_at_utc)?;
    validate_required("game", &capture.game)?;
    validate_required("windows_build", &capture.windows_build)?;
    validate_required("driver_version", &capture.driver_version)?;
    validate_required("active_power_plan", &capture.active_power_plan)?;
    validate_required(
        "active_optimizer_profile",
        &capture.active_optimizer_profile,
    )?;
    validate_required("measurement_source", &capture.measurement_source)?;
    validate_metrics(&capture.metrics)
}

fn validate_pending_sync(item: &PendingSyncItem) -> LocalStoreResult<()> {
    validate_identifier("id", &item.id)?;
    validate_identifier("record_kind", &item.record_kind)?;
    validate_identifier("record_id", &item.record_id)?;
    validate_required("created_at_utc", &item.created_at_utc)?;
    validate_required("consent_granted_at_utc", &item.consent_granted_at_utc)?;
    validate_json("payload_json", &item.payload_json)?;

    if item.attempts < 0 {
        return Err(LocalStoreError::invalid_value(
            "attempts",
            "attempt count cannot be negative",
        ));
    }

    Ok(())
}

fn validate_benchmark_session_sync_request(
    request: &BenchmarkSessionSyncRequest,
) -> LocalStoreResult<()> {
    validate_identifier("id", &request.id)?;
    validate_identifier("session_id", &request.session_id)?;
    validate_required("created_at_utc", &request.created_at_utc)?;
    validate_required("consent_granted_at_utc", &request.consent_granted_at_utc)
}

fn benchmark_session_sync_payload_json(
    session_id: &str,
    captures: &[BenchmarkCapture],
) -> String {
    let captures_json = captures
        .iter()
        .map(benchmark_capture_sync_json)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"schemaVersion\":{},\"kind\":\"benchmark-session\",\"sessionId\":{},\"captures\":[{}]}}",
        json_string(BENCHMARK_SESSION_SYNC_SCHEMA_VERSION),
        json_string(session_id),
        captures_json
    )
}

fn benchmark_capture_sync_json(capture: &BenchmarkCapture) -> String {
    format!(
        concat!(
            "{{",
            "\"id\":{},",
            "\"phase\":{},",
            "\"capturedAtUtc\":{},",
            "\"game\":{},",
            "\"windowsBuild\":{},",
            "\"driverVersion\":{},",
            "\"activePowerPlan\":{},",
            "\"activeOptimizerProfile\":{},",
            "\"measurementSource\":{},",
            "\"metrics\":{{",
            "\"averageFps\":{},",
            "\"onePercentLowFps\":{},",
            "\"zeroPointOnePercentLowFps\":{},",
            "\"frametimeP50Ms\":{},",
            "\"frametimeP95Ms\":{},",
            "\"frametimeP99Ms\":{},",
            "\"droppedFrames\":{},",
            "\"delayedFrames\":{}",
            "}},",
            "\"generatedFramesDetected\":{},",
            "\"latencyProxy\":{}",
            "}}"
        ),
        json_string(&capture.id),
        json_string(capture.phase.as_str()),
        json_string(&capture.captured_at_utc),
        json_string(&capture.game),
        json_string(&capture.windows_build),
        json_string(&capture.driver_version),
        json_string(&capture.active_power_plan),
        json_string(&capture.active_optimizer_profile),
        json_string(&capture.measurement_source),
        capture.metrics.average_fps,
        capture.metrics.one_percent_low_fps,
        capture.metrics.zero_point_one_percent_low_fps,
        capture.metrics.frametime_p50_ms,
        capture.metrics.frametime_p95_ms,
        capture.metrics.frametime_p99_ms,
        capture.metrics.dropped_frames,
        capture.metrics.delayed_frames,
        bool_to_json(capture.generated_frames_detected),
        bool_to_json(capture.latency_proxy)
    )
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');

    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                write!(&mut output, "\\u{:04x}", character as u32)
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }

    output.push('"');
    output
}

fn bool_to_json(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn validate_metrics(metrics: &BenchmarkMetrics) -> LocalStoreResult<()> {
    validate_non_negative_number("average_fps", metrics.average_fps)?;
    validate_non_negative_number("one_percent_low_fps", metrics.one_percent_low_fps)?;
    validate_non_negative_number(
        "zero_point_one_percent_low_fps",
        metrics.zero_point_one_percent_low_fps,
    )?;
    validate_non_negative_number("frametime_p50_ms", metrics.frametime_p50_ms)?;
    validate_non_negative_number("frametime_p95_ms", metrics.frametime_p95_ms)?;
    validate_non_negative_number("frametime_p99_ms", metrics.frametime_p99_ms)?;

    if metrics.dropped_frames < 0 {
        return Err(LocalStoreError::invalid_value(
            "dropped_frames",
            "frame counts cannot be negative",
        ));
    }

    if metrics.delayed_frames < 0 {
        return Err(LocalStoreError::invalid_value(
            "delayed_frames",
            "frame counts cannot be negative",
        ));
    }

    Ok(())
}

fn validate_non_negative_number(field: &'static str, value: f64) -> LocalStoreResult<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(LocalStoreError::invalid_value(
            field,
            "metric must be finite and non-negative",
        ))
    }
}

fn validate_required(field: &'static str, value: &str) -> LocalStoreResult<()> {
    if value.trim().is_empty() {
        Err(LocalStoreError::invalid_value(
            field,
            "value cannot be empty",
        ))
    } else {
        Ok(())
    }
}

fn validate_identifier(field: &'static str, value: &str) -> LocalStoreResult<()> {
    validate_required(field, value)?;

    if value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
    {
        return Err(LocalStoreError::invalid_value(
            field,
            "identifier contains unsupported characters",
        ));
    }

    Ok(())
}

fn validate_json(field: &'static str, value: &str) -> LocalStoreResult<()> {
    let trimmed = value.trim();

    validate_required(field, trimmed)?;

    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        Ok(())
    } else {
        Err(LocalStoreError::invalid_value(
            field,
            "payload must be a JSON object or array",
        ))
    }
}

fn bool_to_sql(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn sql_to_bool(value: i64) -> bool {
    value != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> OptimizerSnapshot {
        OptimizerSnapshot {
            id: "snapshot:scan:001".to_owned(),
            snapshot_type: "scan".to_owned(),
            created_at_utc: "2026-04-30T12:00:00Z".to_owned(),
            schema_version: "catalog-v1".to_owned(),
            payload_json: "{\"score\":92}".to_owned(),
        }
    }

    fn pubg_config_snapshot(id: &str, created_at_utc: &str) -> OptimizerSnapshot {
        OptimizerSnapshot {
            id: id.to_owned(),
            snapshot_type: PUBG_CONFIG_SNAPSHOT_TYPE.to_owned(),
            created_at_utc: created_at_utc.to_owned(),
            schema_version: PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION.to_owned(),
            payload_json: "{\"files\":[]}".to_owned(),
        }
    }

    fn audit_event() -> AuditEvent {
        AuditEvent {
            id: "audit:001".to_owned(),
            occurred_at_utc: "2026-04-30T12:01:00Z".to_owned(),
            actor: "local-user".to_owned(),
            action: "snapshot.created".to_owned(),
            target: "snapshot:scan:001".to_owned(),
            outcome: "succeeded".to_owned(),
            metadata_json: "{\"source\":\"test\"}".to_owned(),
        }
    }

    fn metrics() -> BenchmarkMetrics {
        BenchmarkMetrics {
            average_fps: 182.4,
            one_percent_low_fps: 122.0,
            zero_point_one_percent_low_fps: 91.5,
            frametime_p50_ms: 5.4,
            frametime_p95_ms: 8.2,
            frametime_p99_ms: 11.8,
            dropped_frames: 2,
            delayed_frames: 1,
        }
    }

    fn benchmark_capture(id: &str, phase: BenchmarkPhase) -> BenchmarkCapture {
        BenchmarkCapture {
            id: id.to_owned(),
            session_id: "bench:session:pubg:001".to_owned(),
            phase,
            captured_at_utc: "2026-04-30T12:02:00Z".to_owned(),
            game: "PUBG".to_owned(),
            windows_build: "22631.3527".to_owned(),
            driver_version: "551.86".to_owned(),
            active_power_plan: "Liiiraa Balanced".to_owned(),
            active_optimizer_profile: "Safe".to_owned(),
            measurement_source: "presentmon-render-present".to_owned(),
            metrics: metrics(),
            generated_frames_detected: false,
            latency_proxy: true,
        }
    }

    fn pending_sync_item() -> PendingSyncItem {
        PendingSyncItem {
            id: "sync:bench:001".to_owned(),
            record_kind: "benchmark_capture".to_owned(),
            record_id: "capture:before:001".to_owned(),
            created_at_utc: "2026-04-30T12:03:00Z".to_owned(),
            consent_granted_at_utc: "2026-04-30T12:00:00Z".to_owned(),
            payload_json: "{\"captureId\":\"capture:before:001\"}".to_owned(),
            attempts: 0,
            next_attempt_at_utc: None,
            last_error: None,
        }
    }

    fn benchmark_session_sync_request() -> BenchmarkSessionSyncRequest {
        BenchmarkSessionSyncRequest {
            id: "sync:bench-session:001".to_owned(),
            session_id: "bench:session:pubg:001".to_owned(),
            created_at_utc: "2026-04-30T12:04:00Z".to_owned(),
            consent_granted_at_utc: "2026-04-30T12:00:00Z".to_owned(),
        }
    }

    #[test]
    fn migration_creates_required_schema() {
        let store = LocalStore::open_in_memory().expect("store should open");

        assert_eq!(
            store
                .schema_version()
                .expect("schema version should be readable"),
            SCHEMA_VERSION
        );

        for table in [
            "optimizer_snapshots",
            "audit_events",
            "benchmark_captures",
            "pending_sync",
        ] {
            let exists: i64 = store
                .connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table query should work");

            assert_eq!(exists, 1, "{table} should exist");
        }
    }

    #[test]
    fn stores_snapshots_audit_events_and_benchmark_history() {
        let store = LocalStore::open_in_memory().expect("store should open");

        store
            .insert_snapshot(&snapshot())
            .expect("snapshot should be stored");
        store
            .insert_audit_event(&audit_event())
            .expect("audit event should be stored");
        store
            .insert_benchmark_capture(&benchmark_capture(
                "capture:before:001",
                BenchmarkPhase::Before,
            ))
            .expect("before benchmark should be stored");
        store
            .insert_benchmark_capture(&benchmark_capture(
                "capture:after:001",
                BenchmarkPhase::After,
            ))
            .expect("after benchmark should be stored");

        let stored_snapshot = store
            .snapshot("snapshot:scan:001")
            .expect("snapshot lookup should work")
            .expect("snapshot should exist");
        assert_eq!(stored_snapshot.payload_json, "{\"score\":92}");

        let audit_events = store.audit_events().expect("audit listing should work");
        assert_eq!(audit_events.len(), 1);
        assert_eq!(audit_events[0].action, "snapshot.created");

        let captures = store
            .benchmark_captures("bench:session:pubg:001")
            .expect("benchmark history should load");
        assert_eq!(
            captures
                .iter()
                .map(|capture| capture.phase)
                .collect::<Vec<_>>(),
            vec![BenchmarkPhase::Before, BenchmarkPhase::After]
        );
        assert!(captures.iter().all(|capture| capture.latency_proxy));
    }

    #[test]
    fn lists_pubg_config_snapshots_in_capture_order() {
        let store = LocalStore::open_in_memory().expect("store should open");
        let later = pubg_config_snapshot("snapshot:pubg-config:002", "2026-04-30T12:05:00Z");
        let earlier = pubg_config_snapshot("snapshot:pubg-config:001", "2026-04-30T12:04:00Z");

        store
            .insert_snapshot(&snapshot())
            .expect("unrelated snapshot should be stored");
        store
            .insert_snapshot(&later)
            .expect("later PUBG snapshot should be stored");
        store
            .insert_snapshot(&earlier)
            .expect("earlier PUBG snapshot should be stored");

        let snapshots = store
            .pubg_config_snapshots()
            .expect("PUBG snapshots should list");

        assert_eq!(snapshots, vec![earlier, later]);
    }

    #[test]
    fn gates_pending_sync_on_explicit_consent() {
        let store = LocalStore::open_in_memory().expect("store should open");
        let item = pending_sync_item();

        let error = store
            .enqueue_pending_sync(&item, SyncConsent::Denied)
            .expect_err("sync should require consent");

        assert_eq!(error.reason(), LocalStoreErrorReason::ConsentRequired);
        assert_eq!(
            store
                .pending_sync_items()
                .expect("pending sync should be readable")
                .len(),
            0
        );

        store
            .enqueue_pending_sync(&item, SyncConsent::Granted)
            .expect("consented sync should queue");

        let queued = store
            .pending_sync_items()
            .expect("pending sync should be readable");
        assert_eq!(queued, vec![item]);
    }

    #[test]
    fn queues_benchmark_session_sync_from_stored_captures_after_consent() {
        let store = LocalStore::open_in_memory().expect("store should open");
        let request = benchmark_session_sync_request();

        store
            .insert_benchmark_capture(&benchmark_capture(
                "capture:before:001",
                BenchmarkPhase::Before,
            ))
            .expect("before benchmark should be stored");
        store
            .insert_benchmark_capture(&benchmark_capture(
                "capture:after:001",
                BenchmarkPhase::After,
            ))
            .expect("after benchmark should be stored");

        let error = store
            .enqueue_benchmark_session_sync(&request, SyncConsent::Denied)
            .expect_err("benchmark sync should require consent");
        assert_eq!(error.reason(), LocalStoreErrorReason::ConsentRequired);

        let item = store
            .enqueue_benchmark_session_sync(&request, SyncConsent::Granted)
            .expect("consented benchmark session should queue");

        assert_eq!(item.record_kind, BENCHMARK_SESSION_SYNC_RECORD_KIND);
        assert_eq!(item.record_id, request.session_id);
        assert_eq!(item.consent_granted_at_utc, "2026-04-30T12:00:00Z");
        assert!(item
            .payload_json
            .contains("\"schemaVersion\":\"benchmark-session-sync-v1\""));
        assert!(item
            .payload_json
            .contains("\"sessionId\":\"bench:session:pubg:001\""));
        assert!(item.payload_json.contains("\"phase\":\"before\""));
        assert!(item.payload_json.contains("\"phase\":\"after\""));
        assert!(item
            .payload_json
            .contains("\"measurementSource\":\"presentmon-render-present\""));

        let queued = store
            .pending_sync_items()
            .expect("pending sync should be readable");
        assert_eq!(queued, vec![item]);
    }

    #[test]
    fn rejects_invalid_identifiers_payloads_and_metrics() {
        let store = LocalStore::open_in_memory().expect("store should open");
        let mut invalid_snapshot = snapshot();
        invalid_snapshot.id = "bad/id".to_owned();

        let error = store
            .insert_snapshot(&invalid_snapshot)
            .expect_err("slash is not accepted in IDs");
        assert_eq!(error.reason(), LocalStoreErrorReason::InvalidValue);
        assert_eq!(error.field(), Some("id"));

        let mut invalid_audit = audit_event();
        invalid_audit.metadata_json = "not-json".to_owned();

        let error = store
            .insert_audit_event(&invalid_audit)
            .expect_err("metadata must be JSON");
        assert_eq!(error.field(), Some("metadata_json"));

        let mut invalid_capture = benchmark_capture("capture:bad:001", BenchmarkPhase::Single);
        invalid_capture.metrics.average_fps = f64::NAN;

        let error = store
            .insert_benchmark_capture(&invalid_capture)
            .expect_err("metrics must be finite");
        assert_eq!(error.field(), Some("average_fps"));
    }
}
