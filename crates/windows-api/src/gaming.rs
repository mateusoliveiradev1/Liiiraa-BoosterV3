//! Registry-fixture adapter for safe Windows gaming capture controls.

use std::fmt;

use optimizer_core::{
    gaming_capture::{is_gaming_capture_registry_target, is_gaming_capture_tweak_id},
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::WindowsRollbackFixture;

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamingCaptureRegistrySummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Registry targets written or verified.
    pub targets: Vec<String>,
}

impl GamingCaptureRegistrySummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Applies T042 gaming capture registry changes to an in-memory Windows fixture.
pub fn apply_gaming_capture_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<GamingCaptureRegistrySummary, GamingCaptureRegistryError> {
    let mut summary = GamingCaptureRegistrySummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                GamingCaptureRegistryError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            fixture.set_value(change.target.clone(), desired.to_owned());
            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Verifies T042 gaming capture registry changes against an in-memory fixture.
pub fn verify_gaming_capture_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<GamingCaptureRegistrySummary, GamingCaptureRegistryError> {
    let mut summary = GamingCaptureRegistrySummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                GamingCaptureRegistryError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(GamingCaptureRegistryError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), GamingCaptureRegistryError> {
    if is_gaming_capture_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(GamingCaptureRegistryError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), GamingCaptureRegistryError> {
    if change.operation != TweakOperationKind::Write {
        return Err(GamingCaptureRegistryError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_gaming_capture_registry_target(&change.target) {
        return Err(GamingCaptureRegistryError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed gaming capture registry operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamingCaptureRegistryErrorReason {
    /// Plan item was not part of the T042 gaming capture scope.
    UnsupportedTweak,
    /// Plan item targeted an unsupported registry value.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
}

impl GamingCaptureRegistryErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-gaming-capture tweak",
            Self::UnsupportedTarget => "Plan targets a registry value outside the T042 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Registry fixture readback did not match the plan",
        }
    }
}

/// Structured error for fixture-backed gaming capture registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamingCaptureRegistryError {
    reason: GamingCaptureRegistryErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl GamingCaptureRegistryError {
    fn new(
        reason: GamingCaptureRegistryErrorReason,
        tweak_id: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            reason,
            tweak_id,
            target,
        }
    }

    fn unsupported_tweak(tweak_id: impl Into<String>) -> Self {
        Self::new(
            GamingCaptureRegistryErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GamingCaptureRegistryErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GamingCaptureRegistryErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GamingCaptureRegistryErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GamingCaptureRegistryErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> GamingCaptureRegistryErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected registry target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for GamingCaptureRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for GamingCaptureRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        gaming_capture::{
            build_gaming_capture_plan, GamingCapturePlanRequest, GamingControlConsent,
            GamingFeatureUse, GAME_CAPTURE_BACKGROUND_TWEAK_ID,
            GAME_NOTIFICATIONS_FOCUS_TWEAK_ID, TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED,
            TARGET_GAME_DVR_APP_CAPTURE_ENABLED, TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED,
            TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR, TARGET_NOTIFICATIONS_DND_ENABLED,
        },
        tweak_contracts::{BackupPayload, RollbackStatus},
    };

    fn plan_item<'a>(
        plan: &'a optimizer_core::tweak_contracts::TweakPlan,
        tweak_id: &str,
    ) -> &'a optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn registry_fixture_applies_verifies_and_rolls_back_capture_values() {
        let mut request = GamingCapturePlanRequest::new("plan-registry-capture");
        request.windows_capture_use = GamingFeatureUse::NotUsed;
        request.include_machine_policy = true;
        let plan = build_gaming_capture_plan(&request);
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED, "1")
            .with_value(TARGET_GAME_DVR_APP_CAPTURE_ENABLED, "1")
            .with_value(TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED, "1")
            .with_value(TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR, "1");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("capture backups should succeed");
        assert_eq!(backups.len(), 1);

        match &backups[0].payload {
            BackupPayload::ExactValues { values } => assert_eq!(values.len(), 4),
            payload => panic!("expected grouped backup payload, got {payload:?}"),
        }

        let applied = apply_gaming_capture_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");
        assert_eq!(applied.targets.len(), 4);
        verify_gaming_capture_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");
        assert_eq!(
            fixture.value(TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED),
            Some("0")
        );

        let item = plan_item(&plan, GAME_CAPTURE_BACKGROUND_TWEAK_ID);
        let rollback_request = RollbackRequest::new(
            GAME_CAPTURE_BACKGROUND_TWEAK_ID,
            backups[0].clone(),
            item.rollback.clone(),
        )
        .expect("rollback request should be valid");
        let rollback = execute_rollback(&mut fixture, &rollback_request)
            .expect("rollback should restore previous registry values");

        assert_eq!(rollback.status, RollbackStatus::Restored);
        assert_eq!(
            fixture.value(TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED),
            Some("1")
        );
        assert_eq!(fixture.value(TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR), Some("1"));
    }

    #[test]
    fn registry_fixture_restores_session_focus_state() {
        let mut request = GamingCapturePlanRequest::new("plan-registry-focus");
        request.windows_capture_use = GamingFeatureUse::Used;
        request.focus_assist_consent = GamingControlConsent::Granted;
        let plan = build_gaming_capture_plan(&request);
        let mut fixture =
            WindowsRollbackFixture::new().with_value(TARGET_NOTIFICATIONS_DND_ENABLED, "0");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("focus backup should succeed");
        let applied = apply_gaming_capture_plan_to_fixture(&mut fixture, &plan)
            .expect("focus apply should succeed");

        assert_eq!(applied.item_count, 1);
        assert_eq!(fixture.value(TARGET_NOTIFICATIONS_DND_ENABLED), Some("1"));
        verify_gaming_capture_plan_fixture(&fixture, &plan)
            .expect("focus readback should verify");

        let item = plan_item(&plan, GAME_NOTIFICATIONS_FOCUS_TWEAK_ID);
        let rollback_request = RollbackRequest::new(
            GAME_NOTIFICATIONS_FOCUS_TWEAK_ID,
            backups[0].clone(),
            item.rollback.clone(),
        )
        .expect("focus rollback request should be valid");

        execute_rollback(&mut fixture, &rollback_request)
            .expect("focus rollback should restore previous notification state");

        assert_eq!(fixture.value(TARGET_NOTIFICATIONS_DND_ENABLED), Some("0"));
    }
}
