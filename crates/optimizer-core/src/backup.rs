//! Backup capture and rollback execution interfaces.

use std::fmt;

use crate::tweak_contracts::{
    BackupRecord, BackupRequirement, PlanAction, PlannedChange, RollbackKind, RollbackPlan,
    RollbackResult, TweakId, TweakPlan, TweakPlanItem,
};

/// Request passed to a platform adapter before a mutable apply step runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupCaptureRequest {
    /// Plan that requested the backup.
    pub plan_id: String,
    /// Catalog schema version used to build the plan.
    pub catalog_schema_version: String,
    /// Tweak that owns the backup.
    pub tweak_id: TweakId,
    /// Rollback strategy that determines the backup payload shape.
    pub kind: RollbackKind,
    /// Primary logical target being backed up.
    pub target: String,
    /// Planned changes that will run only after backup succeeds.
    pub changes: Vec<PlannedChange>,
}

impl BackupCaptureRequest {
    /// Creates a validated backup capture request for a rollback-capable tweak.
    pub fn new(
        plan_id: impl Into<String>,
        catalog_schema_version: impl Into<String>,
        tweak_id: impl Into<String>,
        kind: RollbackKind,
        target: impl Into<String>,
        changes: Vec<PlannedChange>,
    ) -> Result<Self, BackupError> {
        let plan_id = plan_id.into();
        let catalog_schema_version = catalog_schema_version.into();
        let tweak_id = tweak_id.into();
        let target = target.into();

        if plan_id.trim().is_empty()
            || catalog_schema_version.trim().is_empty()
            || tweak_id.trim().is_empty()
            || target.trim().is_empty()
        {
            return Err(BackupError::invalid_request(
                tweak_id,
                "backup request identifiers and target must be non-empty",
            ));
        }

        if !kind.needs_backup_before_apply() {
            return Err(BackupError::backup_not_required(tweak_id, kind));
        }

        Ok(Self {
            plan_id,
            catalog_schema_version,
            tweak_id,
            kind,
            target,
            changes,
        })
    }

    /// Builds a backup request from a planned apply item.
    pub fn from_plan_item(
        plan: &TweakPlan,
        item: &TweakPlanItem,
    ) -> Result<Option<Self>, BackupError> {
        if item.action != PlanAction::Apply {
            return Ok(None);
        }

        match &item.backup {
            BackupRequirement::Required { kind, target } => {
                if *kind != item.rollback.kind {
                    return Err(BackupError::rollback_kind_mismatch(
                        item.tweak_id.clone(),
                        *kind,
                        item.rollback.kind,
                    ));
                }

                Self::new(
                    plan.id.clone(),
                    plan.catalog_schema_version.clone(),
                    item.tweak_id.clone(),
                    *kind,
                    target.clone(),
                    item.changes.clone(),
                )
                .map(Some)
            }
            BackupRequirement::NotRequired if item.rollback.kind.needs_backup_before_apply() => {
                Err(BackupError::missing_required_backup(
                    item.tweak_id.clone(),
                    item.rollback.kind,
                ))
            }
            BackupRequirement::NotRequired => Ok(None),
        }
    }
}

/// Platform boundary that captures rollback material before apply.
pub trait BackupAdapter {
    /// Captures and persists rollback material for the requested target.
    fn capture_backup(
        &mut self,
        request: &BackupCaptureRequest,
    ) -> Result<BackupRecord, BackupError>;
}

/// Platform boundary that restores a previously captured backup.
pub trait RollbackAdapter {
    /// Executes rollback using the supplied backup and rollback plan.
    fn execute_rollback(
        &mut self,
        request: &RollbackRequest,
    ) -> Result<RollbackResult, RollbackError>;
}

/// Captures backups for every apply item in a plan that requires one.
pub fn capture_plan_backups<A>(
    plan: &TweakPlan,
    adapter: &mut A,
) -> Result<Vec<BackupRecord>, BackupError>
where
    A: BackupAdapter,
{
    let mut records = Vec::new();

    for item in &plan.items {
        let Some(request) = BackupCaptureRequest::from_plan_item(plan, item)? else {
            continue;
        };

        let record = adapter.capture_backup(&request)?;
        validate_backup_record(&request, &record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_backup_record(
    request: &BackupCaptureRequest,
    record: &BackupRecord,
) -> Result<(), BackupError> {
    if record.tweak_id.as_str() != request.tweak_id.as_str() {
        return Err(BackupError::invalid_backup_record(
            request.tweak_id.clone(),
            "adapter returned a backup for a different tweak",
        ));
    }

    if record.rollback_kind != request.kind {
        return Err(BackupError::invalid_backup_record(
            request.tweak_id.clone(),
            "adapter returned a backup with the wrong rollback kind",
        ));
    }

    if record.catalog_schema_version.as_str() != request.catalog_schema_version.as_str() {
        return Err(BackupError::invalid_backup_record(
            request.tweak_id.clone(),
            "adapter returned a backup for a different catalog schema",
        ));
    }

    Ok(())
}

/// Request passed to a platform adapter when rollback is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRequest {
    /// Tweak being rolled back.
    pub tweak_id: TweakId,
    /// Backup material captured before apply.
    pub backup: BackupRecord,
    /// Rollback plan produced during dry-run planning.
    pub plan: RollbackPlan,
}

impl RollbackRequest {
    /// Creates a rollback request and validates that backup and plan match.
    pub fn new(
        tweak_id: impl Into<String>,
        backup: BackupRecord,
        plan: RollbackPlan,
    ) -> Result<Self, RollbackError> {
        let tweak_id = tweak_id.into();

        if tweak_id.trim().is_empty() {
            return Err(RollbackError::invalid_request(
                tweak_id,
                "rollback tweak ID must be non-empty",
            ));
        }

        if backup.tweak_id.as_str() != tweak_id.as_str() {
            return Err(RollbackError::backup_plan_mismatch(
                tweak_id,
                "backup belongs to a different tweak",
            ));
        }

        if backup.rollback_kind != plan.kind {
            return Err(RollbackError::backup_plan_mismatch(
                tweak_id,
                "backup rollback kind does not match rollback plan",
            ));
        }

        if plan.kind.needs_backup_before_apply() && plan.steps.is_empty() {
            return Err(RollbackError::invalid_request(
                tweak_id,
                "rollback plan must include at least one restore step",
            ));
        }

        if plan.kind == RollbackKind::ManualInstructions
            && plan.steps.is_empty()
            && plan.manual_instructions.is_empty()
        {
            return Err(RollbackError::invalid_request(
                tweak_id,
                "manual rollback requires user-facing instructions",
            ));
        }

        Ok(Self {
            tweak_id,
            backup,
            plan,
        })
    }
}

/// Runs a rollback request through the supplied platform adapter.
pub fn execute_rollback<A>(
    adapter: &mut A,
    request: &RollbackRequest,
) -> Result<RollbackResult, RollbackError>
where
    A: RollbackAdapter,
{
    adapter.execute_rollback(request)
}

/// Reason a backup capture request failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupErrorReason {
    /// Backup request did not include the required identifiers or target.
    InvalidRequest,
    /// The rollback strategy does not require captured backup material.
    BackupNotRequired,
    /// An apply item was missing a required backup contract.
    MissingRequiredBackup,
    /// Backup and rollback strategies did not agree.
    RollbackKindMismatch,
    /// Platform adapter does not support the rollback kind yet.
    UnsupportedRollbackKind,
    /// Platform adapter failed while capturing backup material.
    CaptureFailed,
    /// Platform adapter returned a malformed backup record.
    InvalidBackupRecord,
}

impl BackupErrorReason {
    /// Returns a stable reason string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::BackupNotRequired => "backup_not_required",
            Self::MissingRequiredBackup => "missing_required_backup",
            Self::RollbackKindMismatch => "rollback_kind_mismatch",
            Self::UnsupportedRollbackKind => "unsupported_rollback_kind",
            Self::CaptureFailed => "capture_failed",
            Self::InvalidBackupRecord => "invalid_backup_record",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidRequest => "Backup request failed validation",
            Self::BackupNotRequired => "Rollback kind does not require backup capture",
            Self::MissingRequiredBackup => "Apply item is missing a required backup",
            Self::RollbackKindMismatch => "Backup and rollback kinds do not match",
            Self::UnsupportedRollbackKind => "Rollback kind is not supported by this adapter",
            Self::CaptureFailed => "Backup adapter failed to capture rollback material",
            Self::InvalidBackupRecord => "Backup adapter returned an invalid record",
        }
    }
}

/// Structured backup capture error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupError {
    reason: BackupErrorReason,
    tweak_id: Option<TweakId>,
    detail: Option<String>,
}

impl BackupError {
    fn new(reason: BackupErrorReason, tweak_id: Option<TweakId>, detail: Option<String>) -> Self {
        Self {
            reason,
            tweak_id,
            detail,
        }
    }

    /// Creates an invalid request error.
    #[must_use]
    pub fn invalid_request(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            BackupErrorReason::InvalidRequest,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Creates an error for a rollback kind that needs no backup.
    #[must_use]
    pub fn backup_not_required(tweak_id: impl Into<String>, kind: RollbackKind) -> Self {
        Self::new(
            BackupErrorReason::BackupNotRequired,
            Some(tweak_id.into()),
            Some(kind.as_str().to_owned()),
        )
    }

    /// Creates an error for a planned apply item missing backup material.
    #[must_use]
    pub fn missing_required_backup(tweak_id: impl Into<String>, kind: RollbackKind) -> Self {
        Self::new(
            BackupErrorReason::MissingRequiredBackup,
            Some(tweak_id.into()),
            Some(kind.as_str().to_owned()),
        )
    }

    /// Creates an error when the backup and rollback contracts disagree.
    #[must_use]
    pub fn rollback_kind_mismatch(
        tweak_id: impl Into<String>,
        backup_kind: RollbackKind,
        rollback_kind: RollbackKind,
    ) -> Self {
        Self::new(
            BackupErrorReason::RollbackKindMismatch,
            Some(tweak_id.into()),
            Some(format!(
                "backup={}, rollback={}",
                backup_kind.as_str(),
                rollback_kind.as_str()
            )),
        )
    }

    /// Creates an adapter unsupported-kind error.
    #[must_use]
    pub fn unsupported_rollback_kind(tweak_id: impl Into<String>, kind: RollbackKind) -> Self {
        Self::new(
            BackupErrorReason::UnsupportedRollbackKind,
            Some(tweak_id.into()),
            Some(kind.as_str().to_owned()),
        )
    }

    /// Creates an adapter capture failure error.
    #[must_use]
    pub fn capture_failed(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            BackupErrorReason::CaptureFailed,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Creates an invalid backup record error.
    #[must_use]
    pub fn invalid_backup_record(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            BackupErrorReason::InvalidBackupRecord,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Returns the backup failure reason.
    #[must_use]
    pub const fn reason(&self) -> BackupErrorReason {
        self.reason
    }

    /// Returns the associated tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns extra error detail, when known.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for BackupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, " [{detail}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for BackupError {}

/// Reason a rollback request failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackErrorReason {
    /// Rollback request did not include the required identifiers or plan shape.
    InvalidRequest,
    /// Backup record and rollback plan do not describe the same restore path.
    BackupPlanMismatch,
    /// Platform adapter does not support the rollback kind yet.
    UnsupportedRollbackKind,
    /// Platform adapter failed while restoring backup material.
    RestoreFailed,
}

impl RollbackErrorReason {
    /// Returns a stable reason string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::BackupPlanMismatch => "backup_plan_mismatch",
            Self::UnsupportedRollbackKind => "unsupported_rollback_kind",
            Self::RestoreFailed => "restore_failed",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidRequest => "Rollback request failed validation",
            Self::BackupPlanMismatch => "Backup record does not match rollback plan",
            Self::UnsupportedRollbackKind => "Rollback kind is not supported by this adapter",
            Self::RestoreFailed => "Rollback adapter failed to restore backup material",
        }
    }
}

/// Structured rollback execution error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackError {
    reason: RollbackErrorReason,
    tweak_id: Option<TweakId>,
    detail: Option<String>,
}

impl RollbackError {
    fn new(reason: RollbackErrorReason, tweak_id: Option<TweakId>, detail: Option<String>) -> Self {
        Self {
            reason,
            tweak_id,
            detail,
        }
    }

    /// Creates an invalid rollback request error.
    #[must_use]
    pub fn invalid_request(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            RollbackErrorReason::InvalidRequest,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Creates a backup/plan mismatch error.
    #[must_use]
    pub fn backup_plan_mismatch(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            RollbackErrorReason::BackupPlanMismatch,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Creates an adapter unsupported-kind error.
    #[must_use]
    pub fn unsupported_rollback_kind(tweak_id: impl Into<String>, kind: RollbackKind) -> Self {
        Self::new(
            RollbackErrorReason::UnsupportedRollbackKind,
            Some(tweak_id.into()),
            Some(kind.as_str().to_owned()),
        )
    }

    /// Creates an adapter restore failure error.
    #[must_use]
    pub fn restore_failed(tweak_id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            RollbackErrorReason::RestoreFailed,
            Some(tweak_id.into()),
            Some(detail.into()),
        )
    }

    /// Returns the rollback failure reason.
    #[must_use]
    pub const fn reason(&self) -> RollbackErrorReason {
        self.reason
    }

    /// Returns the associated tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns extra error detail, when known.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for RollbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, " [{detail}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for RollbackError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweak_contracts::{
        BackupPayload, RebootPolicy, RollbackStep, RollbackStatus, SessionScope,
        TweakOperationKind,
    };

    #[derive(Debug, Default)]
    struct FixtureBackupAdapter {
        value: String,
    }

    impl BackupAdapter for FixtureBackupAdapter {
        fn capture_backup(
            &mut self,
            request: &BackupCaptureRequest,
        ) -> Result<BackupRecord, BackupError> {
            Ok(BackupRecord {
                id: format!("backup:{}", request.tweak_id),
                tweak_id: request.tweak_id.clone(),
                rollback_kind: request.kind,
                catalog_schema_version: request.catalog_schema_version.clone(),
                created_at_utc: "2026-01-01T00:00:00Z".to_owned(),
                payload: BackupPayload::ExactValue {
                    target: request.target.clone(),
                    value: self.value.clone(),
                },
            })
        }
    }

    #[derive(Debug, Default)]
    struct FixtureRollbackAdapter;

    impl RollbackAdapter for FixtureRollbackAdapter {
        fn execute_rollback(
            &mut self,
            request: &RollbackRequest,
        ) -> Result<RollbackResult, RollbackError> {
            Ok(RollbackResult {
                backup_id: Some(request.backup.id.clone()),
                status: RollbackStatus::Restored,
                restored_targets: request
                    .plan
                    .steps
                    .iter()
                    .map(|step| step.target.clone())
                    .collect(),
                messages: Vec::new(),
            })
        }
    }

    fn planned_change() -> PlannedChange {
        PlannedChange {
            target: "registry:hkcu/gamebar/capture".to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: Some("1".to_owned()),
            desired_value: Some("0".to_owned()),
            scope: SessionScope::Persistent,
        }
    }

    fn rollback_plan() -> RollbackPlan {
        RollbackPlan {
            kind: RollbackKind::ExactValue,
            steps: vec![RollbackStep {
                summary: "Restore previous capture setting.".to_owned(),
                target: "registry:hkcu/gamebar/capture".to_owned(),
                operation: TweakOperationKind::Write,
                expected_state: Some("1".to_owned()),
            }],
            requires_admin: false,
            reboot: RebootPolicy::None,
            manual_instructions: Vec::new(),
        }
    }

    fn plan_item() -> TweakPlanItem {
        TweakPlanItem {
            tweak_id: "game.capture.background.off".to_owned(),
            category: crate::tweak_contracts::TweakCategory::WindowsGaming,
            action: PlanAction::Apply,
            mode: crate::tweak_contracts::TweakMode::Safe,
            risk: crate::tweak_contracts::TweakRisk::Low,
            changes: vec![planned_change()],
            backup: BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: "registry:hkcu/gamebar/capture".to_owned(),
            },
            rollback: rollback_plan(),
            reboot: RebootPolicy::None,
            requires_admin: false,
            warnings: Vec::new(),
        }
    }

    fn plan_with_item(item: TweakPlanItem) -> TweakPlan {
        TweakPlan {
            id: "plan-001".to_owned(),
            requested_mode: crate::tweak_contracts::TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![item],
            warnings: Vec::new(),
        }
    }

    #[test]
    fn captures_required_backup_before_apply_items() {
        let plan = plan_with_item(plan_item());
        let mut adapter = FixtureBackupAdapter {
            value: "1".to_owned(),
        };

        let records = capture_plan_backups(&plan, &mut adapter)
            .expect("required backup should be captured");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tweak_id, "game.capture.background.off");
        assert_eq!(records[0].rollback_kind, RollbackKind::ExactValue);
        assert_eq!(records[0].catalog_schema_version, "1");
    }

    #[test]
    fn rejects_apply_item_that_lost_required_backup_contract() {
        let mut item = plan_item();
        item.backup = BackupRequirement::NotRequired;
        let plan = plan_with_item(item);
        let mut adapter = FixtureBackupAdapter::default();

        let error = capture_plan_backups(&plan, &mut adapter)
            .expect_err("apply item should require backup before apply");

        assert_eq!(error.reason(), BackupErrorReason::MissingRequiredBackup);
        assert_eq!(error.tweak_id(), Some("game.capture.background.off"));
    }

    #[test]
    fn rollback_request_rejects_kind_mismatch() {
        let backup = BackupRecord {
            id: "backup-001".to_owned(),
            tweak_id: "game.capture.background.off".to_owned(),
            rollback_kind: RollbackKind::DeleteCreatedValue,
            catalog_schema_version: "1".to_owned(),
            created_at_utc: "2026-01-01T00:00:00Z".to_owned(),
            payload: BackupPayload::CreatedValue {
                target: "registry:hkcu/gamebar/capture".to_owned(),
            },
        };

        let error = RollbackRequest::new(
            "game.capture.background.off",
            backup,
            rollback_plan(),
        )
        .expect_err("rollback kind mismatch should be denied");

        assert_eq!(error.reason(), RollbackErrorReason::BackupPlanMismatch);
    }

    #[test]
    fn delegates_valid_rollback_request_to_adapter() {
        let backup = BackupRecord {
            id: "backup-001".to_owned(),
            tweak_id: "game.capture.background.off".to_owned(),
            rollback_kind: RollbackKind::ExactValue,
            catalog_schema_version: "1".to_owned(),
            created_at_utc: "2026-01-01T00:00:00Z".to_owned(),
            payload: BackupPayload::ExactValue {
                target: "registry:hkcu/gamebar/capture".to_owned(),
                value: "1".to_owned(),
            },
        };
        let request = RollbackRequest::new(
            "game.capture.background.off",
            backup,
            rollback_plan(),
        )
        .expect("rollback request should be valid");
        let mut adapter = FixtureRollbackAdapter;

        let result = execute_rollback(&mut adapter, &request)
            .expect("rollback adapter should restore");

        assert_eq!(result.status, RollbackStatus::Restored);
        assert_eq!(
            result.restored_targets,
            vec!["registry:hkcu/gamebar/capture".to_owned()]
        );
    }
}
