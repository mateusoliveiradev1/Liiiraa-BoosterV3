//! Elevated Windows agent boundary for privileged optimizer actions.

use std::fmt;

use windows_api::{StructuredCommandPlan, WindowsCommandPlanError};

/// Stable command ID for activating a prepared power plan through the agent.
pub const ACTIVATE_PREPARED_POWER_PLAN_COMMAND: &str = "agent.power_plan.activate_prepared";

const MAX_REQUESTER_LEN: usize = 64;
const MAX_ROLLBACK_REFERENCE_LEN: usize = 96;
const POWER_SCHEME_GUID_LEN: usize = 36;

/// Static metadata describing this workspace crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrateInfo {
    /// Cargo package name.
    pub name: &'static str,
    /// Design-level responsibility owned by the crate.
    pub responsibility: &'static str,
    /// Whether the crate eventually needs live Windows state for full coverage.
    pub requires_live_windows: bool,
}

/// Windows agent crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "windows-agent",
    responsibility: "host the elevated allowlisted boundary for privileged optimizer actions",
    requires_live_windows: true,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

/// Commands known to the elevated agent boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCommandId {
    /// Activate a previously prepared Windows power scheme.
    ActivatePreparedPowerPlan,
}

impl AgentCommandId {
    /// Returns the stable command string used by callers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivatePreparedPowerPlan => ACTIVATE_PREPARED_POWER_PLAN_COMMAND,
        }
    }

    /// Parses a command string into an agent command ID.
    pub fn parse(command_id: &str) -> Result<Self, AgentDenial> {
        match command_id {
            ACTIVATE_PREPARED_POWER_PLAN_COMMAND => Ok(Self::ActivatePreparedPowerPlan),
            _ => Err(AgentDenial::unknown_command(command_id)),
        }
    }
}

/// Command risk at the elevated agent boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCommandRisk {
    /// Command may mutate system-level Windows state.
    Elevated,
}

impl AgentCommandRisk {
    /// Returns a stable risk string for audit records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Elevated => "elevated",
        }
    }
}

/// Payload schema required for each agent command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPayloadKind {
    /// Payload for [`AgentCommandId::ActivatePreparedPowerPlan`].
    ActivatePreparedPowerPlan,
}

impl AgentPayloadKind {
    /// Returns a stable payload kind string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivatePreparedPowerPlan => "activate_prepared_power_plan",
        }
    }
}

/// Static policy attached to an elevated agent command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentCommandPolicy {
    /// Typed command identifier.
    pub command_id: AgentCommandId,
    /// Required payload schema.
    pub payload_kind: AgentPayloadKind,
    /// Command risk class.
    pub risk: AgentCommandRisk,
    /// Whether the command requires a rollback reference before authorization.
    pub requires_rollback_reference: bool,
    /// Whether denied attempts must be written to the audit sink.
    pub audit_denials: bool,
}

const ACTIVATE_PREPARED_POWER_PLAN_POLICY: AgentCommandPolicy = AgentCommandPolicy {
    command_id: AgentCommandId::ActivatePreparedPowerPlan,
    payload_kind: AgentPayloadKind::ActivatePreparedPowerPlan,
    risk: AgentCommandRisk::Elevated,
    requires_rollback_reference: true,
    audit_denials: true,
};

const AGENT_ALLOWLIST: &[AgentCommandPolicy] = &[ACTIVATE_PREPARED_POWER_PLAN_POLICY];

/// Returns the elevated agent deny-by-default allowlist.
#[must_use]
pub const fn agent_allowlist() -> &'static [AgentCommandPolicy] {
    AGENT_ALLOWLIST
}

/// Validated requester identity for an agent request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequester {
    value: String,
}

impl AgentRequester {
    /// Creates a requester label accepted by the agent boundary.
    pub fn new(value: impl Into<String>) -> Result<Self, AgentDenial> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty()
            || trimmed.len() > MAX_REQUESTER_LEN
            || !trimmed.bytes().all(is_allowed_label_byte)
        {
            return Err(AgentDenial::invalid_requester());
        }

        Ok(Self {
            value: trimmed.to_owned(),
        })
    }

    /// Returns the validated requester string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Rollback snapshot reference required before privileged mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackReference {
    value: String,
}

impl RollbackReference {
    /// Creates a local rollback reference for audit and recovery records.
    pub fn new(value: impl Into<String>) -> Result<Self, AgentDenial> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty()
            || trimmed.len() > MAX_ROLLBACK_REFERENCE_LEN
            || !trimmed.bytes().all(is_allowed_label_byte)
        {
            return Err(AgentDenial::invalid_rollback_reference());
        }

        Ok(Self {
            value: trimmed.to_owned(),
        })
    }

    /// Returns the rollback reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Validated Windows power scheme GUID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSchemeGuid {
    value: String,
}

impl PowerSchemeGuid {
    /// Creates a power scheme GUID accepted by `powercfg`.
    pub fn new(value: impl Into<String>) -> Result<Self, AgentDenial> {
        let value = value.into();
        let trimmed = value.trim();

        if !is_power_scheme_guid(trimmed) {
            return Err(AgentDenial::invalid_power_scheme_guid());
        }

        Ok(Self {
            value: trimmed.to_owned(),
        })
    }

    /// Returns the validated GUID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_power_scheme_guid(value: &str) -> bool {
    value.len() == POWER_SCHEME_GUID_LEN
        && value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 8 | 13 | 18 | 23) == (byte == b'-')
                && (byte == b'-' || byte.is_ascii_hexdigit())
        })
}

fn is_allowed_label_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
}

/// Payload for activating a previously prepared power plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatePreparedPowerPlanPayload {
    scheme_guid: PowerSchemeGuid,
    rollback_reference: RollbackReference,
}

impl ActivatePreparedPowerPlanPayload {
    /// Creates the typed payload for a prepared power plan activation.
    pub fn new(
        scheme_guid: impl Into<String>,
        rollback_reference: impl Into<String>,
    ) -> Result<Self, AgentDenial> {
        Ok(Self {
            scheme_guid: PowerSchemeGuid::new(scheme_guid)?,
            rollback_reference: RollbackReference::new(rollback_reference)?,
        })
    }

    /// Returns the power scheme GUID.
    #[must_use]
    pub fn scheme_guid(&self) -> &PowerSchemeGuid {
        &self.scheme_guid
    }

    /// Returns the rollback reference attached to the request.
    #[must_use]
    pub fn rollback_reference(&self) -> &RollbackReference {
        &self.rollback_reference
    }
}

/// Typed agent payload variants accepted by the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentCommandPayload {
    /// No schema was supplied.
    Empty,
    /// Activate a prepared power plan.
    ActivatePreparedPowerPlan(ActivatePreparedPowerPlanPayload),
}

impl AgentCommandPayload {
    fn kind(&self) -> Option<AgentPayloadKind> {
        match self {
            Self::Empty => None,
            Self::ActivatePreparedPowerPlan(_) => {
                Some(AgentPayloadKind::ActivatePreparedPowerPlan)
            }
        }
    }

    fn rollback_reference(&self) -> Option<&RollbackReference> {
        match self {
            Self::Empty => None,
            Self::ActivatePreparedPowerPlan(payload) => Some(payload.rollback_reference()),
        }
    }
}

/// Raw agent request before authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommandRequest {
    command_id: String,
    requester: AgentRequester,
    payload: AgentCommandPayload,
}

impl AgentCommandRequest {
    /// Creates a request for allowlist authorization.
    pub fn new(
        command_id: impl Into<String>,
        requester: impl Into<String>,
        payload: AgentCommandPayload,
    ) -> Result<Self, AgentDenial> {
        Ok(Self {
            command_id: command_id.into(),
            requester: AgentRequester::new(requester)?,
            payload,
        })
    }

    /// Creates a typed request for prepared power plan activation.
    pub fn activate_prepared_power_plan(
        requester: impl Into<String>,
        payload: ActivatePreparedPowerPlanPayload,
    ) -> Result<Self, AgentDenial> {
        Self::new(
            ACTIVATE_PREPARED_POWER_PLAN_COMMAND,
            requester,
            AgentCommandPayload::ActivatePreparedPowerPlan(payload),
        )
    }
}

/// Agent command authorized for privileged execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedAgentCommand {
    command_id: AgentCommandId,
    requester: AgentRequester,
    command_plan: StructuredCommandPlan,
    rollback_reference: RollbackReference,
    policy: AgentCommandPolicy,
}

impl AuthorizedAgentCommand {
    /// Returns the authorized command ID.
    #[must_use]
    pub const fn command_id(&self) -> AgentCommandId {
        self.command_id
    }

    /// Returns the requester attached to the command.
    #[must_use]
    pub fn requester(&self) -> &AgentRequester {
        &self.requester
    }

    /// Returns the fixed executable plan.
    #[must_use]
    pub fn command_plan(&self) -> &StructuredCommandPlan {
        &self.command_plan
    }

    /// Returns the rollback reference required for recovery.
    #[must_use]
    pub fn rollback_reference(&self) -> &RollbackReference {
        &self.rollback_reference
    }

    /// Returns the allowlist policy that authorized this command.
    #[must_use]
    pub const fn policy(&self) -> AgentCommandPolicy {
        self.policy
    }
}

/// Audit outcome for an agent authorization attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAuditOutcome {
    /// Request passed allowlist and payload checks.
    Authorized,
    /// Request was denied before privileged execution.
    Denied,
}

impl AgentAuditOutcome {
    /// Returns a stable audit outcome string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authorized => "authorized",
            Self::Denied => "denied",
        }
    }
}

/// Local audit event emitted by the elevated agent boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAuditEvent {
    command_id: String,
    requester: String,
    outcome: AgentAuditOutcome,
    reason: Option<AgentDenialReason>,
    rollback_reference: Option<String>,
}

impl AgentAuditEvent {
    fn authorized(command: &AuthorizedAgentCommand) -> Self {
        Self {
            command_id: command.command_id().as_str().to_owned(),
            requester: command.requester().as_str().to_owned(),
            outcome: AgentAuditOutcome::Authorized,
            reason: None,
            rollback_reference: Some(command.rollback_reference().as_str().to_owned()),
        }
    }

    fn denied(request: &AgentCommandRequest, denial: &AgentDenial) -> Self {
        Self {
            command_id: request.command_id.clone(),
            requester: request.requester.as_str().to_owned(),
            outcome: AgentAuditOutcome::Denied,
            reason: Some(denial.reason()),
            rollback_reference: request
                .payload
                .rollback_reference()
                .map(|reference| reference.as_str().to_owned()),
        }
    }

    /// Returns the audited command string.
    #[must_use]
    pub fn command_id(&self) -> &str {
        &self.command_id
    }

    /// Returns the audited requester string.
    #[must_use]
    pub fn requester(&self) -> &str {
        &self.requester
    }

    /// Returns whether the request was authorized or denied.
    #[must_use]
    pub const fn outcome(&self) -> AgentAuditOutcome {
        self.outcome
    }

    /// Returns the denial reason when the request was denied.
    #[must_use]
    pub const fn reason(&self) -> Option<AgentDenialReason> {
        self.reason
    }

    /// Returns the rollback reference, when one was supplied.
    #[must_use]
    pub fn rollback_reference(&self) -> Option<&str> {
        self.rollback_reference.as_deref()
    }
}

/// Storage sink for local elevated-agent audit records.
pub trait AgentAuditSink {
    /// Records an audit event.
    fn record(&mut self, event: AgentAuditEvent);
}

/// In-memory local audit store for tests and future persistence adapters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryAgentAuditLog {
    events: Vec<AgentAuditEvent>,
}

impl InMemoryAgentAuditLog {
    /// Returns the audit events recorded by the local store.
    #[must_use]
    pub fn events(&self) -> &[AgentAuditEvent] {
        &self.events
    }
}

impl AgentAuditSink for InMemoryAgentAuditLog {
    fn record(&mut self, event: AgentAuditEvent) {
        self.events.push(event);
    }
}

/// Authorizes a typed agent request and records the decision locally.
pub fn authorize_agent_request<S>(
    request: AgentCommandRequest,
    audit_sink: &mut S,
) -> Result<AuthorizedAgentCommand, AgentDenial>
where
    S: AgentAuditSink,
{
    let command_id = match AgentCommandId::parse(&request.command_id) {
        Ok(command_id) => command_id,
        Err(denial) => {
            audit_sink.record(AgentAuditEvent::denied(&request, &denial));
            return Err(denial);
        }
    };

    let Some(policy) = agent_allowlist()
        .iter()
        .copied()
        .find(|policy| policy.command_id == command_id)
    else {
        let denial = AgentDenial::command_not_allowlisted(command_id.as_str());
        audit_sink.record(AgentAuditEvent::denied(&request, &denial));
        return Err(denial);
    };

    if request.payload.kind() != Some(policy.payload_kind) {
        let denial = AgentDenial::invalid_payload(command_id.as_str());
        audit_sink.record(AgentAuditEvent::denied(&request, &denial));
        return Err(denial);
    }

    if policy.requires_rollback_reference && request.payload.rollback_reference().is_none() {
        let denial = AgentDenial::missing_rollback_reference(command_id.as_str());
        audit_sink.record(AgentAuditEvent::denied(&request, &denial));
        return Err(denial);
    }

    let command_plan = match command_plan_for_payload(&request.payload) {
        Ok(command_plan) => command_plan,
        Err(_) => {
            let denial = AgentDenial::unsafe_command_plan(command_id.as_str());
            audit_sink.record(AgentAuditEvent::denied(&request, &denial));
            return Err(denial);
        }
    };

    let rollback_reference = request
        .payload
        .rollback_reference()
        .expect("rollback reference checked above")
        .clone();

    let command = AuthorizedAgentCommand {
        command_id,
        requester: request.requester,
        command_plan,
        rollback_reference,
        policy,
    };

    audit_sink.record(AgentAuditEvent::authorized(&command));

    Ok(command)
}

fn command_plan_for_payload(
    payload: &AgentCommandPayload,
) -> Result<StructuredCommandPlan, WindowsCommandPlanError> {
    match payload {
        AgentCommandPayload::ActivatePreparedPowerPlan(payload) => {
            StructuredCommandPlan::powercfg_activate_scheme(payload.scheme_guid().as_str())
        }
        AgentCommandPayload::Empty => Err(WindowsCommandPlanError::InvalidArgument),
    }
}

/// Denial reason emitted by the elevated agent boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentDenialReason {
    /// Command ID is not registered with the agent.
    UnknownCommand,
    /// Command ID is registered but not enabled in the allowlist.
    CommandNotAllowlisted,
    /// Payload did not match the command schema.
    InvalidPayload,
    /// Requester identity failed validation.
    InvalidRequester,
    /// A privileged mutation was requested without a rollback reference.
    MissingRollbackReference,
    /// Power scheme GUID failed validation.
    InvalidPowerSchemeGuid,
    /// Rollback reference failed validation.
    InvalidRollbackReference,
    /// Command plan failed fixed executable or structured argument checks.
    UnsafeCommandPlan,
}

impl AgentDenialReason {
    /// Returns a stable reason code for audit records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCommand => "unknown_command",
            Self::CommandNotAllowlisted => "command_not_allowlisted",
            Self::InvalidPayload => "invalid_payload",
            Self::InvalidRequester => "invalid_requester",
            Self::MissingRollbackReference => "missing_rollback_reference",
            Self::InvalidPowerSchemeGuid => "invalid_power_scheme_guid",
            Self::InvalidRollbackReference => "invalid_rollback_reference",
            Self::UnsafeCommandPlan => "unsafe_command_plan",
        }
    }

    /// Returns a short human-readable denial message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnknownCommand => "Agent command is not registered",
            Self::CommandNotAllowlisted => "Agent command is not allowlisted",
            Self::InvalidPayload => "Agent command payload failed validation",
            Self::InvalidRequester => "Agent requester failed validation",
            Self::MissingRollbackReference => {
                "Agent command requires a rollback reference"
            }
            Self::InvalidPowerSchemeGuid => "Power scheme GUID failed validation",
            Self::InvalidRollbackReference => "Rollback reference failed validation",
            Self::UnsafeCommandPlan => "Agent command plan failed safety checks",
        }
    }
}

/// Structured denial returned by the agent boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDenial {
    reason: AgentDenialReason,
    command_id: Option<String>,
}

impl AgentDenial {
    fn unknown_command(command_id: impl Into<String>) -> Self {
        Self {
            reason: AgentDenialReason::UnknownCommand,
            command_id: Some(command_id.into()),
        }
    }

    fn invalid_payload(command_id: impl Into<String>) -> Self {
        Self {
            reason: AgentDenialReason::InvalidPayload,
            command_id: Some(command_id.into()),
        }
    }

    fn command_not_allowlisted(command_id: impl Into<String>) -> Self {
        Self {
            reason: AgentDenialReason::CommandNotAllowlisted,
            command_id: Some(command_id.into()),
        }
    }

    fn invalid_requester() -> Self {
        Self {
            reason: AgentDenialReason::InvalidRequester,
            command_id: None,
        }
    }

    fn missing_rollback_reference(command_id: impl Into<String>) -> Self {
        Self {
            reason: AgentDenialReason::MissingRollbackReference,
            command_id: Some(command_id.into()),
        }
    }

    fn invalid_power_scheme_guid() -> Self {
        Self {
            reason: AgentDenialReason::InvalidPowerSchemeGuid,
            command_id: Some(ACTIVATE_PREPARED_POWER_PLAN_COMMAND.to_owned()),
        }
    }

    fn invalid_rollback_reference() -> Self {
        Self {
            reason: AgentDenialReason::InvalidRollbackReference,
            command_id: Some(ACTIVATE_PREPARED_POWER_PLAN_COMMAND.to_owned()),
        }
    }

    fn unsafe_command_plan(command_id: impl Into<String>) -> Self {
        Self {
            reason: AgentDenialReason::UnsafeCommandPlan,
            command_id: Some(command_id.into()),
        }
    }

    /// Returns the denial reason.
    #[must_use]
    pub const fn reason(&self) -> AgentDenialReason {
        self.reason
    }

    /// Returns the denied command ID when available.
    #[must_use]
    pub fn command_id(&self) -> Option<&str> {
        self.command_id.as_deref()
    }

    /// Returns a short human-readable denial message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.reason.message()
    }
}

impl fmt::Display for AgentDenial {
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

impl std::error::Error for AgentDenial {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "windows-agent");
        assert!(info.responsibility.contains("allowlisted"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn denies_unknown_command_and_audits_the_attempt() {
        let request = AgentCommandRequest::new(
            "agent.run_shell",
            "main-window",
            AgentCommandPayload::Empty,
        )
        .expect("requester should be valid");
        let mut audit_log = InMemoryAgentAuditLog::default();

        let denial = authorize_agent_request(request, &mut audit_log)
            .expect_err("unknown commands must be denied");

        assert_eq!(denial.reason(), AgentDenialReason::UnknownCommand);
        assert_eq!(denial.command_id(), Some("agent.run_shell"));
        assert_eq!(audit_log.events().len(), 1);
        assert_eq!(audit_log.events()[0].command_id(), "agent.run_shell");
        assert_eq!(audit_log.events()[0].requester(), "main-window");
        assert_eq!(audit_log.events()[0].outcome(), AgentAuditOutcome::Denied);
        assert_eq!(
            audit_log.events()[0].reason(),
            Some(AgentDenialReason::UnknownCommand)
        );
    }

    #[test]
    fn authorizes_prepared_power_plan_with_fixed_command_and_rollback() {
        let payload = ActivatePreparedPowerPlanPayload::new(
            "381b4222-f694-41f0-9685-ff5bb260df2e",
            "rollback:snapshot-001",
        )
        .expect("typed payload should be valid");
        let request =
            AgentCommandRequest::activate_prepared_power_plan("main-window", payload)
                .expect("request should be valid");
        let mut audit_log = InMemoryAgentAuditLog::default();

        let command = authorize_agent_request(request, &mut audit_log)
            .expect("allowlisted command should be authorized");

        assert_eq!(
            command.command_id(),
            AgentCommandId::ActivatePreparedPowerPlan
        );
        assert_eq!(command.policy().risk, AgentCommandRisk::Elevated);
        assert_eq!(
            command.command_plan().executable().path(),
            "C:\\Windows\\System32\\powercfg.exe"
        );
        assert_eq!(command.command_plan().arguments()[0].as_str(), "/setactive");
        assert_eq!(
            command.command_plan().arguments()[1].as_str(),
            "381b4222-f694-41f0-9685-ff5bb260df2e"
        );
        assert_eq!(command.rollback_reference().as_str(), "rollback:snapshot-001");
        assert_eq!(audit_log.events().len(), 1);
        assert_eq!(audit_log.events()[0].outcome(), AgentAuditOutcome::Authorized);
        assert_eq!(
            audit_log.events()[0].rollback_reference(),
            Some("rollback:snapshot-001")
        );
    }

    #[test]
    fn denies_payload_schema_mismatch_and_audits_the_attempt() {
        let request = AgentCommandRequest::new(
            ACTIVATE_PREPARED_POWER_PLAN_COMMAND,
            "main-window",
            AgentCommandPayload::Empty,
        )
        .expect("requester should be valid");
        let mut audit_log = InMemoryAgentAuditLog::default();

        let denial = authorize_agent_request(request, &mut audit_log)
            .expect_err("missing typed payload must be denied");

        assert_eq!(denial.reason(), AgentDenialReason::InvalidPayload);
        assert_eq!(audit_log.events().len(), 1);
        assert_eq!(audit_log.events()[0].outcome(), AgentAuditOutcome::Denied);
        assert_eq!(
            audit_log.events()[0].reason(),
            Some(AgentDenialReason::InvalidPayload)
        );
    }

    #[test]
    fn rejects_shell_like_power_scheme_guid_before_authorization() {
        let denial = ActivatePreparedPowerPlanPayload::new(
            "381b4222-f694-41f0-9685-ff5bb260df2e && calc.exe",
            "rollback:snapshot-001",
        )
        .expect_err("shell-like GUID must be rejected");

        assert_eq!(denial.reason(), AgentDenialReason::InvalidPowerSchemeGuid);
    }
}
