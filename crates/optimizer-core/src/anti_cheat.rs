//! Anti-cheat safety guardrails for PUBG and protected game workflows.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan, TweakPlanItem,
        TweakRisk,
    },
};

/// Tweak ID for denying PUBG game memory edits or process-memory access.
pub const PUBG_MEMORY_EDIT_TWEAK_ID: &str = "pubg.memory-edit";
/// Tweak ID for denying PUBG game-content deletion or patching.
pub const PUBG_DELETE_GAME_CONTENT_TWEAK_ID: &str = "pubg.delete-game-content";
/// Tweak ID for denying BattlEye file mutation.
pub const PUBG_BATTLEYE_FILES_TWEAK_ID: &str = "pubg.battleye-files";
/// Blocked guardrail ID for driver signing, test-signing, and kernel debugging changes.
pub const BLOCKED_DRIVER_SIGNING_GUARDRAIL_ID: &str = "blocked.driver-signing";
/// Blocked guardrail ID for anti-cheat process, service, handle, or permission tampering.
pub const BLOCKED_ANTICHEAT_TAMPER_GUARDRAIL_ID: &str = "blocked.anticheat-tamper";

/// Logical denial target for PUBG process-memory requests.
pub const TARGET_PUBG_GAME_MEMORY: &str = "blocked:pubg/game-memory";
/// Logical denial target for PUBG game-content mutation requests.
pub const TARGET_PUBG_GAME_CONTENT: &str = "blocked:pubg/game-content";
/// Logical denial target for BattlEye file mutation requests.
pub const TARGET_BATTLEYE_FILES: &str = "blocked:pubg/battleye-files";
/// Logical denial target for driver signature enforcement changes.
pub const TARGET_DRIVER_SIGNATURE_ENFORCEMENT: &str =
    "blocked:windows/driver-signature-enforcement";
/// Logical denial target for Windows test-signing changes.
pub const TARGET_TEST_SIGNING: &str = "blocked:windows/testsigning";
/// Logical denial target for kernel debugging changes.
pub const TARGET_KERNEL_DEBUGGING: &str = "blocked:windows/kernel-debugging";
/// Logical denial target for anti-cheat service or permission tampering.
pub const TARGET_ANTICHEAT_SERVICE_TAMPER: &str = "blocked:pubg/anticheat-service-tamper";

/// Unsafe anti-cheat-adjacent action requested by a script, catalog entry, or shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiCheatGuardrailAction {
    /// Request to read, write, patch, or scan PUBG process memory.
    ModifyGameMemory,
    /// Request to delete or patch PUBG binaries, PAKs, movies, shaders, or content folders.
    ModifyPubgGameContent,
    /// Request to modify BattlEye files or directories.
    ModifyBattleEyeFiles,
    /// Request to disable driver signature enforcement or integrity checks.
    DisableDriverSignatureEnforcement,
    /// Request to enable Windows test-signing.
    EnableTestSigning,
    /// Request to enable kernel debugging or anti-cheat-hostile debug state.
    EnableKernelDebugging,
    /// Request to alter BattlEye services, permissions, handles, or launch integrity.
    TamperWithAntiCheatService,
}

impl AntiCheatGuardrailAction {
    /// All anti-cheat guardrail actions owned by T075.
    pub const ALL: [Self; 7] = [
        Self::ModifyGameMemory,
        Self::ModifyPubgGameContent,
        Self::ModifyBattleEyeFiles,
        Self::DisableDriverSignatureEnforcement,
        Self::EnableTestSigning,
        Self::EnableKernelDebugging,
        Self::TamperWithAntiCheatService,
    ];

    const fn target(self) -> &'static str {
        match self {
            Self::ModifyGameMemory => TARGET_PUBG_GAME_MEMORY,
            Self::ModifyPubgGameContent => TARGET_PUBG_GAME_CONTENT,
            Self::ModifyBattleEyeFiles => TARGET_BATTLEYE_FILES,
            Self::DisableDriverSignatureEnforcement => TARGET_DRIVER_SIGNATURE_ENFORCEMENT,
            Self::EnableTestSigning => TARGET_TEST_SIGNING,
            Self::EnableKernelDebugging => TARGET_KERNEL_DEBUGGING,
            Self::TamperWithAntiCheatService => TARGET_ANTICHEAT_SERVICE_TAMPER,
        }
    }

    const fn desired_value(self) -> &'static str {
        match self {
            Self::ModifyGameMemory => "modify_game_memory",
            Self::ModifyPubgGameContent => "modify_pubg_game_content",
            Self::ModifyBattleEyeFiles => "modify_battleye_files",
            Self::DisableDriverSignatureEnforcement => "disable_driver_signature_enforcement",
            Self::EnableTestSigning => "enable_testsigning",
            Self::EnableKernelDebugging => "enable_kernel_debugging",
            Self::TamperWithAntiCheatService => "tamper_anticheat_service",
        }
    }

    const fn denial_warning(self) -> &'static str {
        match self {
            Self::ModifyGameMemory => {
                "PUBG game memory and process-memory access are denied; Liiiraa never reads, writes, patches, or scans the live game process."
            }
            Self::ModifyPubgGameContent => {
                "PUBG binaries, PAKs, movies, shaders, and content folders are denied mutation; use official launcher verify or repair flows."
            }
            Self::ModifyBattleEyeFiles => {
                "BattlEye file and directory mutation is denied to preserve anti-cheat trust."
            }
            Self::DisableDriverSignatureEnforcement => {
                "Driver signature enforcement and integrity-check bypasses are denied because they weaken Windows and anti-cheat trust."
            }
            Self::EnableTestSigning => {
                "Windows test-signing changes are denied because BattlEye flags test-signing as incompatible with trusted play."
            }
            Self::EnableKernelDebugging => {
                "Kernel debugging changes are denied because anti-cheat systems treat kernel debug state as unsafe."
            }
            Self::TamperWithAntiCheatService => {
                "BattlEye service, permission, handle, and launch-integrity tampering is denied."
            }
        }
    }
}

/// Request used to build T075 anti-cheat guardrail denials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiCheatGuardrailPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Unsafe anti-cheat-adjacent actions requested by a script, catalog entry, or shortcut.
    pub requested_actions: Vec<AntiCheatGuardrailAction>,
}

impl AntiCheatGuardrailPlanRequest {
    /// Creates an anti-cheat guardrail request with no unsafe actions requested yet.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            requested_actions: Vec::new(),
        }
    }

    /// Creates an anti-cheat guardrail request with explicit unsafe action candidates.
    #[must_use]
    pub fn with_actions(
        plan_id: impl Into<String>,
        requested_actions: Vec<AntiCheatGuardrailAction>,
    ) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            requested_actions,
        }
    }
}

/// Builds an anti-cheat guardrail plan that denies PUBG memory, BattlEye, and kernel bypass requests.
#[must_use]
pub fn build_anticheat_guardrail_plan(request: &AntiCheatGuardrailPlanRequest) -> TweakPlan {
    let items = vec![
        guardrail_item(
            PUBG_MEMORY_EDIT_TWEAK_ID,
            &[AntiCheatGuardrailAction::ModifyGameMemory],
            request,
        ),
        guardrail_item(
            PUBG_DELETE_GAME_CONTENT_TWEAK_ID,
            &[AntiCheatGuardrailAction::ModifyPubgGameContent],
            request,
        ),
        guardrail_item(
            PUBG_BATTLEYE_FILES_TWEAK_ID,
            &[AntiCheatGuardrailAction::ModifyBattleEyeFiles],
            request,
        ),
        guardrail_item(
            BLOCKED_DRIVER_SIGNING_GUARDRAIL_ID,
            &[
                AntiCheatGuardrailAction::DisableDriverSignatureEnforcement,
                AntiCheatGuardrailAction::EnableTestSigning,
                AntiCheatGuardrailAction::EnableKernelDebugging,
            ],
            request,
        ),
        guardrail_item(
            BLOCKED_ANTICHEAT_TAMPER_GUARDRAIL_ID,
            &[AntiCheatGuardrailAction::TamperWithAntiCheatService],
            request,
        ),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .cloned()
        .collect();

    TweakPlan {
        id: request.plan_id.clone(),
        requested_mode: request.requested_mode,
        catalog_schema_version: SUPPORTED_CATALOG_SCHEMA_VERSION.to_owned(),
        items,
        warnings,
    }
}

/// Returns true when the ID belongs to the T075 anti-cheat guardrail scope.
#[must_use]
pub fn is_anticheat_guardrail_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        PUBG_MEMORY_EDIT_TWEAK_ID
            | PUBG_DELETE_GAME_CONTENT_TWEAK_ID
            | PUBG_BATTLEYE_FILES_TWEAK_ID
            | BLOCKED_DRIVER_SIGNING_GUARDRAIL_ID
            | BLOCKED_ANTICHEAT_TAMPER_GUARDRAIL_ID
    )
}

/// Returns true when every T075 anti-cheat request is denied and no apply action exists.
#[must_use]
pub fn anticheat_guardrail_plan_blocks_unsafe_actions(plan: &TweakPlan) -> bool {
    !plan.has_apply_items()
        && plan.items.iter().all(|item| {
            is_anticheat_guardrail_tweak_id(&item.tweak_id)
                && item.category == TweakCategory::BlockedGuardrail
                && item.mode == TweakMode::Blocked
                && item.risk == TweakRisk::Critical
                && item.backup == BackupRequirement::NotRequired
                && item.rollback.kind == RollbackKind::NotNeededReadonly
                && matches!(item.action, PlanAction::DetectOnly | PlanAction::Deny)
                && item.changes.iter().all(|change| {
                    change.operation == TweakOperationKind::Deny
                        && change.scope == SessionScope::Blocked
                })
        })
}

fn guardrail_item(
    tweak_id: &str,
    actions: &[AntiCheatGuardrailAction],
    request: &AntiCheatGuardrailPlanRequest,
) -> TweakPlanItem {
    let requested_actions = actions
        .iter()
        .copied()
        .filter(|action| request.requested_actions.contains(action))
        .collect::<Vec<_>>();
    let changes = requested_actions
        .iter()
        .map(|action| PlannedChange {
            target: action.target().to_owned(),
            operation: TweakOperationKind::Deny,
            previous_value: None,
            desired_value: Some(action.desired_value().to_owned()),
            scope: SessionScope::Blocked,
        })
        .collect::<Vec<_>>();
    let warnings = requested_actions
        .iter()
        .map(|action| action.denial_warning().to_owned())
        .collect::<Vec<_>>();

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::BlockedGuardrail,
        action: if requested_actions.is_empty() {
            PlanAction::DetectOnly
        } else {
            PlanAction::Deny
        },
        mode: TweakMode::Blocked,
        risk: TweakRisk::Critical,
        changes,
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("guardrail item should exist")
    }

    #[test]
    fn idle_anticheat_plan_has_only_detect_guards() {
        let request = AntiCheatGuardrailPlanRequest::new("plan-anticheat-idle");

        let plan = build_anticheat_guardrail_plan(&request);

        assert!(anticheat_guardrail_plan_blocks_unsafe_actions(&plan));
        assert!(!plan.has_apply_items());
        assert!(!plan.has_denials());
        assert!(plan.items.iter().all(|item| item.action == PlanAction::DetectOnly));
    }

    #[test]
    fn denies_pubg_memory_battleye_and_kernel_bypass_requests() {
        let request = AntiCheatGuardrailPlanRequest::with_actions(
            "plan-anticheat-denials",
            vec![
                AntiCheatGuardrailAction::ModifyGameMemory,
                AntiCheatGuardrailAction::ModifyBattleEyeFiles,
                AntiCheatGuardrailAction::DisableDriverSignatureEnforcement,
                AntiCheatGuardrailAction::EnableTestSigning,
                AntiCheatGuardrailAction::EnableKernelDebugging,
            ],
        );

        let plan = build_anticheat_guardrail_plan(&request);

        assert!(anticheat_guardrail_plan_blocks_unsafe_actions(&plan));
        assert!(plan.has_denials());
        assert!(!plan.has_apply_items());

        let memory = item(&plan, PUBG_MEMORY_EDIT_TWEAK_ID);
        assert_eq!(memory.action, PlanAction::Deny);
        assert_eq!(memory.changes[0].target, TARGET_PUBG_GAME_MEMORY);
        assert!(memory.warnings[0].contains("process-memory"));

        let battleye = item(&plan, PUBG_BATTLEYE_FILES_TWEAK_ID);
        assert_eq!(battleye.action, PlanAction::Deny);
        assert_eq!(battleye.changes[0].target, TARGET_BATTLEYE_FILES);
        assert!(battleye.warnings[0].contains("BattlEye"));

        let driver_signing = item(&plan, BLOCKED_DRIVER_SIGNING_GUARDRAIL_ID);
        assert_eq!(driver_signing.action, PlanAction::Deny);
        assert_eq!(driver_signing.changes.len(), 3);
        assert!(driver_signing
            .warnings
            .iter()
            .any(|warning| warning.contains("test-signing")));
        assert!(driver_signing
            .warnings
            .iter()
            .any(|warning| warning.contains("Kernel debugging")));
        assert!(driver_signing
            .changes
            .iter()
            .all(|change| change.scope == SessionScope::Blocked));
    }

    #[test]
    fn denies_pubg_content_and_anticheat_service_tampering() {
        let request = AntiCheatGuardrailPlanRequest::with_actions(
            "plan-anticheat-service",
            vec![
                AntiCheatGuardrailAction::ModifyPubgGameContent,
                AntiCheatGuardrailAction::TamperWithAntiCheatService,
            ],
        );

        let plan = build_anticheat_guardrail_plan(&request);

        let content = item(&plan, PUBG_DELETE_GAME_CONTENT_TWEAK_ID);
        assert_eq!(content.action, PlanAction::Deny);
        assert_eq!(content.changes[0].target, TARGET_PUBG_GAME_CONTENT);
        assert!(content.warnings[0].contains("official launcher verify"));

        let service = item(&plan, BLOCKED_ANTICHEAT_TAMPER_GUARDRAIL_ID);
        assert_eq!(service.action, PlanAction::Deny);
        assert_eq!(service.changes[0].target, TARGET_ANTICHEAT_SERVICE_TAMPER);
        assert!(service.warnings[0].contains("service"));
    }
}
