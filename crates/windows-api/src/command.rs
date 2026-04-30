//! Structured Windows command plans for privileged adapters.

use std::fmt;

const MAX_COMMAND_ARGUMENTS: usize = 16;
const MAX_ARGUMENT_LEN: usize = 128;

/// Fixed Windows executables the optimizer is allowed to invoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedWindowsExecutable {
    /// Windows power configuration utility.
    PowerCfg,
}

impl FixedWindowsExecutable {
    /// Returns a stable short name for logs and audit records.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PowerCfg => "powercfg",
        }
    }

    /// Returns the absolute executable path.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::PowerCfg => "C:\\Windows\\System32\\powercfg.exe",
        }
    }
}

/// A single validated process argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsArgument {
    value: String,
}

impl WindowsArgument {
    /// Creates a structured argument with shell-control bytes rejected.
    pub fn new(value: impl Into<String>) -> Result<Self, WindowsCommandPlanError> {
        let value = value.into();

        if value.is_empty() || value.len() > MAX_ARGUMENT_LEN {
            return Err(WindowsCommandPlanError::InvalidArgument);
        }

        if value.bytes().any(is_unsafe_argument_byte) {
            return Err(WindowsCommandPlanError::UnsafeArgument);
        }

        Ok(Self { value })
    }

    /// Returns the validated argument string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_unsafe_argument_byte(byte: u8) -> bool {
    matches!(
        byte,
        0 | b'\r' | b'\n' | b'|' | b'&' | b';' | b'`' | b'<' | b'>'
    )
}

/// A fixed executable with validated, structured arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredCommandPlan {
    executable: FixedWindowsExecutable,
    arguments: Vec<WindowsArgument>,
}

impl StructuredCommandPlan {
    /// Creates a command plan from a fixed executable and structured arguments.
    pub fn new<I, S>(
        executable: FixedWindowsExecutable,
        arguments: I,
    ) -> Result<Self, WindowsCommandPlanError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let arguments = arguments
            .into_iter()
            .map(WindowsArgument::new)
            .collect::<Result<Vec<_>, _>>()?;

        if arguments.len() > MAX_COMMAND_ARGUMENTS {
            return Err(WindowsCommandPlanError::TooManyArguments);
        }

        Ok(Self {
            executable,
            arguments,
        })
    }

    /// Builds the fixed command plan for activating a prepared power scheme.
    pub fn powercfg_activate_scheme(
        scheme_guid: &str,
    ) -> Result<Self, WindowsCommandPlanError> {
        Self::new(FixedWindowsExecutable::PowerCfg, ["/setactive", scheme_guid])
    }

    /// Builds the fixed command plan for querying the active power scheme.
    pub fn powercfg_query_active_scheme() -> Result<Self, WindowsCommandPlanError> {
        Self::new(FixedWindowsExecutable::PowerCfg, ["/getactivescheme"])
    }

    /// Returns the fixed executable.
    #[must_use]
    pub const fn executable(&self) -> FixedWindowsExecutable {
        self.executable
    }

    /// Returns the validated argument list.
    #[must_use]
    pub fn arguments(&self) -> &[WindowsArgument] {
        &self.arguments
    }
}

/// Errors raised while building a structured Windows command plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsCommandPlanError {
    /// Argument is empty or too long for this boundary.
    InvalidArgument,
    /// Argument count exceeds the narrow command contract.
    TooManyArguments,
    /// Argument contains shell-control bytes.
    UnsafeArgument,
}

impl WindowsCommandPlanError {
    /// Returns a stable error code for audit records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "invalid_argument",
            Self::TooManyArguments => "too_many_arguments",
            Self::UnsafeArgument => "unsafe_argument",
        }
    }

    /// Returns a human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidArgument => "Windows command argument failed validation",
            Self::TooManyArguments => "Windows command has too many arguments",
            Self::UnsafeArgument => "Windows command argument contains shell-control bytes",
        }
    }
}

impl fmt::Display for WindowsCommandPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.as_str(), self.message())
    }
}

impl std::error::Error for WindowsCommandPlanError {}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SCHEME_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

    #[test]
    fn powercfg_plan_uses_fixed_executable_path() {
        let plan = StructuredCommandPlan::powercfg_activate_scheme(TEST_SCHEME_GUID)
            .expect("static powercfg plan should be valid");

        assert_eq!(plan.executable(), FixedWindowsExecutable::PowerCfg);
        assert_eq!(
            plan.executable().path(),
            "C:\\Windows\\System32\\powercfg.exe"
        );
        assert_eq!(plan.arguments()[0].as_str(), "/setactive");
        assert_eq!(plan.arguments()[1].as_str(), TEST_SCHEME_GUID);
    }

    #[test]
    fn rejects_shell_control_argument_bytes() {
        let error =
            StructuredCommandPlan::new(FixedWindowsExecutable::PowerCfg, ["x && calc.exe"])
                .expect_err("shell-control arguments must be denied");

        assert_eq!(error, WindowsCommandPlanError::UnsafeArgument);
    }
}
