//! PUBG discovery, settings inspection, and safe recommendation planning.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

/// PUBG client executable used by Steam/Epic installs.
pub const PUBG_EXECUTABLE_NAME: &str = "TslGame.exe";

/// Steam application ID for PUBG: BATTLEGROUNDS.
pub const PUBG_STEAM_APP_ID: &str = "578080";

/// PUBG-owned process names that block driver profile mutation while running.
pub const PUBG_PROCESS_NAMES: &[&str] = &[PUBG_EXECUTABLE_NAME, "TslGame_BE.exe"];

/// BattlEye process names observed in PUBG sessions.
pub const BATTLEYE_PROCESS_NAMES: &[&str] = &["BEService.exe", "BEService_x64.exe"];

/// Config directories used by current and legacy PUBG clients.
pub const PUBG_CONFIG_DIR_NAMES: &[&str] = &["WindowsClient", "WindowsNoEditor"];

/// Supported config files detected by T070 without parsing their contents.
pub const PUBG_CONFIG_FILE_NAMES: &[&str] =
    &["GameUserSettings.ini", "Engine.ini", "Scalability.ini", "Input.ini"];

/// Launcher family that provided installation metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PubgLauncher {
    /// Steam app manifest and library metadata.
    Steam,
    /// Epic Games Launcher item manifest metadata.
    Epic,
}

impl PubgLauncher {
    /// Returns a stable lowercase label for telemetry and test fixtures.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steam => "steam",
            Self::Epic => "epic",
        }
    }

    /// Returns the launcher name shown to users.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Epic => "Epic Games",
        }
    }
}

/// Read-only launcher metadata used to explain where an install was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgLauncherMetadata {
    /// Launcher metadata file, such as `appmanifest_578080.acf` or an Epic `.item`.
    pub path: PathBuf,
    /// Stable launcher identifier when exposed, such as the Steam app ID or Epic app name.
    pub identifier: Option<String>,
    /// User-facing game name read from launcher metadata when available.
    pub display_name: Option<String>,
}

impl PubgLauncherMetadata {
    /// Creates launcher metadata with optional identifier and display name.
    #[must_use]
    pub fn new(
        path: impl Into<PathBuf>,
        identifier: Option<String>,
        display_name: Option<String>,
    ) -> Self {
        Self {
            path: path.into(),
            identifier,
            display_name,
        }
    }
}

/// Read-only BattlEye file presence discovered under a PUBG install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgBattleEyePresence {
    /// Existing BattlEye paths found by the discovery pass.
    pub paths: Vec<PathBuf>,
}

impl PubgBattleEyePresence {
    /// Creates an empty BattlEye presence record.
    #[must_use]
    pub fn absent() -> Self {
        Self { paths: Vec::new() }
    }

    /// Returns true when BattlEye files or directories were found.
    #[must_use]
    pub fn is_present(&self) -> bool {
        !self.paths.is_empty()
    }
}

/// One PUBG install candidate discovered from store metadata or conventional paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgInstallation {
    /// Launcher family that owns this installation.
    pub launcher: PubgLauncher,
    /// PUBG install root, usually the folder containing `TslGame`.
    pub install_dir: PathBuf,
    /// Existing `TslGame.exe` path when found.
    pub executable_path: Option<PathBuf>,
    /// Store metadata that led to this install candidate.
    pub metadata: Option<PubgLauncherMetadata>,
    /// Read-only BattlEye file presence under this install.
    pub battleye: PubgBattleEyePresence,
}

impl PubgInstallation {
    /// Returns true when the install has an existing `TslGame.exe`.
    #[must_use]
    pub fn has_executable(&self) -> bool {
        self.executable_path.is_some()
    }

    /// Returns true when BattlEye files or directories were found under the install.
    #[must_use]
    pub fn has_battleye(&self) -> bool {
        self.battleye.is_present()
    }
}

/// PUBG user config paths discovered from local app data roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgConfigDiscovery {
    /// Existing config directories such as `Saved\Config\WindowsClient`.
    pub directories: Vec<PathBuf>,
    /// Known config files found inside the detected config directories.
    pub files: Vec<PathBuf>,
}

impl PubgConfigDiscovery {
    /// Creates an empty config discovery record.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            directories: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Returns true when config directories or files were found.
    #[must_use]
    pub fn is_present(&self) -> bool {
        !self.directories.is_empty() || !self.files.is_empty()
    }
}

/// Filesystem roots used by read-only PUBG discovery.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PubgDiscoveryRoots {
    /// Steam install roots or library roots to inspect.
    pub steam_roots: Vec<PathBuf>,
    /// Epic Games Launcher manifest directories to inspect.
    pub epic_manifest_dirs: Vec<PathBuf>,
    /// Local app data roots to inspect for PUBG config folders.
    pub local_app_data_roots: Vec<PathBuf>,
}

impl PubgDiscoveryRoots {
    /// Creates an empty discovery-root set for fixture or caller-provided paths.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a Steam install root or library root.
    #[must_use]
    pub fn with_steam_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.steam_roots.push(root.into());
        self
    }

    /// Adds an Epic Games Launcher manifest directory.
    #[must_use]
    pub fn with_epic_manifest_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.epic_manifest_dirs.push(dir.into());
        self
    }

    /// Adds a local app data root for config detection.
    #[must_use]
    pub fn with_local_app_data_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.local_app_data_roots.push(root.into());
        self
    }
}

/// Full read-only PUBG discovery report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgDiscoveryReport {
    /// Store-backed install candidates.
    pub installations: Vec<PubgInstallation>,
    /// Config directories and files found in local app data.
    pub config: PubgConfigDiscovery,
}

impl PubgDiscoveryReport {
    /// Returns true when any install candidate has an executable.
    #[must_use]
    pub fn is_installed(&self) -> bool {
        self.installations
            .iter()
            .any(PubgInstallation::has_executable)
    }

    /// Returns the first executable-backed install, or the first metadata-only candidate.
    #[must_use]
    pub fn primary_installation(&self) -> Option<&PubgInstallation> {
        self.installations
            .iter()
            .find(|installation| installation.has_executable())
            .or_else(|| self.installations.first())
    }
}

/// Builds default Windows discovery roots from environment variables.
#[must_use]
pub fn default_pubg_discovery_roots() -> PubgDiscoveryRoots {
    let mut roots = PubgDiscoveryRoots::new();

    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(program_files) = env::var(variable) {
            roots = roots.with_steam_root(PathBuf::from(program_files).join("Steam"));
        }
    }

    if let Ok(program_data) = env::var("ProgramData") {
        roots = roots.with_epic_manifest_dir(
            PathBuf::from(program_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        );
    }

    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        roots = roots.with_local_app_data_root(local_app_data);
    }

    roots
}

/// Discovers PUBG install, config, launcher metadata, and BattlEye presence.
///
/// This is a read-only scan and never mutates game files, BattlEye files, or launcher metadata.
#[must_use]
pub fn discover_pubg() -> PubgDiscoveryReport {
    discover_pubg_from_roots(&default_pubg_discovery_roots())
}

/// Discovers PUBG from caller-provided roots for fixtures and platform adapters.
///
/// This is a read-only scan and never mutates game files, BattlEye files, or launcher metadata.
#[must_use]
pub fn discover_pubg_from_roots(roots: &PubgDiscoveryRoots) -> PubgDiscoveryReport {
    let mut installations = Vec::new();
    let mut seen_installations = BTreeSet::new();

    for installation in discover_steam_installations(&roots.steam_roots)
        .into_iter()
        .chain(discover_epic_installations(&roots.epic_manifest_dirs))
    {
        if seen_installations.insert(installation_key(&installation)) {
            installations.push(installation);
        }
    }

    installations.sort_by(|left, right| {
        left.launcher
            .cmp(&right.launcher)
            .then_with(|| left.install_dir.cmp(&right.install_dir))
    });

    PubgDiscoveryReport {
        installations,
        config: discover_pubg_configs(&roots.local_app_data_roots),
    }
}

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

fn discover_steam_installations(steam_roots: &[PathBuf]) -> Vec<PubgInstallation> {
    let mut installations = Vec::new();

    for library_root in steam_library_candidates(steam_roots) {
        if let Some(installation) = discover_steam_installation_from_library(&library_root) {
            installations.push(installation);
        }
    }

    installations
}

fn steam_library_candidates(steam_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates = BTreeSet::new();

    for steam_root in steam_roots {
        candidates.insert(steam_root.clone());

        let library_folders = steam_root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(source) = fs::read_to_string(library_folders) {
            for path in quoted_metadata_values(&source, "path") {
                if !path.trim().is_empty() {
                    candidates.insert(PathBuf::from(path));
                }
            }
        }
    }

    candidates.into_iter().collect()
}

fn discover_steam_installation_from_library(library_root: &Path) -> Option<PubgInstallation> {
    let steamapps_dir = library_root.join("steamapps");
    let manifest_path = steamapps_dir.join(format!("appmanifest_{PUBG_STEAM_APP_ID}.acf"));

    if manifest_path.is_file() {
        let source = fs::read_to_string(&manifest_path).unwrap_or_default();
        let install_dir_name = quoted_metadata_value(&source, "installdir")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "PUBG".to_owned());
        let install_dir = steamapps_dir.join("common").join(install_dir_name);
        let metadata = PubgLauncherMetadata::new(
            manifest_path,
            Some(
                quoted_metadata_value(&source, "appid")
                    .unwrap_or_else(|| PUBG_STEAM_APP_ID.to_owned()),
            ),
            quoted_metadata_value(&source, "name"),
        );

        return Some(build_installation(
            PubgLauncher::Steam,
            install_dir,
            Some(metadata),
            None,
        ));
    }

    let conventional_install_dir = steamapps_dir.join("common").join("PUBG");
    find_existing_pubg_executable(&conventional_install_dir, None).map(|_| {
        build_installation(
            PubgLauncher::Steam,
            conventional_install_dir,
            None,
            None,
        )
    })
}

fn discover_epic_installations(manifest_dirs: &[PathBuf]) -> Vec<PubgInstallation> {
    let mut installations = Vec::new();

    for manifest_dir in manifest_dirs {
        let Ok(entries) = fs::read_dir(manifest_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let manifest_path = entry.path();
            if !is_epic_manifest_file(&manifest_path) {
                continue;
            }

            let Ok(source) = fs::read_to_string(&manifest_path) else {
                continue;
            };

            if !metadata_mentions_pubg(&source) {
                continue;
            }

            let Some(install_location) = quoted_metadata_value(&source, "InstallLocation") else {
                continue;
            };

            let launch_executable = quoted_metadata_value(&source, "LaunchExecutable");
            let install_dir = store_metadata_path(&install_location);
            let metadata = PubgLauncherMetadata::new(
                manifest_path,
                quoted_metadata_value(&source, "AppName")
                    .or_else(|| quoted_metadata_value(&source, "CatalogItemId")),
                quoted_metadata_value(&source, "DisplayName"),
            );

            installations.push(build_installation(
                PubgLauncher::Epic,
                install_dir,
                Some(metadata),
                launch_executable.as_deref(),
            ));
        }
    }

    installations
}

fn is_epic_manifest_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map_or(false, |extension| extension.eq_ignore_ascii_case("item"))
}

fn metadata_mentions_pubg(source: &str) -> bool {
    let normalized = source.to_ascii_lowercase();
    normalized.contains("pubg")
        || normalized.contains("tslgame")
        || normalized.contains("playerunknown")
        || normalized.contains(PUBG_STEAM_APP_ID)
}

fn build_installation(
    launcher: PubgLauncher,
    install_dir: PathBuf,
    metadata: Option<PubgLauncherMetadata>,
    launch_executable: Option<&str>,
) -> PubgInstallation {
    PubgInstallation {
        launcher,
        executable_path: find_existing_pubg_executable(&install_dir, launch_executable),
        battleye: discover_battleye_presence(&install_dir),
        install_dir,
        metadata,
    }
}

fn find_existing_pubg_executable(
    install_dir: &Path,
    launch_executable: Option<&str>,
) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(launch_executable) =
        launch_executable.and_then(|path| resolve_store_path(install_dir, path))
    {
        candidates.push(launch_executable);
    }

    candidates.extend([
        install_dir
            .join("TslGame")
            .join("Binaries")
            .join("Win64")
            .join(PUBG_EXECUTABLE_NAME),
        install_dir.join(PUBG_EXECUTABLE_NAME),
        install_dir
            .join("Binaries")
            .join("Win64")
            .join(PUBG_EXECUTABLE_NAME),
    ]);

    candidates
        .into_iter()
        .find(|candidate| is_pubg_executable_path(candidate) && candidate.is_file())
}

fn resolve_store_path(install_dir: &Path, raw_path: &str) -> Option<PathBuf> {
    let trimmed = raw_path.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return Some(path);
    }

    Some(join_store_relative_path(
        install_dir,
        trimmed.trim_start_matches(['\\', '/']),
    ))
}

fn store_metadata_path(raw_path: &str) -> PathBuf {
    let trimmed = raw_path.trim().trim_matches('"');
    let path = PathBuf::from(trimmed);

    if path.is_absolute() {
        path
    } else {
        join_store_relative_path(Path::new(""), trimmed.trim_start_matches(['\\', '/']))
    }
}

fn join_store_relative_path(root: &Path, relative_path: &str) -> PathBuf {
    let mut path = root.to_path_buf();

    for segment in relative_path
        .split(['\\', '/'])
        .filter(|segment| !segment.is_empty())
    {
        path.push(segment);
    }

    path
}

fn is_pubg_executable_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .map_or(false, |file_name| {
            file_name.eq_ignore_ascii_case(PUBG_EXECUTABLE_NAME)
        })
}

fn discover_battleye_presence(install_dir: &Path) -> PubgBattleEyePresence {
    let paths = battleye_candidate_paths(install_dir)
        .into_iter()
        .filter(|path| path.exists())
        .collect();

    PubgBattleEyePresence { paths }
}

fn battleye_candidate_paths(install_dir: &Path) -> Vec<PathBuf> {
    let battleye_dir = install_dir
        .join("TslGame")
        .join("Binaries")
        .join("Win64")
        .join("BattlEye");

    vec![
        battleye_dir.clone(),
        battleye_dir.join("BEService.exe"),
        battleye_dir.join("BEService_x64.exe"),
        battleye_dir.join("Install_BattlEye.bat"),
        install_dir.join("BattlEye"),
        install_dir.join("BattlEye").join("BEService.exe"),
        install_dir.join("BattlEye").join("BEService_x64.exe"),
    ]
}

fn discover_pubg_configs(local_app_data_roots: &[PathBuf]) -> PubgConfigDiscovery {
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();

    for local_app_data_root in local_app_data_roots {
        let config_root = local_app_data_root
            .join("TslGame")
            .join("Saved")
            .join("Config");

        for dir_name in PUBG_CONFIG_DIR_NAMES {
            let config_dir = config_root.join(dir_name);
            if !config_dir.is_dir() {
                continue;
            }

            directories.insert(config_dir.clone());

            for file_name in PUBG_CONFIG_FILE_NAMES {
                let config_file = config_dir.join(file_name);
                if config_file.is_file() {
                    files.insert(config_file);
                }
            }
        }
    }

    PubgConfigDiscovery {
        directories: directories.into_iter().collect(),
        files: files.into_iter().collect(),
    }
}

fn quoted_metadata_value(source: &str, key: &str) -> Option<String> {
    quoted_metadata_values(source, key).into_iter().next()
}

fn quoted_metadata_values(source: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\"");
    let mut values = Vec::new();
    let mut remainder = source;

    while let Some(index) = remainder.find(&needle) {
        let after_key = &remainder[index + needle.len()..];

        if let Some(value) = first_quoted_value(after_key) {
            values.push(value);
        }

        remainder = after_key;
    }

    values
}

fn first_quoted_value(source: &str) -> Option<String> {
    let start = source.find('"')? + 1;
    let rest = &source[start..];
    let end = rest.find('"')?;

    Some(unescape_metadata_string(&rest[..end]))
}

fn unescape_metadata_string(value: &str) -> String {
    value
        .replace("\\\\", "\\")
        .replace("\\/", "/")
        .replace("\\\"", "\"")
}

fn installation_key(installation: &PubgInstallation) -> String {
    format!(
        "{}:{}",
        installation.launcher.as_str(),
        installation.install_dir.to_string_lossy().to_ascii_lowercase()
    )
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
    use std::{
        env,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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

    #[test]
    fn discovers_steam_install_config_and_battleye_from_fixture() {
        let fixture = FixtureDir::new("steam");
        let steam_root = fixture.path().join("Steam");
        let steam_library = fixture.path().join("SteamLibrary");
        let install_dir = steam_library
            .join("steamapps")
            .join("common")
            .join("PUBG");
        let executable = install_dir
            .join("TslGame")
            .join("Binaries")
            .join("Win64")
            .join(PUBG_EXECUTABLE_NAME);
        let battleye = install_dir
            .join("TslGame")
            .join("Binaries")
            .join("Win64")
            .join("BattlEye")
            .join("BEService_x64.exe");
        let manifest = steam_library
            .join("steamapps")
            .join(format!("appmanifest_{PUBG_STEAM_APP_ID}.acf"));
        let local_app_data = fixture.path().join("LocalAppData");
        let config_file = local_app_data
            .join("TslGame")
            .join("Saved")
            .join("Config")
            .join("WindowsNoEditor")
            .join("GameUserSettings.ini");

        write_file(
            steam_root.join("steamapps").join("libraryfolders.vdf"),
            &format!(
                r#""libraryfolders"
{{
    "1"
    {{
        "path" "{}"
    }}
}}"#,
                metadata_path(&steam_library)
            ),
        );
        write_file(
            &manifest,
            r#""AppState"
{
    "appid" "578080"
    "name" "PUBG: BATTLEGROUNDS"
    "installdir" "PUBG"
}"#,
        );
        write_file(&executable, "");
        write_file(&battleye, "");
        write_file(&config_file, "[/Script/TslGame.TslGameUserSettings]\n");

        let report = discover_pubg_from_roots(
            &PubgDiscoveryRoots::new()
                .with_steam_root(steam_root)
                .with_local_app_data_root(local_app_data),
        );

        assert!(report.is_installed());
        assert!(report.config.is_present());
        assert_eq!(report.installations.len(), 1);

        let installation = report.primary_installation().expect("Steam PUBG install");
        assert_eq!(installation.launcher, PubgLauncher::Steam);
        assert_eq!(installation.install_dir, install_dir);
        assert_eq!(installation.executable_path.as_deref(), Some(executable.as_path()));
        assert!(installation.has_battleye());
        assert_eq!(
            installation.metadata.as_ref().and_then(|metadata| metadata.identifier.as_deref()),
            Some(PUBG_STEAM_APP_ID)
        );
        assert_eq!(
            report.config.files,
            vec![config_file]
        );
    }

    #[test]
    fn discovers_epic_install_from_manifest_fixture() {
        let fixture = FixtureDir::new("epic");
        let manifest_dir = fixture.path().join("EpicManifests");
        let install_dir = fixture.path().join("EpicGames").join("PUBG");
        let executable = install_dir
            .join("TslGame")
            .join("Binaries")
            .join("Win64")
            .join(PUBG_EXECUTABLE_NAME);
        let battleye_installer = install_dir
            .join("TslGame")
            .join("Binaries")
            .join("Win64")
            .join("BattlEye")
            .join("Install_BattlEye.bat");

        write_file(&executable, "");
        write_file(&battleye_installer, "");
        write_file(
            manifest_dir.join("pubg.item"),
            &format!(
                r#"{{
    "AppName": "pubg-live",
    "DisplayName": "PUBG: BATTLEGROUNDS",
    "InstallLocation": "{}",
    "LaunchExecutable": "TslGame\\Binaries\\Win64\\TslGame.exe"
}}"#,
                metadata_path(&install_dir)
            ),
        );

        let report = discover_pubg_from_roots(
            &PubgDiscoveryRoots::new().with_epic_manifest_dir(manifest_dir),
        );

        assert!(report.is_installed());
        assert_eq!(report.installations.len(), 1);

        let installation = report.primary_installation().expect("Epic PUBG install");
        assert_eq!(installation.launcher, PubgLauncher::Epic);
        assert_eq!(installation.install_dir, install_dir);
        assert_eq!(installation.executable_path.as_deref(), Some(executable.as_path()));
        assert!(installation.has_battleye());
        assert_eq!(
            installation.metadata.as_ref().and_then(|metadata| metadata.identifier.as_deref()),
            Some("pubg-live")
        );
    }

    #[test]
    fn missing_or_unrelated_metadata_returns_empty_report() {
        let fixture = FixtureDir::new("empty");
        let manifest_dir = fixture.path().join("EpicManifests");

        write_file(
            manifest_dir.join("other-game.item"),
            r#"{"AppName":"other-game","DisplayName":"Other Game","InstallLocation":"C:\\Games\\Other"}"#,
        );

        let report = discover_pubg_from_roots(
            &PubgDiscoveryRoots::new()
                .with_steam_root(fixture.path().join("MissingSteam"))
                .with_epic_manifest_dir(manifest_dir)
                .with_local_app_data_root(fixture.path().join("MissingLocalAppData")),
        );

        assert!(!report.is_installed());
        assert!(report.installations.is_empty());
        assert!(!report.config.is_present());
    }

    struct FixtureDir {
        path: PathBuf,
    }

    impl FixtureDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos();
            let path = env::temp_dir().join(format!(
                "liiiraa-pubg-fixture-{label}-{unique}-{}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("fixture dir should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for FixtureDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(path: impl AsRef<Path>, contents: &str) {
        let path = path.as_ref();
        fs::create_dir_all(path.parent().expect("fixture file should have a parent"))
            .expect("fixture parent should be created");
        fs::write(path, contents).expect("fixture file should be written");
    }

    fn metadata_path(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "\\\\")
    }
}
