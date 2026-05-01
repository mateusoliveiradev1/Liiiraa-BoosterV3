//! Windows rollback fixture adapter for backup and restore contracts.

use std::collections::BTreeMap;

use optimizer_core::{
    backup::{
        BackupAdapter, BackupCaptureRequest, BackupError, RollbackAdapter, RollbackError,
        RollbackRequest,
    },
    tweak_contracts::{
        BackupExactValue, BackupPayload, BackupRecord, RollbackKind, RollbackResult,
        RollbackStatus,
    },
};

const MAX_LOGICAL_TARGET_LEN: usize = 160;

/// In-memory Windows state fixture that implements backup and rollback adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsRollbackFixture {
    values: BTreeMap<String, String>,
    next_backup_id: u64,
    created_at_utc: String,
}

impl WindowsRollbackFixture {
    /// Creates an empty rollback fixture.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            next_backup_id: 1,
            created_at_utc: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    /// Adds or replaces one logical Windows target in the fixture state.
    #[must_use]
    pub fn with_value(
        mut self,
        target: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.values.insert(target.into(), value.into());
        self
    }

    /// Writes a value as if an apply step had mutated Windows state.
    pub fn set_value(&mut self, target: impl Into<String>, value: impl Into<String>) {
        self.values.insert(target.into(), value.into());
    }

    /// Reads a logical Windows target from the fixture.
    #[must_use]
    pub fn value(&self, target: &str) -> Option<&str> {
        self.values.get(target).map(String::as_str)
    }

    /// Returns whether the fixture contains a logical Windows target.
    #[must_use]
    pub fn contains_target(&self, target: &str) -> bool {
        self.values.contains_key(target)
    }

    fn next_backup_id(&mut self, request: &BackupCaptureRequest) -> String {
        let backup_id = format!(
            "backup:{}:{}:{}",
            request.plan_id, request.tweak_id, self.next_backup_id
        );
        self.next_backup_id += 1;
        backup_id
    }
}

impl Default for WindowsRollbackFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupAdapter for WindowsRollbackFixture {
    fn capture_backup(
        &mut self,
        request: &BackupCaptureRequest,
    ) -> Result<BackupRecord, BackupError> {
        validate_logical_target(&request.target).map_err(|detail| {
            BackupError::capture_failed(request.tweak_id.clone(), detail)
        })?;

        let payload = match request.kind {
            RollbackKind::ExactValue => {
                let values = exact_value_backup_targets(request)
                    .into_iter()
                    .map(|target| {
                        validate_logical_target(&target).map_err(|detail| {
                            BackupError::capture_failed(request.tweak_id.clone(), detail)
                        })?;

                        let value = self.values.get(&target).ok_or_else(|| {
                            BackupError::capture_failed(
                                request.tweak_id.clone(),
                                "exact-value backup target is missing from fixture state",
                            )
                        })?;

                        Ok(BackupExactValue {
                            target,
                            value: value.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, BackupError>>()?;

                match values.as_slice() {
                    [value] => BackupPayload::ExactValue {
                        target: value.target.clone(),
                        value: value.value.clone(),
                    },
                    _ => BackupPayload::ExactValues { values },
                }
            }
            RollbackKind::DeleteCreatedValue => {
                if self.values.contains_key(&request.target) {
                    return Err(BackupError::capture_failed(
                        request.tweak_id.clone(),
                        "delete-created-value backup target already exists in fixture state",
                    ));
                }

                BackupPayload::CreatedValue {
                    target: request.target.clone(),
                }
            }
            RollbackKind::RestoreBackupFile
            | RollbackKind::RestoreProfileExport
            | RollbackKind::ManualInstructions
            | RollbackKind::NotNeededReadonly => {
                return Err(BackupError::unsupported_rollback_kind(
                    request.tweak_id.clone(),
                    request.kind,
                ));
            }
        };

        Ok(BackupRecord {
            id: self.next_backup_id(request),
            tweak_id: request.tweak_id.clone(),
            rollback_kind: request.kind,
            catalog_schema_version: request.catalog_schema_version.clone(),
            created_at_utc: self.created_at_utc.clone(),
            payload,
        })
    }
}

impl RollbackAdapter for WindowsRollbackFixture {
    fn execute_rollback(
        &mut self,
        request: &RollbackRequest,
    ) -> Result<RollbackResult, RollbackError> {
        match &request.backup.payload {
            BackupPayload::ExactValue { target, value } => {
                self.ensure_plan_restores_target(request, target)?;
                validate_logical_target(target).map_err(|detail| {
                    RollbackError::restore_failed(request.tweak_id.clone(), detail)
                })?;

                self.values.insert(target.clone(), value.clone());

                Ok(restored_result(&request.backup.id, target))
            }
            BackupPayload::ExactValues { values } => {
                if values.is_empty() {
                    return Err(RollbackError::restore_failed(
                        request.tweak_id.clone(),
                        "exact-values backup payload is empty",
                    ));
                }

                let mut restored_targets = Vec::with_capacity(values.len());

                for exact in values {
                    self.ensure_plan_restores_target(request, &exact.target)?;
                    validate_logical_target(&exact.target).map_err(|detail| {
                        RollbackError::restore_failed(request.tweak_id.clone(), detail)
                    })?;

                    self.values
                        .insert(exact.target.clone(), exact.value.clone());
                    restored_targets.push(exact.target.clone());
                }

                Ok(RollbackResult {
                    backup_id: Some(request.backup.id.clone()),
                    status: RollbackStatus::Restored,
                    restored_targets,
                    messages: Vec::new(),
                })
            }
            BackupPayload::CreatedValue { target } => {
                self.ensure_plan_restores_target(request, target)?;
                validate_logical_target(target).map_err(|detail| {
                    RollbackError::restore_failed(request.tweak_id.clone(), detail)
                })?;

                self.values.remove(target);

                Ok(restored_result(&request.backup.id, target))
            }
            BackupPayload::ManualInstructions { instructions } => Ok(RollbackResult {
                backup_id: Some(request.backup.id.clone()),
                status: RollbackStatus::ManualRequired,
                restored_targets: Vec::new(),
                messages: vec![instructions.clone()],
            }),
            BackupPayload::ReadOnly => Ok(RollbackResult {
                backup_id: Some(request.backup.id.clone()),
                status: RollbackStatus::NotNeeded,
                restored_targets: Vec::new(),
                messages: vec!["Read-only backup does not require rollback.".to_owned()],
            }),
            BackupPayload::FileSnapshot { .. } => Err(RollbackError::unsupported_rollback_kind(
                request.tweak_id.clone(),
                RollbackKind::RestoreBackupFile,
            )),
            BackupPayload::ProfileExport { .. } => Err(RollbackError::unsupported_rollback_kind(
                request.tweak_id.clone(),
                RollbackKind::RestoreProfileExport,
            )),
        }
    }
}

impl WindowsRollbackFixture {
    fn ensure_plan_restores_target(
        &self,
        request: &RollbackRequest,
        target: &str,
    ) -> Result<(), RollbackError> {
        if request
            .plan
            .steps
            .iter()
            .any(|step| step.target.as_str() == target)
        {
            Ok(())
        } else {
            Err(RollbackError::backup_plan_mismatch(
                request.tweak_id.clone(),
                "rollback plan does not include the backup target",
            ))
        }
    }
}

fn exact_value_backup_targets(request: &BackupCaptureRequest) -> Vec<String> {
    let mut targets = vec![request.target.clone()];

    for change in &request.changes {
        if !targets.contains(&change.target) {
            targets.push(change.target.clone());
        }
    }

    targets
}

fn restored_result(backup_id: &str, target: &str) -> RollbackResult {
    RollbackResult {
        backup_id: Some(backup_id.to_owned()),
        status: RollbackStatus::Restored,
        restored_targets: vec![target.to_owned()],
        messages: Vec::new(),
    }
}

fn validate_logical_target(target: &str) -> Result<(), &'static str> {
    let trimmed = target.trim();

    if trimmed.is_empty()
        || trimmed != target
        || trimmed.len() > MAX_LOGICAL_TARGET_LEN
        || !trimmed.bytes().all(is_allowed_target_byte)
    {
        Err("logical Windows target failed validation")
    } else {
        Ok(())
    }
}

fn is_allowed_target_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'/' | b'.' | b'-' | b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, BackupErrorReason},
        tweak_contracts::{
            BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackPlan,
            RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
            TweakPlanItem, TweakRisk,
        },
    };

    const TARGET: &str = "registry:hkcu/software/liiiraa/gamebar/capture";

    fn planned_change(target: &str) -> PlannedChange {
        PlannedChange {
            target: target.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: None,
            desired_value: Some("0".to_owned()),
            scope: SessionScope::Persistent,
        }
    }

    fn rollback_plan(kind: RollbackKind, operation: TweakOperationKind) -> RollbackPlan {
        RollbackPlan {
            kind,
            steps: vec![RollbackStep {
                summary: "Restore fixture target.".to_owned(),
                target: TARGET.to_owned(),
                operation,
                expected_state: None,
            }],
            requires_admin: false,
            reboot: RebootPolicy::None,
            manual_instructions: Vec::new(),
        }
    }

    fn plan(kind: RollbackKind, operation: TweakOperationKind) -> TweakPlan {
        TweakPlan {
            id: "plan-rollback-fixture".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: "game.capture.background.off".to_owned(),
                category: TweakCategory::WindowsGaming,
                action: PlanAction::Apply,
                mode: TweakMode::Safe,
                risk: TweakRisk::Low,
                changes: vec![planned_change(TARGET)],
                backup: BackupRequirement::Required {
                    kind,
                    target: TARGET.to_owned(),
                },
                rollback: rollback_plan(kind, operation),
                reboot: RebootPolicy::None,
                requires_admin: false,
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        }
    }

    #[test]
    fn rollback_fixture_restores_exact_previous_value() {
        let plan = plan(RollbackKind::ExactValue, TweakOperationKind::Write);
        let mut fixture = WindowsRollbackFixture::new().with_value(TARGET, "1");
        let backups = capture_plan_backups(&plan, &mut fixture)
            .expect("exact-value backup should be captured");

        fixture.set_value(TARGET, "0");
        assert_eq!(fixture.value(TARGET), Some("0"));

        let request = RollbackRequest::new(
            "game.capture.background.off",
            backups[0].clone(),
            plan.items[0].rollback.clone(),
        )
        .expect("rollback request should be valid");
        let result = execute_rollback(&mut fixture, &request)
            .expect("exact-value rollback should restore fixture state");

        assert_eq!(result.status, RollbackStatus::Restored);
        assert_eq!(fixture.value(TARGET), Some("1"));
        assert_eq!(result.restored_targets, vec![TARGET.to_owned()]);
    }

    #[test]
    fn rollback_fixture_deletes_optimizer_created_value() {
        let plan = plan(RollbackKind::DeleteCreatedValue, TweakOperationKind::Delete);
        let mut fixture = WindowsRollbackFixture::new();
        let backups = capture_plan_backups(&plan, &mut fixture)
            .expect("created-value backup should be captured");

        fixture.set_value(TARGET, "created");
        assert!(fixture.contains_target(TARGET));

        let request = RollbackRequest::new(
            "game.capture.background.off",
            backups[0].clone(),
            plan.items[0].rollback.clone(),
        )
        .expect("rollback request should be valid");
        let result = execute_rollback(&mut fixture, &request)
            .expect("created-value rollback should remove fixture state");

        assert_eq!(result.status, RollbackStatus::Restored);
        assert!(!fixture.contains_target(TARGET));
    }

    #[test]
    fn rollback_fixture_rejects_delete_created_for_preexisting_value() {
        let plan = plan(RollbackKind::DeleteCreatedValue, TweakOperationKind::Delete);
        let mut fixture = WindowsRollbackFixture::new().with_value(TARGET, "preexisting");

        let error = capture_plan_backups(&plan, &mut fixture)
            .expect_err("preexisting values need exact-value rollback");

        assert_eq!(error.reason(), BackupErrorReason::CaptureFailed);
        assert_eq!(fixture.value(TARGET), Some("preexisting"));
    }

    #[test]
    fn rollback_fixture_rejects_unsafe_logical_targets() {
        let unsafe_target = "registry:hkcu/software/liiiraa && calc.exe";
        let plan = TweakPlan {
            id: "plan-unsafe-target".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: "game.capture.background.off".to_owned(),
                category: TweakCategory::WindowsGaming,
                action: PlanAction::Apply,
                mode: TweakMode::Safe,
                risk: TweakRisk::Low,
                changes: vec![planned_change(unsafe_target)],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::DeleteCreatedValue,
                    target: unsafe_target.to_owned(),
                },
                rollback: rollback_plan(
                    RollbackKind::DeleteCreatedValue,
                    TweakOperationKind::Delete,
                ),
                reboot: RebootPolicy::None,
                requires_admin: false,
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new();

        let error = capture_plan_backups(&plan, &mut fixture)
            .expect_err("unsafe target should not be captured");

        assert_eq!(error.reason(), BackupErrorReason::CaptureFailed);
        assert_eq!(error.tweak_id(), Some("game.capture.background.off"));
    }
}
