//! PUBG discovery, settings inspection, and safe recommendation planning.

use std::collections::BTreeSet;

/// PUBG client executable used by Steam/Epic installs.
pub const PUBG_EXECUTABLE_NAME: &str = "TslGame.exe";

/// PUBG-owned process names that block driver profile mutation while running.
pub const PUBG_PROCESS_NAMES: &[&str] = &[PUBG_EXECUTABLE_NAME, "TslGame_BE.exe"];

/// BattlEye process names observed in PUBG sessions.
pub const BATTLEYE_PROCESS_NAMES: &[&str] = &["BEService.exe", "BEService_x64.exe"];

/// Read-only snapshot of PUBG and anti-cheat processes relevant to profile mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgRuntimeState {
    /// Canonical process names currently running.
    pub running_processes: Vec<String>,
}

impl PubgRuntimeState {
    /// Creates a runtime state with no PUBG or BattlEye process evidence.
    #[must_use]
    pub fn no_processes() -> Self {
        Self {
            running_processes: Vec::new(),
        }
    }

    /// Creates a runtime state from process names or executable paths.
    #[must_use]
    pub fn from_process_names<I, S>(process_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            running_processes: normalized_process_names(process_names),
        }
    }

    /// Returns PUBG or BattlEye processes that require deferring driver profile writes.
    #[must_use]
    pub fn blocking_profile_mutation_processes(&self) -> Vec<String> {
        self.running_processes
            .iter()
            .filter(|process| is_pubg_or_battleye_process_name(process))
            .cloned()
            .collect()
    }

    /// Returns true when NVIDIA/AMD profile changes can proceed without touching a live session.
    #[must_use]
    pub fn allows_profile_mutation(&self) -> bool {
        self.blocking_profile_mutation_processes().is_empty()
    }
}

/// Returns true for PUBG or BattlEye process names that block profile mutation.
#[must_use]
pub fn is_pubg_or_battleye_process_name(process_name: &str) -> bool {
    let process_name = canonical_process_name(process_name);
    PUBG_PROCESS_NAMES
        .iter()
        .chain(BATTLEYE_PROCESS_NAMES.iter())
        .any(|candidate| candidate.eq_ignore_ascii_case(&process_name))
}

fn normalized_process_names<I, S>(process_names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    process_names
        .into_iter()
        .map(Into::into)
        .map(|process_name| canonical_process_name(&process_name))
        .filter(|process_name| !process_name.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn canonical_process_name(process_name: &str) -> String {
    let trimmed = process_name.trim().trim_matches('"');
    trimmed
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_owned()
}

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

/// PUBG crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "pubg",
    responsibility: "discover PUBG installs, inspect settings, and protect anti-cheat boundaries",
    requires_live_windows: true,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "pubg");
        assert!(info.responsibility.contains("anti-cheat"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn identifies_pubg_and_battleye_process_names() {
        assert!(is_pubg_or_battleye_process_name("TslGame.exe"));
        assert!(is_pubg_or_battleye_process_name(
            r"C:\Games\PUBG\TslGame_BE.exe"
        ));
        assert!(is_pubg_or_battleye_process_name("beservice_x64.exe"));
        assert!(!is_pubg_or_battleye_process_name("steam.exe"));
    }

    #[test]
    fn runtime_state_reports_profile_mutation_blockers() {
        let state = PubgRuntimeState::from_process_names([
            r"C:\Games\PUBG\TslGame.exe",
            "Discord.exe",
            "BEService.exe",
        ]);

        assert_eq!(
            state.blocking_profile_mutation_processes(),
            vec!["BEService.exe".to_owned(), "TslGame.exe".to_owned()]
        );
        assert!(!state.allows_profile_mutation());
        assert!(PubgRuntimeState::no_processes().allows_profile_mutation());
    }
}
