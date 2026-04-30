//! Typed IPC command validation and allowlist policy.

use std::fmt;

/// Stable command ID for the read-only IPC security status command.
pub const SECURITY_STATUS_COMMAND: &str = "security.status";

const MAX_REQUESTER_LEN: usize = 64;

/// Commands known to the optimizer IPC boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommandId {
    /// Returns read-only metadata about the IPC security boundary.
    SecurityStatus,
}

impl IpcCommandId {
    /// Returns the stable string identifier used across the IPC boundary.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SecurityStatus => SECURITY_STATUS_COMMAND,
        }
    }

    /// Parses a string command ID into a typed command.
    pub fn parse(command_id: &str) -> Result<Self, IpcDenial> {
        match command_id {
            SECURITY_STATUS_COMMAND => Ok(Self::SecurityStatus),
            _ => Err(IpcDenial::unknown_command(command_id)),
        }
    }
}

/// High-level risk class for an IPC command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcRisk {
    /// Command reads local state or policy metadata and cannot change the system.
    ReadOnly,
    /// Command may affect user-mode app state without elevation.
    UserMode,
    /// Command may require elevated or system-level privileges.
    Elevated,
}

impl IpcRisk {
    /// Returns a stable string representation for logs and frontend DTOs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::UserMode => "user_mode",
            Self::Elevated => "elevated",
        }
    }
}

/// Typed payload schema expected for an IPC command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcPayloadKind {
    /// Payload for [`IpcCommandId::SecurityStatus`].
    SecurityStatus,
}

impl IpcPayloadKind {
    /// Returns a stable string representation for logs and frontend DTOs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SecurityStatus => "security_status",
        }
    }
}

/// Static policy attached to an allowlisted IPC command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpcCommandPolicy {
    /// Typed command identifier.
    pub command_id: IpcCommandId,
    /// Payload schema expected before the command can run.
    pub payload_kind: IpcPayloadKind,
    /// Command risk class.
    pub risk: IpcRisk,
    /// Whether the command is allowed to cross into elevated execution.
    pub allows_elevation: bool,
    /// Whether denied attempts should be recorded by the caller's audit sink.
    pub audit_denials: bool,
}

const SECURITY_STATUS_POLICY: IpcCommandPolicy = IpcCommandPolicy {
    command_id: IpcCommandId::SecurityStatus,
    payload_kind: IpcPayloadKind::SecurityStatus,
    risk: IpcRisk::ReadOnly,
    allows_elevation: false,
    audit_denials: true,
};

const IPC_ALLOWLIST: &[IpcCommandPolicy] = &[SECURITY_STATUS_POLICY];

/// Returns the deny-by-default IPC command allowlist.
#[must_use]
pub const fn ipc_allowlist() -> &'static [IpcCommandPolicy] {
    IPC_ALLOWLIST
}

/// Typed payload for the read-only IPC security status command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SecurityStatusPayload {
    /// Include command policy details in the response.
    pub include_allowlist: bool,
}

/// Typed IPC payload variants accepted by the validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommandPayload {
    /// No schema was provided by the caller.
    Empty,
    /// Read-only IPC security status payload.
    SecurityStatus(SecurityStatusPayload),
}

impl IpcCommandPayload {
    fn kind(&self) -> Option<IpcPayloadKind> {
        match self {
            Self::Empty => None,
            Self::SecurityStatus(_) => Some(IpcPayloadKind::SecurityStatus),
        }
    }
}

/// Validated requester identity attached to an IPC command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcRequester {
    value: String,
}

impl IpcRequester {
    /// Validates a requester label supplied by the IPC caller.
    pub fn new(value: impl Into<String>) -> Result<Self, IpcDenial> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty()
            || trimmed.len() > MAX_REQUESTER_LEN
            || !trimmed.bytes().all(is_allowed_requester_byte)
        {
            return Err(IpcDenial::invalid_requester());
        }

        Ok(Self {
            value: trimmed.to_owned(),
        })
    }

    /// Returns the validated requester label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_allowed_requester_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
}

/// Raw typed command request before allowlist validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawIpcRequest {
    command_id: String,
    requester: IpcRequester,
    payload: IpcCommandPayload,
}

impl RawIpcRequest {
    /// Creates a request for allowlist validation.
    pub fn new(
        command_id: impl Into<String>,
        requester: impl Into<String>,
        payload: IpcCommandPayload,
    ) -> Result<Self, IpcDenial> {
        Ok(Self {
            command_id: command_id.into(),
            requester: IpcRequester::new(requester)?,
            payload,
        })
    }

    /// Creates a typed request for the read-only security status command.
    pub fn security_status(
        requester: impl Into<String>,
        payload: SecurityStatusPayload,
    ) -> Result<Self, IpcDenial> {
        Self::new(
            SECURITY_STATUS_COMMAND,
            requester,
            IpcCommandPayload::SecurityStatus(payload),
        )
    }
}

/// IPC command that passed requester, schema, and allowlist validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedIpcCommand {
    command_id: IpcCommandId,
    requester: IpcRequester,
    payload: IpcCommandPayload,
    policy: IpcCommandPolicy,
}

impl ValidatedIpcCommand {
    /// Returns the typed command identifier.
    #[must_use]
    pub const fn command_id(&self) -> IpcCommandId {
        self.command_id
    }

    /// Returns the validated requester label.
    #[must_use]
    pub fn requester(&self) -> &IpcRequester {
        &self.requester
    }

    /// Returns the typed command payload.
    #[must_use]
    pub fn payload(&self) -> &IpcCommandPayload {
        &self.payload
    }

    /// Returns the allowlist policy that authorized this command.
    #[must_use]
    pub const fn policy(&self) -> IpcCommandPolicy {
        self.policy
    }
}

/// Validates a raw typed IPC request against the deny-by-default allowlist.
pub fn validate_ipc_request(request: RawIpcRequest) -> Result<ValidatedIpcCommand, IpcDenial> {
    let command_id = IpcCommandId::parse(&request.command_id)?;
    let policy = ipc_allowlist()
        .iter()
        .copied()
        .find(|policy| policy.command_id == command_id)
        .ok_or_else(|| IpcDenial::command_not_allowlisted(command_id.as_str()))?;

    if request.payload.kind() != Some(policy.payload_kind) {
        return Err(IpcDenial::invalid_payload(command_id.as_str()));
    }

    Ok(ValidatedIpcCommand {
        command_id,
        requester: request.requester,
        payload: request.payload,
        policy,
    })
}

/// Denial reason emitted by the IPC validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcDenialReason {
    /// Command ID is not known to the typed command registry.
    UnknownCommand,
    /// Command ID is known but not present in the active allowlist.
    CommandNotAllowlisted,
    /// Payload did not match the command schema.
    InvalidPayload,
    /// Requester identity failed validation.
    InvalidRequester,
}

impl IpcDenialReason {
    /// Returns a stable string representation for logs and frontend DTOs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCommand => "unknown_command",
            Self::CommandNotAllowlisted => "command_not_allowlisted",
            Self::InvalidPayload => "invalid_payload",
            Self::InvalidRequester => "invalid_requester",
        }
    }
}

/// Structured denial returned by the IPC validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcDenial {
    reason: IpcDenialReason,
    command_id: Option<String>,
}

impl IpcDenial {
    fn unknown_command(command_id: impl Into<String>) -> Self {
        Self {
            reason: IpcDenialReason::UnknownCommand,
            command_id: Some(command_id.into()),
        }
    }

    fn command_not_allowlisted(command_id: impl Into<String>) -> Self {
        Self {
            reason: IpcDenialReason::CommandNotAllowlisted,
            command_id: Some(command_id.into()),
        }
    }

    fn invalid_payload(command_id: impl Into<String>) -> Self {
        Self {
            reason: IpcDenialReason::InvalidPayload,
            command_id: Some(command_id.into()),
        }
    }

    fn invalid_requester() -> Self {
        Self {
            reason: IpcDenialReason::InvalidRequester,
            command_id: None,
        }
    }

    /// Returns the denial reason.
    #[must_use]
    pub const fn reason(&self) -> IpcDenialReason {
        self.reason
    }

    /// Returns the denied command ID when validation reached one.
    #[must_use]
    pub fn command_id(&self) -> Option<&str> {
        self.command_id.as_deref()
    }

    /// Returns a short human-readable denial message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self.reason {
            IpcDenialReason::UnknownCommand => "IPC command is not registered",
            IpcDenialReason::CommandNotAllowlisted => "IPC command is not allowlisted",
            IpcDenialReason::InvalidPayload => "IPC command payload failed validation",
            IpcDenialReason::InvalidRequester => "IPC requester failed validation",
        }
    }
}

impl fmt::Display for IpcDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.command_id() {
            Some(command_id) => write!(
                formatter,
                "{}: {} ({command_id})",
                self.reason.as_str(),
                self.message()
            ),
            None => write!(formatter, "{}: {}", self.reason.as_str(), self.message()),
        }
    }
}

impl std::error::Error for IpcDenial {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_allowlisted_typed_security_status_request() {
        let request = RawIpcRequest::security_status(
            "main-window",
            SecurityStatusPayload {
                include_allowlist: true,
            },
        )
        .expect("requester should be valid");

        let validated = validate_ipc_request(request).expect("command should be allowlisted");

        assert_eq!(validated.command_id(), IpcCommandId::SecurityStatus);
        assert_eq!(validated.requester().as_str(), "main-window");
        assert_eq!(validated.policy().risk, IpcRisk::ReadOnly);
        assert!(!validated.policy().allows_elevation);
    }

    #[test]
    fn denies_unknown_command() {
        let request = RawIpcRequest::new(
            "agent.run_shell",
            "main-window",
            IpcCommandPayload::SecurityStatus(SecurityStatusPayload::default()),
        )
        .expect("requester should be valid");

        let denial = validate_ipc_request(request).expect_err("unknown command must be denied");

        assert_eq!(denial.reason(), IpcDenialReason::UnknownCommand);
        assert_eq!(denial.command_id(), Some("agent.run_shell"));
    }

    #[test]
    fn denies_missing_payload_schema() {
        let request = RawIpcRequest::new(
            SECURITY_STATUS_COMMAND,
            "main-window",
            IpcCommandPayload::Empty,
        )
        .expect("requester should be valid");

        let denial = validate_ipc_request(request).expect_err("empty payload must be denied");

        assert_eq!(denial.reason(), IpcDenialReason::InvalidPayload);
        assert_eq!(denial.command_id(), Some(SECURITY_STATUS_COMMAND));
    }

    #[test]
    fn denies_invalid_requester() {
        let denial = RawIpcRequest::security_status(
            "main window; rm",
            SecurityStatusPayload::default(),
        )
        .expect_err("shell-like requester label must be denied");

        assert_eq!(denial.reason(), IpcDenialReason::InvalidRequester);
    }

    #[test]
    fn allowlist_starts_read_only_and_non_elevated() {
        let allowlist = ipc_allowlist();

        assert_eq!(allowlist.len(), 1);
        assert_eq!(allowlist[0].command_id, IpcCommandId::SecurityStatus);
        assert_eq!(allowlist[0].risk, IpcRisk::ReadOnly);
        assert!(!allowlist[0].allows_elevation);
        assert!(allowlist[0].audit_denials);
    }
}
