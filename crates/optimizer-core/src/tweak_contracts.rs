//! Domain contracts for tweak definitions, plans, backups, results, and rollback.

/// Stable identifier for a tweak definition.
pub type TweakId = String;

/// Stable identifier for an optimizer plan.
pub type PlanId = String;

/// Stable identifier for a backup record.
pub type BackupId = String;

/// Supported tweak execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakMode {
    /// Conservative default-eligible behavior with rollback or read-only scope.
    Safe,
    /// User-opt-in behavior that trades comfort, power, or security posture for performance.
    Competitive,
    /// Advanced experiment that needs benchmark framing and explicit per-tweak consent.
    Lab,
    /// A denial guardrail for unsafe optimizer actions.
    Blocked,
}

impl TweakMode {
    /// Returns the stable catalog string for this mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Competitive => "competitive",
            Self::Lab => "lab",
            Self::Blocked => "blocked",
        }
    }

    /// Returns whether this mode requires explicit user opt-in before apply.
    #[must_use]
    pub const fn requires_explicit_opt_in(self) -> bool {
        matches!(self, Self::Competitive | Self::Lab)
    }

    /// Returns whether this mode can be part of the default optimize flow.
    #[must_use]
    pub const fn can_be_default(self) -> bool {
        matches!(self, Self::Safe)
    }
}

/// Product risk level assigned to a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TweakRisk {
    /// Low regression risk and routine rollback.
    Low,
    /// Meaningful side effects are possible but bounded and disclosed.
    Medium,
    /// High regression risk or sensitive compatibility surface.
    High,
    /// Critical security, stability, or anti-cheat risk.
    Critical,
}

impl TweakRisk {
    /// Returns the stable catalog string for this risk level.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Top-level product category for a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakCategory {
    /// Baseline inventory, restore-point, and health checks.
    BaselineHealth,
    /// CPU vendor, topology, scheduler, and platform readiness.
    CpuPlatform,
    /// Windows power-plan and latency-related settings.
    PowerAndLatency,
    /// Windows gaming surfaces such as Game Mode, capture, VRR, and HAGS.
    WindowsGaming,
    /// Security tradeoff and security-preserving performance controls.
    SecurityTradeoff,
    /// Background work, startup, services, and update behavior.
    BackgroundWork,
    /// Storage, filesystem, DirectStorage, and shader cache behavior.
    Storage,
    /// Network adapter and delivery optimization behavior.
    Network,
    /// NVIDIA detection, profile, and rollback behavior.
    Nvidia,
    /// AMD Radeon detection, profile, and rollback behavior.
    Amd,
    /// Intel graphics detection and recommendation behavior.
    IntelGraphics,
    /// PUBG install, config, benchmark, and anti-cheat-safe behavior.
    Pubg,
    /// Benchmark capture, scoring, variance, and telemetry consent behavior.
    Benchmarking,
    /// Explicit denial for unsafe actions.
    BlockedGuardrail,
    /// A catalog category that is not part of the fixed V1 groups yet.
    Other(String),
}

impl TweakCategory {
    /// Returns a stable category label for logs and UI DTOs.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::BaselineHealth => "baseline_health",
            Self::CpuPlatform => "cpu_platform",
            Self::PowerAndLatency => "power_and_latency",
            Self::WindowsGaming => "windows_gaming",
            Self::SecurityTradeoff => "security_tradeoff",
            Self::BackgroundWork => "background_work",
            Self::Storage => "storage",
            Self::Network => "network",
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::IntelGraphics => "intel_graphics",
            Self::Pubg => "pubg",
            Self::Benchmarking => "benchmarking",
            Self::BlockedGuardrail => "blocked_guardrail",
            Self::Other(value) => value.as_str(),
        }
    }
}

/// Whether a tweak requires a reboot or similar restart boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebootPolicy {
    /// No reboot is expected.
    None,
    /// A reboot may improve correctness or measurement quality.
    Recommended,
    /// A reboot is required before the target state can be verified.
    Required,
}

impl RebootPolicy {
    /// Returns the stable catalog string for this reboot policy.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Recommended => "recommended",
            Self::Required => "required",
        }
    }
}

/// Persistence scope for a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionScope {
    /// Persistent system, user, app, or profile state.
    Persistent,
    /// State scoped to a named app, game, driver, or hardware profile.
    ProfileScoped,
    /// Temporary state restored when the session ends.
    SessionOnly,
    /// Advisory/read-only behavior without direct mutation.
    RecommendationOnly,
    /// Denied behavior that must never be applied.
    Blocked,
}

impl SessionScope {
    /// Returns the stable catalog string for this scope.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Persistent => "persistent",
            Self::ProfileScoped => "profile-scoped",
            Self::SessionOnly => "session-only",
            Self::RecommendationOnly => "recommendation-only",
            Self::Blocked => "blocked",
        }
    }
}

/// Rollback strategy attached to a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackKind {
    /// Restore an exact previous setting value.
    ExactValue,
    /// Delete a value that the optimizer created.
    DeleteCreatedValue,
    /// Restore a backed-up file.
    RestoreBackupFile,
    /// Restore a driver or game profile export.
    RestoreProfileExport,
    /// Give the user explicit manual recovery instructions.
    ManualInstructions,
    /// Read-only or recommendation-only tweak; no rollback is needed.
    NotNeededReadonly,
}

impl RollbackKind {
    /// Returns the stable catalog string for this rollback kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactValue => "exact-value",
            Self::DeleteCreatedValue => "delete-created-value",
            Self::RestoreBackupFile => "restore-backup-file",
            Self::RestoreProfileExport => "restore-profile-export",
            Self::ManualInstructions => "manual-instructions",
            Self::NotNeededReadonly => "not-needed-readonly",
        }
    }

    /// Returns whether this rollback kind needs a stored backup before apply.
    #[must_use]
    pub const fn needs_backup_before_apply(self) -> bool {
        !matches!(self, Self::ManualInstructions | Self::NotNeededReadonly)
    }
}

/// Laptop-specific behavior for a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaptopPolicy {
    /// Laptop behavior is the same as desktop behavior.
    SameAsDesktop,
    /// Laptop users must see a warning before apply.
    Warn,
    /// Laptop defaults differ from desktop defaults.
    DifferentDefaults,
    /// The tweak is blocked while the device is on battery.
    BlockedOnBattery,
}

impl LaptopPolicy {
    /// Returns the stable catalog string for this laptop policy.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameAsDesktop => "same-as-desktop",
            Self::Warn => "warn",
            Self::DifferentDefaults => "different-defaults",
            Self::BlockedOnBattery => "blocked-on-battery",
        }
    }
}

/// Power-source applicability for a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSourcePolicy {
    /// The tweak can run on AC or battery.
    Any,
    /// The tweak can run only on AC power.
    AcOnly,
    /// The tweak can run only in battery-safe form on battery power.
    BatterySafeOnly,
}

impl PowerSourcePolicy {
    /// Returns the stable catalog string for this power policy.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::AcOnly => "ac-only",
            Self::BatterySafeOnly => "battery-safe-only",
        }
    }
}

/// Evidence confidence level for a tweak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceLevel {
    /// Vendor, OS, or game documentation supports the behavior.
    Official,
    /// Reputable community research exists but official docs are incomplete.
    CommunityTested,
    /// Plausible but hardware, driver, or version sensitive.
    Experimental,
    /// Must be proven by the app before recommendation.
    InternalBenchmarkRequired,
}

impl EvidenceLevel {
    /// Returns the stable catalog string for this evidence level.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::CommunityTested => "community-tested",
            Self::Experimental => "experimental",
            Self::InternalBenchmarkRequired => "internal-benchmark-required",
        }
    }
}

/// Compatibility surface evaluated before a tweak can be planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityTarget {
    /// Windows build, edition, feature, or policy state.
    Windows,
    /// CPU vendor, generation, topology, or feature state.
    Cpu,
    /// GPU vendor, model, driver, or display pipeline state.
    Gpu,
    /// Storage media, filesystem, or driver state.
    Storage,
    /// Network adapter, driver, or topology state.
    Network,
    /// Power source, battery, laptop, or thermal state.
    Power,
    /// Game, anti-cheat, executable, or profile state.
    Game,
    /// A named driver or application capability.
    Capability,
    /// A custom target reserved for catalog evolution.
    Other(String),
}

/// Rule describing required, supported, or unsupported applicability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityRule {
    /// Compatibility surface being evaluated.
    pub target: CompatibilityTarget,
    /// Machine-readable expression or capability key.
    pub expression: String,
    /// User-facing reason for the rule.
    pub reason: String,
}

/// Source document supporting a tweak decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLink {
    /// Short source title.
    pub title: String,
    /// Source URL or local source identifier.
    pub url: String,
    /// Evidence level supplied by the source.
    pub evidence: EvidenceLevel,
}

/// Metric collected before or after a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementMetric {
    /// Stable metric key, such as `fps.p1` or `gpu_busy.p95`.
    pub key: String,
    /// Human-readable metric label.
    pub label: String,
    /// Unit displayed with the metric.
    pub unit: String,
}

/// Measurement requirements for a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementPlan {
    /// Whether this tweak needs a before/after benchmark before recommendation.
    pub benchmark_required: bool,
    /// Metrics collected for this tweak.
    pub metrics: Vec<MeasurementMetric>,
    /// Notes that explain measurement limits or variance risks.
    pub notes: Vec<String>,
}

/// Expected direction for an impact claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactDirection {
    /// The metric is expected to increase.
    Increase,
    /// The metric is expected to decrease.
    Decrease,
    /// The metric is expected to become more stable.
    Stabilize,
    /// The tweak is informational and has no direct metric target.
    Informational,
}

/// Expected user-visible impact for a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedImpact {
    /// Primary metric or experience dimension.
    pub metric: String,
    /// Expected direction of change.
    pub direction: ImpactDirection,
    /// Confidence in the expected impact.
    pub evidence: EvidenceLevel,
    /// Short user-facing impact summary.
    pub summary: String,
}

/// Type of operation a tweak step may request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakOperationKind {
    /// Read local state only.
    Read,
    /// Create or update local state.
    Write,
    /// Delete local state that the optimizer owns or explicitly backed up.
    Delete,
    /// Ask the user to perform a manual action.
    Manual,
    /// Deny an unsafe requested action.
    Deny,
}

/// Typed operation included in a tweak step or plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakOperation {
    /// Operation kind.
    pub kind: TweakOperationKind,
    /// Stable target key, path-like logical identifier, or capability name.
    pub target: String,
    /// Operation-specific value or command template.
    pub value: Option<String>,
}

/// Named phase of a tweak lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakStepKind {
    /// Detect current applicability and state.
    Detect,
    /// Check blockers before planning or applying.
    Precheck,
    /// Build a dry-run plan.
    Plan,
    /// Capture rollback material.
    Backup,
    /// Apply planned changes.
    Apply,
    /// Verify the target state.
    Verify,
    /// Restore the previous state.
    Rollback,
}

/// Lifecycle step contract for a tweak definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakStep {
    /// Step phase.
    pub kind: TweakStepKind,
    /// Short step summary.
    pub summary: String,
    /// Typed operations requested by this step.
    pub operations: Vec<TweakOperation>,
    /// Whether the step mutates system, profile, or app state.
    pub mutates_system: bool,
}

impl TweakStep {
    /// Creates a read-only step for detection, precheck, planning, or verification.
    #[must_use]
    pub fn read_only(kind: TweakStepKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            operations: Vec::new(),
            mutates_system: false,
        }
    }
}

/// Required test case for a tweak definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakTestCase {
    /// Stable test case key.
    pub id: String,
    /// Behavior covered by the test.
    pub covers: String,
}

/// Test plan attached to a tweak definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakTestPlan {
    /// Unit or integration cases required for the tweak.
    pub cases: Vec<TweakTestCase>,
    /// Fixture identifiers used by the tests.
    pub fixtures: Vec<String>,
    /// Whether this test plan needs an explicitly enabled live Windows mode.
    pub requires_live_windows: bool,
}

/// Complete product contract for a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakDefinition {
    /// Stable tweak ID.
    pub id: TweakId,
    /// User-facing title.
    pub title: String,
    /// User-facing summary.
    pub summary: String,
    /// Product category.
    pub category: TweakCategory,
    /// Execution mode.
    pub mode: TweakMode,
    /// Product risk level.
    pub risk: TweakRisk,
    /// Whether this tweak is enabled by default for eligible users.
    pub default_enabled: bool,
    /// Persistence scope.
    pub session_scope: SessionScope,
    /// Rollback strategy.
    pub rollback_kind: RollbackKind,
    /// Whether the apply path requires administrator privileges.
    pub requires_admin: bool,
    /// Reboot requirement.
    pub reboot: RebootPolicy,
    /// Supported Windows versions or build expressions.
    pub supported_os: Vec<String>,
    /// Supported hardware rules.
    pub supported_hardware: Vec<CompatibilityRule>,
    /// Supported driver rules.
    pub supported_drivers: Vec<CompatibilityRule>,
    /// Rules that block planning when matched.
    pub unsupported_when: Vec<CompatibilityRule>,
    /// Tweak IDs that conflict with this definition.
    pub conflicts_with: Vec<TweakId>,
    /// Laptop-specific policy.
    pub laptop_policy: LaptopPolicy,
    /// Power-source policy.
    pub power_source_policy: PowerSourcePolicy,
    /// Source links supporting this tweak.
    pub source_links: Vec<SourceLink>,
    /// Evidence level used for product promotion decisions.
    pub evidence_level: EvidenceLevel,
    /// Measurement requirements.
    pub measurement_plan: MeasurementPlan,
    /// Expected impact summary.
    pub expected_impact: ExpectedImpact,
    /// Known side effects shown before apply.
    pub known_side_effects: Vec<String>,
    /// Anti-cheat notes or confirmations.
    pub anti_cheat_notes: Vec<String>,
    /// Whether a game process must be closed before apply.
    pub game_closed_required: bool,
    /// User-facing disclosure shown before apply.
    pub user_disclosure: String,
    /// Recommended operator/user actions.
    pub r#do: Vec<String>,
    /// Explicitly prohibited actions.
    pub dont: Vec<String>,
    /// Read-only detection step.
    pub detect: TweakStep,
    /// Read-only precheck step.
    pub precheck: TweakStep,
    /// Dry-run plan step.
    pub plan: TweakStep,
    /// Backup step.
    pub backup: TweakStep,
    /// Apply step.
    pub apply: TweakStep,
    /// Verification step.
    pub verify: TweakStep,
    /// Rollback step.
    pub rollback: TweakStep,
    /// Required tests.
    pub tests: TweakTestPlan,
}

impl TweakDefinition {
    /// Returns whether the definition is an unsafe-action guardrail.
    #[must_use]
    pub const fn is_blocked_guardrail(&self) -> bool {
        matches!(self.mode, TweakMode::Blocked) || matches!(self.session_scope, SessionScope::Blocked)
    }

    /// Returns whether the apply step mutates state.
    #[must_use]
    pub const fn is_mutable(&self) -> bool {
        self.apply.mutates_system
    }

    /// Returns whether this definition can participate in the default optimize flow.
    #[must_use]
    pub fn is_default_candidate(&self) -> bool {
        self.default_enabled
            && self.mode.can_be_default()
            && self.risk <= TweakRisk::Medium
            && !self.is_blocked_guardrail()
    }
}

/// Action chosen for a tweak inside a dry-run plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanAction {
    /// Detect only, with no recommendation or mutation.
    DetectOnly,
    /// Recommend a user decision but do not mutate automatically.
    Recommend,
    /// Apply a planned change after precheck and backup.
    Apply,
    /// Deny an unsafe or unsupported action.
    Deny,
}

/// Planned state change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedChange {
    /// Logical target affected by the plan.
    pub target: String,
    /// Planned operation.
    pub operation: TweakOperationKind,
    /// Previously detected value when known.
    pub previous_value: Option<String>,
    /// Desired value when applicable.
    pub desired_value: Option<String>,
    /// Persistence scope for the change.
    pub scope: SessionScope,
}

/// Backup requirement attached to a plan item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupRequirement {
    /// No backup is needed because the item is read-only or recommendation-only.
    NotRequired,
    /// Backup must complete before apply can run.
    Required {
        /// Rollback strategy that determines the backup shape.
        kind: RollbackKind,
        /// Human-readable backup target.
        target: String,
    },
}

/// One planned tweak decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakPlanItem {
    /// Planned tweak ID.
    pub tweak_id: TweakId,
    /// Product category.
    pub category: TweakCategory,
    /// Planned action.
    pub action: PlanAction,
    /// Tweak mode.
    pub mode: TweakMode,
    /// Risk level.
    pub risk: TweakRisk,
    /// Planned changes or denial targets.
    pub changes: Vec<PlannedChange>,
    /// Backup requirement before apply.
    pub backup: BackupRequirement,
    /// Rollback plan for this item.
    pub rollback: RollbackPlan,
    /// Reboot requirement.
    pub reboot: RebootPolicy,
    /// Whether admin is required.
    pub requires_admin: bool,
    /// Warnings shown to the user.
    pub warnings: Vec<String>,
}

/// Dry-run plan produced from tweak definitions and local state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakPlan {
    /// Stable plan ID.
    pub id: PlanId,
    /// Requested optimization mode.
    pub requested_mode: TweakMode,
    /// Catalog schema version used to build the plan.
    pub catalog_schema_version: String,
    /// Plan items.
    pub items: Vec<TweakPlanItem>,
    /// Plan-level warnings or blockers.
    pub warnings: Vec<String>,
}

impl TweakPlan {
    /// Returns whether this plan contains at least one apply action.
    #[must_use]
    pub fn has_apply_items(&self) -> bool {
        self.items.iter().any(|item| item.action == PlanAction::Apply)
    }

    /// Returns whether any planned item is denied.
    #[must_use]
    pub fn has_denials(&self) -> bool {
        self.items.iter().any(|item| item.action == PlanAction::Deny)
    }
}

/// Backup payload captured before apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupPayload {
    /// Exact previous value for a logical target.
    ExactValue {
        /// Backed-up target.
        target: String,
        /// Previous value.
        value: String,
    },
    /// Exact previous values for a group of logical targets.
    ExactValues {
        /// Backed-up target/value pairs.
        values: Vec<BackupExactValue>,
    },
    /// Marker for a value created by the optimizer.
    CreatedValue {
        /// Created target.
        target: String,
    },
    /// Backed-up file metadata.
    FileSnapshot {
        /// Source file path or logical file key.
        path: String,
        /// Integrity hash for the backup bytes.
        content_hash: String,
    },
    /// Driver or game profile export metadata.
    ProfileExport {
        /// Profile name or ID.
        profile_id: String,
        /// Export file path or protected storage key.
        export_ref: String,
    },
    /// Manual recovery text.
    ManualInstructions {
        /// Recovery instructions.
        instructions: String,
    },
    /// Read-only item with no backup material.
    ReadOnly,
}

/// One exact backed-up value within a grouped backup payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupExactValue {
    /// Backed-up target.
    pub target: String,
    /// Previous value.
    pub value: String,
}

/// Backup record persisted before apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRecord {
    /// Backup ID.
    pub id: BackupId,
    /// Tweak ID that produced the backup.
    pub tweak_id: TweakId,
    /// Rollback strategy for this record.
    pub rollback_kind: RollbackKind,
    /// Catalog version that produced the record.
    pub catalog_schema_version: String,
    /// UTC timestamp string generated by the persistence layer.
    pub created_at_utc: String,
    /// Captured backup payload.
    pub payload: BackupPayload,
}

/// One rollback operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackStep {
    /// Step summary.
    pub summary: String,
    /// Logical target to restore.
    pub target: String,
    /// Restore operation.
    pub operation: TweakOperationKind,
    /// Expected restored state.
    pub expected_state: Option<String>,
}

/// Rollback plan for a planned or applied tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackPlan {
    /// Rollback strategy.
    pub kind: RollbackKind,
    /// Steps required to roll back.
    pub steps: Vec<RollbackStep>,
    /// Whether rollback needs administrator privileges.
    pub requires_admin: bool,
    /// Reboot requirement after rollback.
    pub reboot: RebootPolicy,
    /// Manual fallback instructions.
    pub manual_instructions: Vec<String>,
}

impl RollbackPlan {
    /// Creates a no-op rollback plan for read-only or recommendation-only items.
    #[must_use]
    pub const fn not_needed() -> Self {
        Self {
            kind: RollbackKind::NotNeededReadonly,
            steps: Vec::new(),
            requires_admin: false,
            reboot: RebootPolicy::None,
            manual_instructions: Vec::new(),
        }
    }
}

/// Verification outcome after apply or rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationOutcome {
    /// Verification has not run yet.
    NotRun,
    /// Verification proved the expected state.
    Succeeded,
    /// Verification failed and rollback should be considered.
    Failed {
        /// Failure reason.
        reason: String,
    },
    /// Verification cannot prove state and explains the limitation.
    Impossible {
        /// Limitation reason.
        reason: String,
    },
}

impl VerificationOutcome {
    /// Returns whether this outcome requires rollback consideration.
    #[must_use]
    pub const fn rollback_required(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Final status of a tweak execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakResultStatus {
    /// Planned but not executed.
    Planned,
    /// Skipped because the local state was already acceptable or user declined.
    Skipped,
    /// Applied but not yet verified.
    Applied,
    /// Applied and verified.
    Verified,
    /// Verification failed and rollback is required.
    RollbackRequired,
    /// Rollback completed.
    RolledBack,
    /// Denied by policy.
    Denied,
    /// Failed before completion.
    Failed,
}

/// Execution result for one tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakResult {
    /// Tweak ID.
    pub tweak_id: TweakId,
    /// Result status.
    pub status: TweakResultStatus,
    /// Verification outcome.
    pub verification: VerificationOutcome,
    /// Backup used for rollback when applicable.
    pub backup_id: Option<BackupId>,
    /// Rollback plan to use when verification fails.
    pub rollback: Option<RollbackPlan>,
    /// User-facing and audit-facing messages.
    pub messages: Vec<String>,
}

impl TweakResult {
    /// Builds a result from a verification outcome.
    #[must_use]
    pub fn from_verification(
        tweak_id: impl Into<String>,
        verification: VerificationOutcome,
        backup_id: Option<BackupId>,
        rollback: Option<RollbackPlan>,
    ) -> Self {
        let status = if verification.rollback_required() {
            TweakResultStatus::RollbackRequired
        } else {
            TweakResultStatus::Verified
        };

        Self {
            tweak_id: tweak_id.into(),
            status,
            verification,
            backup_id,
            rollback,
            messages: Vec::new(),
        }
    }
}

/// Rollback execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackStatus {
    /// Rollback was not needed.
    NotNeeded,
    /// All targets were restored.
    Restored,
    /// Some targets were restored and others need attention.
    PartiallyRestored,
    /// User must follow manual recovery instructions.
    ManualRequired,
    /// Rollback failed.
    Failed,
}

/// Result of a rollback attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackResult {
    /// Backup used for rollback.
    pub backup_id: Option<BackupId>,
    /// Rollback status.
    pub status: RollbackStatus,
    /// Targets restored by this rollback.
    pub restored_targets: Vec<String>,
    /// Messages explaining residual work or failures.
    pub messages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_measurement_plan() -> MeasurementPlan {
        MeasurementPlan {
            benchmark_required: false,
            metrics: vec![MeasurementMetric {
                key: "fps.p1".to_owned(),
                label: "1% low FPS".to_owned(),
                unit: "fps".to_owned(),
            }],
            notes: vec!["Compare against same map and driver metadata.".to_owned()],
        }
    }

    fn sample_impact() -> ExpectedImpact {
        ExpectedImpact {
            metric: "frametime consistency".to_owned(),
            direction: ImpactDirection::Stabilize,
            evidence: EvidenceLevel::Official,
            summary: "Keeps the default path measurable without promising FPS.".to_owned(),
        }
    }

    fn sample_test_plan() -> TweakTestPlan {
        TweakTestPlan {
            cases: vec![TweakTestCase {
                id: "parses-definition".to_owned(),
                covers: "definition is complete".to_owned(),
            }],
            fixtures: Vec::new(),
            requires_live_windows: false,
        }
    }

    fn sample_definition() -> TweakDefinition {
        TweakDefinition {
            id: "sys.scan.inventory".to_owned(),
            title: "System inventory scan".to_owned(),
            summary: "Reads system state before planning tweaks.".to_owned(),
            category: TweakCategory::BaselineHealth,
            mode: TweakMode::Safe,
            risk: TweakRisk::Low,
            default_enabled: true,
            session_scope: SessionScope::RecommendationOnly,
            rollback_kind: RollbackKind::NotNeededReadonly,
            requires_admin: false,
            reboot: RebootPolicy::None,
            supported_os: vec!["windows-10+".to_owned()],
            supported_hardware: Vec::new(),
            supported_drivers: Vec::new(),
            unsupported_when: Vec::new(),
            conflicts_with: Vec::new(),
            laptop_policy: LaptopPolicy::SameAsDesktop,
            power_source_policy: PowerSourcePolicy::Any,
            source_links: vec![SourceLink {
                title: "Windows system APIs".to_owned(),
                url: "local:windows-api".to_owned(),
                evidence: EvidenceLevel::Official,
            }],
            evidence_level: EvidenceLevel::Official,
            measurement_plan: sample_measurement_plan(),
            expected_impact: sample_impact(),
            known_side_effects: Vec::new(),
            anti_cheat_notes: vec!["Read-only; no game or anti-cheat mutation.".to_owned()],
            game_closed_required: false,
            user_disclosure: "Inventory is read-only.".to_owned(),
            r#do: vec!["Read current system state.".to_owned()],
            dont: vec!["Do not mutate state during scan.".to_owned()],
            detect: TweakStep::read_only(TweakStepKind::Detect, "Read system inventory."),
            precheck: TweakStep::read_only(TweakStepKind::Precheck, "Check API availability."),
            plan: TweakStep::read_only(TweakStepKind::Plan, "Build a read-only finding."),
            backup: TweakStep::read_only(TweakStepKind::Backup, "No backup required."),
            apply: TweakStep::read_only(TweakStepKind::Apply, "No apply operation."),
            verify: TweakStep::read_only(TweakStepKind::Verify, "Confirm scan completed."),
            rollback: TweakStep::read_only(TweakStepKind::Rollback, "No rollback required."),
            tests: sample_test_plan(),
        }
    }

    #[test]
    fn exposes_stable_mode_and_risk_strings() {
        assert_eq!(TweakMode::Safe.as_str(), "safe");
        assert_eq!(TweakMode::Competitive.as_str(), "competitive");
        assert!(TweakMode::Lab.requires_explicit_opt_in());
        assert!(!TweakMode::Blocked.can_be_default());

        assert_eq!(TweakRisk::Critical.as_str(), "critical");
        assert!(TweakRisk::High > TweakRisk::Medium);
    }

    #[test]
    fn defines_complete_read_only_tweak_definition() {
        let definition = sample_definition();

        assert_eq!(definition.id, "sys.scan.inventory");
        assert_eq!(definition.category.as_str(), "baseline_health");
        assert_eq!(definition.rollback_kind, RollbackKind::NotNeededReadonly);
        assert!(!definition.is_mutable());
        assert!(definition.is_default_candidate());
    }

    #[test]
    fn blocked_guardrail_is_not_default_candidate() {
        let mut definition = sample_definition();
        definition.id = "blocked.defender.disable".to_owned();
        definition.category = TweakCategory::BlockedGuardrail;
        definition.mode = TweakMode::Blocked;
        definition.risk = TweakRisk::Critical;
        definition.session_scope = SessionScope::Blocked;
        definition.default_enabled = false;

        assert!(definition.is_blocked_guardrail());
        assert!(!definition.is_default_candidate());
    }

    #[test]
    fn plan_tracks_apply_items_and_denials() {
        let rollback = RollbackPlan {
            kind: RollbackKind::ExactValue,
            steps: vec![RollbackStep {
                summary: "Restore previous registry value.".to_owned(),
                target: "hkcu:gamebar".to_owned(),
                operation: TweakOperationKind::Write,
                expected_state: Some("previous".to_owned()),
            }],
            requires_admin: false,
            reboot: RebootPolicy::None,
            manual_instructions: Vec::new(),
        };

        let plan = TweakPlan {
            id: "plan-001".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![
                TweakPlanItem {
                    tweak_id: "game.capture.background.off".to_owned(),
                    category: TweakCategory::WindowsGaming,
                    action: PlanAction::Apply,
                    mode: TweakMode::Safe,
                    risk: TweakRisk::Low,
                    changes: vec![PlannedChange {
                        target: "hkcu:gamebar-background-capture".to_owned(),
                        operation: TweakOperationKind::Write,
                        previous_value: Some("1".to_owned()),
                        desired_value: Some("0".to_owned()),
                        scope: SessionScope::Persistent,
                    }],
                    backup: BackupRequirement::Required {
                        kind: RollbackKind::ExactValue,
                        target: "hkcu:gamebar-background-capture".to_owned(),
                    },
                    rollback,
                    reboot: RebootPolicy::None,
                    requires_admin: false,
                    warnings: Vec::new(),
                },
                TweakPlanItem {
                    tweak_id: "blocked.defender.disable".to_owned(),
                    category: TweakCategory::BlockedGuardrail,
                    action: PlanAction::Deny,
                    mode: TweakMode::Blocked,
                    risk: TweakRisk::Critical,
                    changes: Vec::new(),
                    backup: BackupRequirement::NotRequired,
                    rollback: RollbackPlan::not_needed(),
                    reboot: RebootPolicy::None,
                    requires_admin: false,
                    warnings: vec!["Global Defender disable is denied.".to_owned()],
                },
            ],
            warnings: Vec::new(),
        };

        assert!(plan.has_apply_items());
        assert!(plan.has_denials());
    }

    #[test]
    fn verification_failure_marks_rollback_required() {
        let result = TweakResult::from_verification(
            "game.capture.background.off",
            VerificationOutcome::Failed {
                reason: "registry readback stayed enabled".to_owned(),
            },
            Some("backup-001".to_owned()),
            Some(RollbackPlan {
                kind: RollbackKind::ExactValue,
                steps: Vec::new(),
                requires_admin: false,
                reboot: RebootPolicy::None,
                manual_instructions: Vec::new(),
            }),
        );

        assert_eq!(result.status, TweakResultStatus::RollbackRequired);
        assert!(result.verification.rollback_required());
        assert_eq!(result.backup_id.as_deref(), Some("backup-001"));
    }
}
