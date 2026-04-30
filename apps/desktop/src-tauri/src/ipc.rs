use optimizer_core::ipc::{
    ipc_allowlist, validate_ipc_request, IpcCommandPayload, IpcDenial, RawIpcRequest,
    SecurityStatusPayload,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SecurityStatusRequest {
    include_allowlist: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityStatusResponse {
    command_id: String,
    requester: String,
    deny_by_default: bool,
    allows_elevation: bool,
    allowlisted_commands: Vec<AllowlistedCommandResponse>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllowlistedCommandResponse {
    command_id: String,
    payload_kind: String,
    risk: String,
    allows_elevation: bool,
    audit_denials: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IpcErrorResponse {
    reason: String,
    message: String,
    command_id: Option<String>,
}

#[tauri::command]
pub(crate) fn get_ipc_security_status(
    requester: String,
    payload: SecurityStatusRequest,
) -> Result<SecurityStatusResponse, IpcErrorResponse> {
    let request = RawIpcRequest::security_status(
        requester,
        SecurityStatusPayload {
            include_allowlist: payload.include_allowlist,
        },
    )
    .map_err(IpcErrorResponse::from)?;

    let validated = validate_ipc_request(request).map_err(IpcErrorResponse::from)?;
    let allows_elevation = validated.policy().allows_elevation;
    let include_allowlist = match validated.payload() {
        IpcCommandPayload::SecurityStatus(payload) => payload.include_allowlist,
        IpcCommandPayload::Empty => false,
    };

    Ok(SecurityStatusResponse {
        command_id: validated.command_id().as_str().to_owned(),
        requester: validated.requester().as_str().to_owned(),
        deny_by_default: true,
        allows_elevation,
        allowlisted_commands: include_allowlist
            .then(|| {
                ipc_allowlist()
                    .iter()
                    .copied()
                    .map(AllowlistedCommandResponse::from)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

impl From<optimizer_core::ipc::IpcCommandPolicy> for AllowlistedCommandResponse {
    fn from(policy: optimizer_core::ipc::IpcCommandPolicy) -> Self {
        Self {
            command_id: policy.command_id.as_str().to_owned(),
            payload_kind: policy.payload_kind.as_str().to_owned(),
            risk: policy.risk.as_str().to_owned(),
            allows_elevation: policy.allows_elevation,
            audit_denials: policy.audit_denials,
        }
    }
}

impl From<IpcDenial> for IpcErrorResponse {
    fn from(denial: IpcDenial) -> Self {
        Self {
            reason: denial.reason().as_str().to_owned(),
            message: denial.message().to_owned(),
            command_id: denial.command_id().map(ToOwned::to_owned),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::ipc::{IpcCommandId, IpcRisk};

    #[test]
    fn security_status_command_returns_allowlist_when_requested() {
        let response = get_ipc_security_status(
            "main-window".to_owned(),
            SecurityStatusRequest {
                include_allowlist: true,
            },
        )
        .expect("read-only IPC status should be available");

        assert_eq!(response.command_id, IpcCommandId::SecurityStatus.as_str());
        assert_eq!(response.requester, "main-window");
        assert!(response.deny_by_default);
        assert!(!response.allows_elevation);
        assert_eq!(response.allowlisted_commands.len(), 1);
        assert_eq!(response.allowlisted_commands[0].risk, IpcRisk::ReadOnly.as_str());
    }

    #[test]
    fn security_status_command_can_omit_allowlist_details() {
        let response = get_ipc_security_status(
            "main-window".to_owned(),
            SecurityStatusRequest {
                include_allowlist: false,
            },
        )
        .expect("read-only IPC status should be available");

        assert!(response.allowlisted_commands.is_empty());
    }

    #[test]
    fn security_status_command_rejects_invalid_requester() {
        let error = get_ipc_security_status(
            "main window; shell".to_owned(),
            SecurityStatusRequest {
                include_allowlist: true,
            },
        )
        .expect_err("invalid requester should be denied");

        assert_eq!(error.reason, "invalid_requester");
        assert_eq!(error.command_id, None);
    }
}
