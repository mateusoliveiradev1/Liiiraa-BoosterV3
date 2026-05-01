//! PUBG discovery, settings inspection, and safe recommendation planning.

use benchmark::{
    compare_benchmark_runs, BenchmarkComparisonSummary, BenchmarkDecision, BenchmarkRunSummary,
    DEFAULT_BENCHMARK_VARIANCE_PERCENT,
};
use local_store::{LocalStore, OptimizerSnapshot};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    env, error, fmt, fs, io,
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

/// Maximum bytes read from one PUBG config file during safe snapshots.
pub const PUBG_CONFIG_MAX_BYTES: u64 = 512 * 1024;

/// Synthetic section name used for key/value lines before the first INI section.
pub const PUBG_CONFIG_GLOBAL_SECTION: &str = "global";

/// Snapshot type used when storing PUBG config captures locally.
pub const PUBG_CONFIG_SNAPSHOT_TYPE: &str = local_store::PUBG_CONFIG_SNAPSHOT_TYPE;

/// Payload schema version used for stored PUBG config captures.
pub const PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION: &str =
    local_store::PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION;

/// Tweak ID for PUBG launch option cleanup recommendations.
pub const PUBG_LAUNCH_OPTIONS_TWEAK_ID: &str = "pubg.launch-options.legacy";

/// Tweak ID for the PUBG DX11 versus DX11 Enhanced benchmark flow.
pub const PUBG_DX_MODE_TWEAK_ID: &str = "pubg.dx11-vs-dx11e";

/// Variance band used until repeated-run confidence modeling is added.
pub const PUBG_DX_BENCHMARK_VARIANCE_PERCENT: f64 = DEFAULT_BENCHMARK_VARIANCE_PERCENT;

/// Launcher family that provided installation metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// Read-only PUBG launch option discovery and cleanup planning results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionsDiscovery {
    /// Cleanup plans built from accessible launcher metadata.
    pub plans: Vec<PubgLaunchOptionsCleanupPlan>,
    /// Non-fatal warnings collected while inspecting launcher metadata.
    pub warnings: Vec<PubgLaunchOptionsWarning>,
}

impl PubgLaunchOptionsDiscovery {
    /// Creates an empty launch option discovery record.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            plans: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns true when one or more launch option sources need cleanup.
    #[must_use]
    pub fn requires_cleanup(&self) -> bool {
        self.plans
            .iter()
            .any(PubgLaunchOptionsCleanupPlan::requires_cleanup)
    }
}

/// A non-fatal warning from launch option discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionsWarning {
    /// Launcher metadata path that raised the warning.
    pub path: PathBuf,
    /// Human-readable warning detail.
    pub message: String,
}

/// Source metadata for one launch option value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionsSource {
    /// Launcher family that owns the launch option value.
    pub launcher: PubgLauncher,
    /// Launcher metadata path where the value was read.
    pub path: Option<PathBuf>,
    /// Steam userdata account directory when the value came from `localconfig.vdf`.
    pub account_id: Option<String>,
}

impl PubgLaunchOptionsSource {
    /// Creates a Steam `localconfig.vdf` source descriptor.
    #[must_use]
    pub fn steam_local_config(path: impl Into<PathBuf>, account_id: Option<String>) -> Self {
        Self {
            launcher: PubgLauncher::Steam,
            path: Some(path.into()),
            account_id,
        }
    }
}

/// Backup payload captured before recommending any launch option cleanup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionsBackup {
    /// Current launcher launch options exactly as discovered.
    pub current_options: String,
    /// Source metadata path users can return to if cleanup is rejected.
    pub source_path: Option<PathBuf>,
    /// User-facing backup note for the cleanup plan.
    pub note: String,
}

/// Cleanup plan for old or viral PUBG launch options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionsCleanupPlan {
    /// Tweak identifier from the V1 matrix.
    pub tweak_id: String,
    /// Source metadata for the launch options.
    pub source: PubgLaunchOptionsSource,
    /// Current launch options exactly as discovered.
    pub current_options: String,
    /// Parsed launch option tokens.
    pub tokens: Vec<String>,
    /// Legacy or viral flags detected in the parsed tokens.
    pub findings: Vec<PubgLaunchOptionFinding>,
    /// Current options with flagged legacy tokens removed and no replacement flags added.
    pub recommended_options: String,
    /// Backup details captured before cleanup.
    pub backup: PubgLaunchOptionsBackup,
    /// High-level action recommended by the planner.
    pub action: PubgLaunchOptionsAction,
    /// User-facing recommendation summary.
    pub guidance: String,
}

impl PubgLaunchOptionsCleanupPlan {
    /// Returns true when cleanup should be offered to the user.
    #[must_use]
    pub const fn requires_cleanup(&self) -> bool {
        matches!(self.action, PubgLaunchOptionsAction::RecommendCleanup)
    }
}

/// Action selected by the launch option cleanup planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PubgLaunchOptionsAction {
    /// No risky legacy launch options were detected.
    Noop,
    /// The user should remove the detected launch options.
    RecommendCleanup,
}

/// One detected launch option that should be removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgLaunchOptionFinding {
    /// Launch option token that triggered the finding.
    pub token: String,
    /// Stable finding class.
    pub kind: PubgLaunchOptionFindingKind,
    /// Why the option is legacy or unsafe as a default.
    pub reason: String,
    /// Recommended user action.
    pub recommendation: String,
}

/// Stable finding class for legacy PUBG launch options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PubgLaunchOptionFindingKind {
    /// Old render-path forcing such as `-sm4`, `-d3d10`, or `-dx11`.
    ForcedRenderer,
    /// CPU scheduler folklore such as `-USEALLAVAILABLECORES`.
    CpuSchedulerMyth,
    /// Process priority forcing such as `-high`.
    ProcessPriority,
    /// Memory allocator forcing such as `-malloc=system`.
    MemoryAllocator,
    /// Memory ceiling flags such as `-maxMem`.
    MemoryLimit,
    /// Thread count forcing such as `-threads`.
    ThreadCount,
    /// Deprecated Unreal or launcher-era flags.
    DeprecatedEngineFlag,
    /// Source-engine flags copied from non-PUBG tweak packs.
    SourceEngineFlag,
}

/// PUBG DirectX render modes that can appear in the benchmark flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PubgDirectXMode {
    /// DirectX 11 compatibility renderer.
    Dx11,
    /// DirectX 11 Enhanced renderer exposed by PUBG.
    Dx11Enhanced,
    /// DirectX 12 is lab-only when exposed by the game.
    Dx12,
}

impl PubgDirectXMode {
    /// Returns a stable machine-readable render mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dx11 => "dx11",
            Self::Dx11Enhanced => "dx11_enhanced",
            Self::Dx12 => "dx12",
        }
    }

    /// Returns a user-facing render mode label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Dx11 => "DX11",
            Self::Dx11Enhanced => "DX11 Enhanced",
            Self::Dx12 => "DX12",
        }
    }
}

/// One step in the PUBG DirectX benchmark flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkStep {
    /// Stable step identifier.
    pub id: String,
    /// User-facing step label.
    pub label: String,
    /// Why the step exists in the flow.
    pub detail: String,
}

/// One benchmarkable DirectX mode choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkChoice {
    /// Render mode represented by this choice.
    pub mode: PubgDirectXMode,
    /// Current workflow state for the choice.
    pub state: String,
    /// Why this choice is present.
    pub rationale: String,
    /// Rollback behavior after testing this choice.
    pub rollback: String,
    /// Whether this mode is apply-enabled before benchmark evidence exists.
    pub apply_by_default: bool,
}

/// PUBG DirectX benchmark plan shown before any mode recommendation is made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkPlan {
    /// Tweak identifier from the V1 matrix.
    pub tweak_id: String,
    /// Current render mode if the config snapshot exposes it.
    pub current_mode: Option<PubgDirectXMode>,
    /// Explicitly empty universal default to prevent forced renderer behavior.
    pub universal_forced_default: Option<PubgDirectXMode>,
    /// Modes included in the benchmark comparison.
    pub choices: Vec<PubgDirectXBenchmarkChoice>,
    /// Ordered flow steps required before a recommendation.
    pub steps: Vec<PubgDirectXBenchmarkStep>,
    /// Metadata that must match across before/after runs.
    pub metadata_requirements: Vec<String>,
    /// Guardrails that keep the flow anti-cheat safe.
    pub guardrails: Vec<String>,
}

/// One measured PUBG DirectX benchmark run.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkRun {
    /// Render mode used for this run.
    pub mode: PubgDirectXMode,
    /// Average native frames per second.
    pub average_fps: f64,
    /// One percent low native frames per second.
    pub one_percent_low_fps: f64,
    /// Point-one percent low native frames per second.
    pub point_one_percent_low_fps: f64,
    /// P95 frametime in milliseconds.
    pub p95_frame_ms: f64,
    /// Dropped or delayed frame count where capture tooling exposes it.
    pub dropped_frames: u32,
    /// Stability notes such as crashes, hitch clusters, or failed replay consistency.
    pub stability_notes: Vec<String>,
}

impl PubgDirectXBenchmarkRun {
    /// Creates a benchmark run for a PUBG DirectX mode.
    #[must_use]
    pub fn new(
        mode: PubgDirectXMode,
        average_fps: f64,
        one_percent_low_fps: f64,
        point_one_percent_low_fps: f64,
        p95_frame_ms: f64,
        dropped_frames: u32,
    ) -> Self {
        Self {
            mode,
            average_fps,
            one_percent_low_fps,
            point_one_percent_low_fps,
            p95_frame_ms,
            dropped_frames,
            stability_notes: Vec::new(),
        }
    }

    /// Adds one stability note to the run.
    #[must_use]
    pub fn with_stability_note(mut self, note: impl Into<String>) -> Self {
        self.stability_notes.push(note.into());
        self
    }

    fn benchmark_summary(&self) -> BenchmarkRunSummary {
        self.stability_notes.iter().fold(
            BenchmarkRunSummary::new(
                self.mode.display_name(),
                self.average_fps,
                self.one_percent_low_fps,
                self.point_one_percent_low_fps,
                self.p95_frame_ms,
                self.dropped_frames,
            ),
            |summary, note| summary.with_stability_warning(note.clone()),
        )
    }
}

/// Action selected after comparing PUBG DirectX benchmark runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PubgDirectXRecommendationAction {
    /// DX11 Enhanced should be selected based on benchmark evidence.
    RecommendDx11Enhanced,
    /// DX11 should be selected based on benchmark or stability evidence.
    RecommendDx11,
    /// Current mode should be kept because results are too close to call.
    KeepCurrent,
}

/// PUBG DirectX benchmark comparison deltas for UI and telemetry.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkComparison {
    /// Average FPS percent delta for DX11 Enhanced versus DX11.
    pub average_fps_delta_percent: f64,
    /// One percent low FPS percent delta for DX11 Enhanced versus DX11.
    pub one_percent_low_delta_percent: f64,
    /// Point-one percent low FPS percent delta for DX11 Enhanced versus DX11.
    pub point_one_percent_low_delta_percent: f64,
    /// P95 frametime percent delta for DX11 Enhanced versus DX11.
    pub p95_frame_time_delta_percent: f64,
    /// Dropped frame delta for DX11 Enhanced versus DX11.
    pub dropped_frame_delta: i64,
    /// Variance band used for the recommendation.
    pub variance_percent: f64,
}

impl From<BenchmarkComparisonSummary> for PubgDirectXBenchmarkComparison {
    fn from(summary: BenchmarkComparisonSummary) -> Self {
        Self {
            average_fps_delta_percent: summary.average_fps_delta_percent,
            one_percent_low_delta_percent: summary.one_percent_low_delta_percent,
            point_one_percent_low_delta_percent: summary.point_one_percent_low_delta_percent,
            p95_frame_time_delta_percent: summary.p95_frame_time_delta_percent,
            dropped_frame_delta: summary.dropped_frame_delta,
            variance_percent: summary.variance_percent,
        }
    }
}

/// Recommendation produced by the PUBG DirectX benchmark flow.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgDirectXBenchmarkRecommendation {
    /// Tweak identifier from the V1 matrix.
    pub tweak_id: String,
    /// Action selected after comparing runs.
    pub action: PubgDirectXRecommendationAction,
    /// Render mode recommended after benchmark evidence.
    pub recommended_mode: PubgDirectXMode,
    /// Human-readable benchmark rationale.
    pub rationale: String,
    /// Rollback behavior for the selected mode.
    pub rollback: String,
    /// Comparison deltas attached to the recommendation.
    pub comparison: PubgDirectXBenchmarkComparison,
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
    /// Steam launch options found in accessible launcher metadata.
    pub launch_options: PubgLaunchOptionsDiscovery,
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
        launch_options: discover_pubg_launch_options_from_steam_roots(&roots.steam_roots),
    }
}

/// Builds the PUBG DX11 versus DX11 Enhanced benchmark plan without forcing a default.
#[must_use]
pub fn plan_pubg_directx_benchmark(
    current_mode: Option<PubgDirectXMode>,
) -> PubgDirectXBenchmarkPlan {
    let choices = [PubgDirectXMode::Dx11, PubgDirectXMode::Dx11Enhanced]
        .into_iter()
        .map(|mode| PubgDirectXBenchmarkChoice {
            mode,
            state: directx_choice_state(mode, current_mode),
            rationale: match mode {
                PubgDirectXMode::Dx11 => {
                    "Compatibility baseline for the same map, route, and capture duration."
                        .to_owned()
                }
                PubgDirectXMode::Dx11Enhanced => {
                    "Candidate mode for modern systems, accepted only with measured stability."
                        .to_owned()
                }
                PubgDirectXMode::Dx12 => "Lab-only mode outside the T073 comparison.".to_owned(),
            },
            rollback: "Restore the render mode captured before the benchmark.".to_owned(),
            apply_by_default: false,
        })
        .collect();

    PubgDirectXBenchmarkPlan {
        tweak_id: PUBG_DX_MODE_TWEAK_ID.to_owned(),
        current_mode,
        universal_forced_default: None,
        choices,
        steps: vec![
            directx_step(
                "snapshot-config",
                "Snapshot config",
                "Capture the current render mode before suggesting or testing alternatives.",
            ),
            directx_step(
                "capture-dx11",
                "Capture DX11",
                "Run the same route and duration with DX11 as the compatibility baseline.",
            ),
            directx_step(
                "capture-dx11-enhanced",
                "Capture DX11 Enhanced",
                "Repeat the run with DX11 Enhanced after user-controlled mode selection.",
            ),
            directx_step(
                "compare-stability",
                "Compare stability",
                "Compare native FPS, 1% lows, 0.1% lows, p95 frametime, and dropped frames.",
            ),
            directx_step(
                "recommend-or-keep",
                "Recommend or keep current",
                "Recommend a mode only when the result clears the variance band and has no stability blocker.",
            ),
        ],
        metadata_requirements: vec![
            "same map or replay route".to_owned(),
            "same capture duration".to_owned(),
            "same driver version".to_owned(),
            "same Windows build and power plan".to_owned(),
            "native frames only, generated frames labeled separately".to_owned(),
        ],
        guardrails: vec![
            "no PUBG binaries, content folders, BattlEye files, or game memory are modified"
                .to_owned(),
            "no Steam launch renderer flags are added as replacements".to_owned(),
            "previous render mode remains rollbackable from the config snapshot".to_owned(),
        ],
    }
}

/// Recommends DX11 or DX11 Enhanced only after comparing benchmark evidence.
#[must_use]
pub fn recommend_pubg_directx_mode(
    current_mode: PubgDirectXMode,
    dx11_run: &PubgDirectXBenchmarkRun,
    dx11_enhanced_run: &PubgDirectXBenchmarkRun,
) -> PubgDirectXBenchmarkRecommendation {
    let comparison = compare_benchmark_runs(
        &dx11_run.benchmark_summary(),
        &dx11_enhanced_run.benchmark_summary(),
        PUBG_DX_BENCHMARK_VARIANCE_PERCENT,
    );

    let (action, recommended_mode, rationale) = match comparison.decision {
        BenchmarkDecision::PreferCandidate => (
            PubgDirectXRecommendationAction::RecommendDx11Enhanced,
            PubgDirectXMode::Dx11Enhanced,
            format!(
                "DX11 Enhanced improved 1% lows by {:.1}% with p95 frametime delta {:.1}% and no stability blocker.",
                comparison.one_percent_low_delta_percent,
                comparison.p95_frame_time_delta_percent
            ),
        ),
        BenchmarkDecision::KeepBaseline => (
            PubgDirectXRecommendationAction::RecommendDx11,
            PubgDirectXMode::Dx11,
            format!(
                "DX11 remains the safer recommendation because DX11 Enhanced regressed or raised stability warnings; 1% low delta {:.1}%, p95 delta {:.1}%, dropped frame delta {}.",
                comparison.one_percent_low_delta_percent,
                comparison.p95_frame_time_delta_percent,
                comparison.dropped_frame_delta
            ),
        ),
        BenchmarkDecision::Inconclusive => (
            PubgDirectXRecommendationAction::KeepCurrent,
            current_mode,
            format!(
                "Results stayed inside the {:.1}% variance band, so the app keeps the current mode instead of forcing a universal default.",
                comparison.variance_percent
            ),
        ),
    };

    PubgDirectXBenchmarkRecommendation {
        tweak_id: PUBG_DX_MODE_TWEAK_ID.to_owned(),
        action,
        recommended_mode,
        rationale,
        rollback: format!(
            "Restore previous render mode ({}) from the pre-benchmark config snapshot.",
            current_mode.display_name()
        ),
        comparison: comparison.into(),
    }
}

fn directx_step(id: &str, label: &str, detail: &str) -> PubgDirectXBenchmarkStep {
    PubgDirectXBenchmarkStep {
        id: id.to_owned(),
        label: label.to_owned(),
        detail: detail.to_owned(),
    }
}

fn directx_choice_state(
    mode: PubgDirectXMode,
    current_mode: Option<PubgDirectXMode>,
) -> String {
    match current_mode {
        Some(current_mode) if current_mode == mode => "Current baseline, benchmark required".to_owned(),
        Some(_) => "Candidate, benchmark required".to_owned(),
        None => "Unknown current mode, benchmark required".to_owned(),
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

/// Read-only snapshot of PUBG config files captured before generating suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgConfigSnapshot {
    /// Parsed config files included in this snapshot.
    pub files: Vec<PubgConfigFileSnapshot>,
    /// Non-fatal read or parse warnings encountered while building the snapshot.
    pub warnings: Vec<PubgConfigWarning>,
}

impl PubgConfigSnapshot {
    /// Returns true when no readable config files were captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Looks up one parsed config value by file, section, and key.
    #[must_use]
    pub fn setting(
        &self,
        file_name: &str,
        section_name: &str,
        key: &str,
    ) -> Option<&PubgConfigEntry> {
        self.files
            .iter()
            .find(|file| file.file_name.eq_ignore_ascii_case(file_name))
            .and_then(|file| file.setting(section_name, key))
    }
}

/// One PUBG config file captured in a read-only snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgConfigFileSnapshot {
    /// Source file path.
    pub path: PathBuf,
    /// File name, such as `GameUserSettings.ini`.
    pub file_name: String,
    /// Number of bytes read from disk before UTF-8 normalization.
    pub byte_len: usize,
    /// Raw text captured before suggestions are generated.
    pub raw_contents: String,
    /// Parsed INI sections in source order.
    pub sections: Vec<PubgConfigSection>,
    /// Non-fatal warnings for this file.
    pub warnings: Vec<PubgConfigWarning>,
}

impl PubgConfigFileSnapshot {
    /// Looks up one parsed value by section and key.
    #[must_use]
    pub fn setting(&self, section_name: &str, key: &str) -> Option<&PubgConfigEntry> {
        self.sections
            .iter()
            .find(|section| section.name.eq_ignore_ascii_case(section_name))
            .and_then(|section| {
                section
                    .entries
                    .iter()
                    .find(|entry| entry.key.eq_ignore_ascii_case(key))
            })
    }
}

/// Parsed PUBG INI section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgConfigSection {
    /// Section name without brackets.
    pub name: String,
    /// Key/value entries parsed inside this section.
    pub entries: Vec<PubgConfigEntry>,
}

/// Parsed PUBG INI key/value entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgConfigEntry {
    /// Source line number, starting at one.
    pub line: usize,
    /// Setting key.
    pub key: String,
    /// Setting value, trimmed but otherwise preserved.
    pub value: String,
}

/// Non-fatal issue encountered while reading or parsing config files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PubgConfigWarning {
    /// File path related to the warning.
    pub path: Option<PathBuf>,
    /// Source line number when the warning came from parser input.
    pub line: Option<usize>,
    /// Stable warning classification.
    pub kind: PubgConfigWarningKind,
    /// Human-readable diagnostic detail.
    pub message: String,
}

/// Stable warning kind for read and parser diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PubgConfigWarningKind {
    /// Path was not one of the supported PUBG config file names.
    UnsupportedFile,
    /// Path is a symlink and was skipped to avoid following unexpected targets.
    Symlink,
    /// Path was not a regular file.
    NotFile,
    /// File exceeded the safe read limit.
    TooLarge,
    /// File could not be read.
    Io,
    /// File was not valid UTF-8 and was decoded lossily for inspection.
    InvalidUtf8,
    /// Section header was malformed.
    MalformedSection,
    /// Key/value line was malformed.
    MalformedEntry,
}

/// Reason a PUBG config snapshot persistence operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubgConfigSnapshotErrorReason {
    /// JSON serialization failed.
    Serialization,
    /// Local SQLite storage rejected the snapshot.
    LocalStore,
}

impl PubgConfigSnapshotErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Serialization => "serialization",
            Self::LocalStore => "local_store",
        }
    }
}

/// Structured error for converting or persisting PUBG config snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubgConfigSnapshotError {
    reason: PubgConfigSnapshotErrorReason,
    detail: String,
}

impl PubgConfigSnapshotError {
    fn serialization(detail: impl Into<String>) -> Self {
        Self {
            reason: PubgConfigSnapshotErrorReason::Serialization,
            detail: detail.into(),
        }
    }

    fn local_store(detail: impl Into<String>) -> Self {
        Self {
            reason: PubgConfigSnapshotErrorReason::LocalStore,
            detail: detail.into(),
        }
    }

    /// Returns the failure reason.
    #[must_use]
    pub const fn reason(&self) -> PubgConfigSnapshotErrorReason {
        self.reason
    }

    /// Returns diagnostic detail for logs or tests.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for PubgConfigSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.detail)
    }
}

impl error::Error for PubgConfigSnapshotError {}

impl From<serde_json::Error> for PubgConfigSnapshotError {
    fn from(error: serde_json::Error) -> Self {
        Self::serialization(error.to_string())
    }
}

impl From<local_store::LocalStoreError> for PubgConfigSnapshotError {
    fn from(error: local_store::LocalStoreError) -> Self {
        Self::local_store(error.to_string())
    }
}

/// Reads and parses discovered PUBG config files into a read-only snapshot.
///
/// Unreadable, oversized, unsupported, or malformed files do not panic or mutate disk; they are
/// represented as warnings so suggestion code can remain conservative.
#[must_use]
pub fn read_pubg_config_snapshot(discovery: &PubgConfigDiscovery) -> PubgConfigSnapshot {
    read_pubg_config_snapshot_from_paths(&discovery.files)
}

/// Reads and parses caller-provided PUBG config paths into a read-only snapshot.
#[must_use]
pub fn read_pubg_config_snapshot_from_paths(paths: &[PathBuf]) -> PubgConfigSnapshot {
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for path in paths {
        let (file, mut file_warnings) = read_pubg_config_file_snapshot(path);
        warnings.append(&mut file_warnings);

        if let Some(file) = file {
            files.push(file);
        }
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    warnings.sort_by_key(warning_sort_key);
    warnings.dedup();

    PubgConfigSnapshot { files, warnings }
}

/// Parses already-read PUBG config text into the same structure used by file snapshots.
#[must_use]
pub fn parse_pubg_config_contents(
    path: impl Into<PathBuf>,
    contents: &str,
) -> PubgConfigFileSnapshot {
    let path = path.into();
    parse_pubg_config_file(&path, contents, contents.len(), Vec::new())
}

/// Builds a local-store optimizer snapshot from a PUBG config snapshot.
pub fn pubg_config_optimizer_snapshot(
    snapshot: &PubgConfigSnapshot,
    id: impl Into<String>,
    created_at_utc: impl Into<String>,
) -> Result<OptimizerSnapshot, PubgConfigSnapshotError> {
    Ok(OptimizerSnapshot {
        id: id.into(),
        snapshot_type: PUBG_CONFIG_SNAPSHOT_TYPE.to_owned(),
        created_at_utc: created_at_utc.into(),
        schema_version: PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION.to_owned(),
        payload_json: serde_json::to_string(snapshot)?,
    })
}

/// Persists a PUBG config snapshot before recommendation code uses the parsed settings.
pub fn persist_pubg_config_snapshot(
    store: &LocalStore,
    snapshot: &PubgConfigSnapshot,
    id: impl Into<String>,
    created_at_utc: impl Into<String>,
) -> Result<OptimizerSnapshot, PubgConfigSnapshotError> {
    let record = pubg_config_optimizer_snapshot(snapshot, id, created_at_utc)?;
    store.insert_snapshot(&record)?;
    Ok(record)
}

/// Discovers PUBG launch options from accessible Steam `localconfig.vdf` files.
#[must_use]
pub fn discover_pubg_launch_options_from_steam_roots(
    steam_roots: &[PathBuf],
) -> PubgLaunchOptionsDiscovery {
    let mut plans = Vec::new();
    let mut warnings = Vec::new();

    for (path, account_id) in steam_local_config_candidates(steam_roots) {
        let source = PubgLaunchOptionsSource::steam_local_config(path.clone(), account_id);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                warnings.push(PubgLaunchOptionsWarning {
                    path,
                    message: format!("Steam launch options could not be read: {error}"),
                });
                continue;
            }
        };

        for current_options in pubg_launch_options_from_steam_local_config(&contents) {
            plans.push(plan_pubg_launch_option_cleanup(
                source.clone(),
                current_options,
            ));
        }
    }

    plans.sort_by_key(|plan| launch_options_plan_sort_key(plan));
    plans.dedup_by_key(|plan| launch_options_plan_sort_key(plan));

    PubgLaunchOptionsDiscovery { plans, warnings }
}

/// Builds a cleanup plan for one PUBG launch option value.
#[must_use]
pub fn plan_pubg_launch_option_cleanup(
    source: PubgLaunchOptionsSource,
    current_options: impl Into<String>,
) -> PubgLaunchOptionsCleanupPlan {
    let current_options = current_options.into();
    let tokens = tokenize_pubg_launch_options(&current_options);
    let (findings, removal_indices) = launch_option_findings(&tokens);
    let recommended_tokens = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if removal_indices.contains(&index) {
                None
            } else {
                Some(token.clone())
            }
        })
        .collect::<Vec<_>>();
    let recommended_options = serialize_launch_option_tokens(&recommended_tokens);
    let action = if findings.is_empty() {
        PubgLaunchOptionsAction::Noop
    } else {
        PubgLaunchOptionsAction::RecommendCleanup
    };
    let guidance = if findings.is_empty() {
        "No legacy PUBG launch options were detected; keep launcher options unchanged.".to_owned()
    } else {
        "Remove the detected legacy launch options and do not add replacement force flags; use the benchmark flow for renderer choices.".to_owned()
    };
    let backup = PubgLaunchOptionsBackup {
        current_options: current_options.clone(),
        source_path: source.path.clone(),
        note: "Current launch options captured before recommending cleanup.".to_owned(),
    };

    PubgLaunchOptionsCleanupPlan {
        tweak_id: PUBG_LAUNCH_OPTIONS_TWEAK_ID.to_owned(),
        source,
        current_options,
        tokens,
        findings,
        recommended_options,
        backup,
        action,
        guidance,
    }
}

/// Splits a launch option string into quote-aware tokens.
#[must_use]
pub fn tokenize_pubg_launch_options(raw_options: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for character in raw_options.chars() {
        match character {
            '"' => in_quotes = !in_quotes,
            character if character.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
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

fn read_pubg_config_file_snapshot(
    path: &Path,
) -> (Option<PubgConfigFileSnapshot>, Vec<PubgConfigWarning>) {
    let mut warnings = Vec::new();

    if !is_supported_pubg_config_file(path) {
        warnings.push(config_warning(
            path,
            None,
            PubgConfigWarningKind::UnsupportedFile,
            "unsupported PUBG config file name",
        ));
        return (None, warnings);
    }

    let Ok(metadata) = fs::symlink_metadata(path) else {
        warnings.push(config_warning(
            path,
            None,
            PubgConfigWarningKind::Io,
            "config file metadata could not be read",
        ));
        return (None, warnings);
    };

    if metadata.file_type().is_symlink() {
        warnings.push(config_warning(
            path,
            None,
            PubgConfigWarningKind::Symlink,
            "symlink config path was skipped",
        ));
        return (None, warnings);
    }

    if !metadata.is_file() {
        warnings.push(config_warning(
            path,
            None,
            PubgConfigWarningKind::NotFile,
            "config path is not a regular file",
        ));
        return (None, warnings);
    }

    if metadata.len() > PUBG_CONFIG_MAX_BYTES {
        warnings.push(config_warning(
            path,
            None,
            PubgConfigWarningKind::TooLarge,
            format!("config file is larger than {PUBG_CONFIG_MAX_BYTES} bytes"),
        ));
        return (None, warnings);
    }

    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(config_warning(
                path,
                None,
                PubgConfigWarningKind::Io,
                format_read_error(error),
            ));
            return (None, warnings);
        }
    };
    let byte_len = bytes.len();

    let contents = match String::from_utf8(bytes) {
        Ok(contents) => contents,
        Err(error) => {
            warnings.push(config_warning(
                path,
                None,
                PubgConfigWarningKind::InvalidUtf8,
                "config file was decoded lossily because it is not valid UTF-8",
            ));
            String::from_utf8_lossy(error.as_bytes()).into_owned()
        }
    };

    let mut file = parse_pubg_config_file(path, &contents, byte_len, warnings);
    file.warnings.sort_by_key(warning_sort_key);
    file.warnings.dedup();
    let file_warnings = file.warnings.clone();

    (Some(file), file_warnings)
}

fn parse_pubg_config_file(
    path: &Path,
    contents: &str,
    byte_len: usize,
    mut warnings: Vec<PubgConfigWarning>,
) -> PubgConfigFileSnapshot {
    let mut sections = Vec::<PubgConfigSection>::new();
    let mut current_section = PubgConfigSection {
        name: PUBG_CONFIG_GLOBAL_SECTION.to_owned(),
        entries: Vec::new(),
    };

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if let Some(section_name) = parse_section_header(line) {
                push_section_if_needed(&mut sections, &mut current_section);
                current_section = PubgConfigSection {
                    name: section_name.to_owned(),
                    entries: Vec::new(),
                };
            } else {
                warnings.push(config_warning(
                    path,
                    Some(line_number),
                    PubgConfigWarningKind::MalformedSection,
                    "section header must end with ']' and include a name",
                ));
            }

            continue;
        }

        if let Some((key, value)) = parse_key_value(line) {
            current_section.entries.push(PubgConfigEntry {
                line: line_number,
                key: key.to_owned(),
                value: value.to_owned(),
            });
        } else {
            warnings.push(config_warning(
                path,
                Some(line_number),
                PubgConfigWarningKind::MalformedEntry,
                "config line was not a key=value entry",
            ));
        }
    }

    push_section_if_needed(&mut sections, &mut current_section);

    PubgConfigFileSnapshot {
        path: path.to_path_buf(),
        file_name: config_file_name(path),
        byte_len,
        raw_contents: contents.to_owned(),
        sections,
        warnings,
    }
}

fn push_section_if_needed(
    sections: &mut Vec<PubgConfigSection>,
    section: &mut PubgConfigSection,
) {
    if !section.entries.is_empty() {
        sections.push(section.clone());
        section.entries.clear();
    }
}

fn parse_section_header(line: &str) -> Option<&str> {
    let section_name = line.strip_prefix('[')?.strip_suffix(']')?.trim();

    if section_name.is_empty() {
        None
    } else {
        Some(section_name)
    }
}

fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();

    if key.is_empty() {
        None
    } else {
        Some((key, value.trim()))
    }
}

fn is_supported_pubg_config_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .map_or(false, |file_name| {
            PUBG_CONFIG_FILE_NAMES
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(file_name))
        })
}

fn config_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn config_warning(
    path: &Path,
    line: Option<usize>,
    kind: PubgConfigWarningKind,
    message: impl Into<String>,
) -> PubgConfigWarning {
    PubgConfigWarning {
        path: Some(path.to_path_buf()),
        line,
        kind,
        message: message.into(),
    }
}

fn warning_sort_key(warning: &PubgConfigWarning) -> (String, Option<usize>, &'static str, String) {
    (
        warning
            .path
            .as_ref()
            .map_or_else(String::new, |path| path.to_string_lossy().into_owned()),
        warning.line,
        warning.kind.as_str(),
        warning.message.clone(),
    )
}

impl PubgConfigWarningKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedFile => "unsupported_file",
            Self::Symlink => "symlink",
            Self::NotFile => "not_file",
            Self::TooLarge => "too_large",
            Self::Io => "io",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::MalformedSection => "malformed_section",
            Self::MalformedEntry => "malformed_entry",
        }
    }
}

fn format_read_error(error: io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "config file was not found".to_owned(),
        io::ErrorKind::PermissionDenied => "permission denied while reading config file".to_owned(),
        _ => format!("config file could not be read: {error}"),
    }
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

fn steam_local_config_candidates(steam_roots: &[PathBuf]) -> Vec<(PathBuf, Option<String>)> {
    let mut candidates = BTreeSet::new();

    for steam_root in steam_roots {
        let userdata_dir = steam_root.join("userdata");
        let Ok(accounts) = fs::read_dir(userdata_dir) else {
            continue;
        };

        for account in accounts.flatten() {
            let account_dir = account.path();
            if !account_dir.is_dir() {
                continue;
            }

            let local_config = account_dir.join("config").join("localconfig.vdf");
            if local_config.is_file() {
                let account_id = account_dir
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map(ToOwned::to_owned);
                candidates.insert((local_config, account_id));
            }
        }
    }

    candidates.into_iter().collect()
}

fn pubg_launch_options_from_steam_local_config(source: &str) -> Vec<String> {
    vdf_blocks_for_key(source, PUBG_STEAM_APP_ID)
        .into_iter()
        .filter_map(|block| quoted_metadata_value(block, "LaunchOptions"))
        .map(|options| options.trim().to_owned())
        .filter(|options| !options.is_empty())
        .collect()
}

fn launch_options_plan_sort_key(plan: &PubgLaunchOptionsCleanupPlan) -> (String, String, String) {
    (
        plan.source
            .path
            .as_ref()
            .map_or_else(String::new, |path| path.to_string_lossy().into_owned()),
        plan.source.account_id.clone().unwrap_or_default(),
        plan.current_options.clone(),
    )
}

fn launch_option_findings(
    tokens: &[String],
) -> (Vec<PubgLaunchOptionFinding>, BTreeSet<usize>) {
    let mut findings = Vec::new();
    let mut removal_indices = BTreeSet::new();

    for (index, token) in tokens.iter().enumerate() {
        let normalized = normalized_launch_option_token(token);
        let Some(kind) = classify_legacy_launch_option(&normalized) else {
            continue;
        };

        findings.push(PubgLaunchOptionFinding {
            token: token.clone(),
            kind,
            reason: kind.reason().to_owned(),
            recommendation: kind.recommendation().to_owned(),
        });
        removal_indices.insert(index);

        if launch_option_expects_value(&normalized)
            && tokens
                .get(index + 1)
                .is_some_and(|next| !is_launch_option_flag(next))
        {
            removal_indices.insert(index + 1);
        }
    }

    (findings, removal_indices)
}

fn normalized_launch_option_token(token: &str) -> String {
    let normalized = token.trim().trim_matches('"').to_ascii_lowercase();
    normalized
        .split_once('=')
        .map_or(normalized.clone(), |(flag, _)| flag.to_owned())
}

fn classify_legacy_launch_option(token: &str) -> Option<PubgLaunchOptionFindingKind> {
    match token {
        "-sm4" | "-d3d10" | "-d3d11" | "-d3d12" | "-dx9" | "-dx10" | "-dx11" | "-dx12"
        | "-force-d3d11" | "-force-d3d12" | "-force-feature-level-10-0" | "-opengl"
        | "-vulkan" => Some(PubgLaunchOptionFindingKind::ForcedRenderer),
        "-useallavailablecores" => Some(PubgLaunchOptionFindingKind::CpuSchedulerMyth),
        "-high" | "-realtime" | "-priority" => Some(PubgLaunchOptionFindingKind::ProcessPriority),
        "-malloc" => Some(PubgLaunchOptionFindingKind::MemoryAllocator),
        "-heapsize" | "-maxmem" => Some(PubgLaunchOptionFindingKind::MemoryLimit),
        "-cpu-count" | "-cpucount" | "-thread" | "-threads" => {
            Some(PubgLaunchOptionFindingKind::ThreadCount)
        }
        "-freq" | "-lowmemory" | "-nomansky" | "-notexturestreaming" | "-novid" | "-nosplash"
        | "-refresh" => Some(PubgLaunchOptionFindingKind::DeprecatedEngineFlag),
        "+cl_forcepreload" | "+mat_queue_mode" => Some(PubgLaunchOptionFindingKind::SourceEngineFlag),
        _ => None,
    }
}

fn launch_option_expects_value(token: &str) -> bool {
    matches!(
        token,
        "-cpu-count"
            | "-cpucount"
            | "-freq"
            | "-heapsize"
            | "-malloc"
            | "-maxmem"
            | "-priority"
            | "-refresh"
            | "-thread"
            | "-threads"
            | "+cl_forcepreload"
            | "+mat_queue_mode"
    )
}

fn is_launch_option_flag(token: &str) -> bool {
    token.starts_with('-') || token.starts_with('+')
}

fn serialize_launch_option_tokens(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| {
            if token.chars().any(char::is_whitespace) {
                format!("\"{}\"", token.replace('"', "\\\""))
            } else {
                token.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl PubgLaunchOptionFindingKind {
    fn reason(self) -> &'static str {
        match self {
            Self::ForcedRenderer => {
                "Forced render path flags are legacy, machine-specific, and should be benchmarked instead."
            }
            Self::CpuSchedulerMyth => {
                "Windows already schedules PUBG across available cores; this flag is old tuning folklore."
            }
            Self::ProcessPriority => {
                "Forcing process priority can starve system work and is blocked as a default optimization."
            }
            Self::MemoryAllocator => {
                "Allocator launch flags are unsupported for current PUBG and can create stability risk."
            }
            Self::MemoryLimit => {
                "Memory ceiling flags are legacy tweaks that can reduce stability on modern systems."
            }
            Self::ThreadCount => {
                "Manual thread counts can fight the Windows scheduler and modern CPU topology."
            }
            Self::DeprecatedEngineFlag => {
                "This old launch flag is not part of the approved PUBG V1 optimization path."
            }
            Self::SourceEngineFlag => {
                "This flag belongs to Source-engine tweak packs and is not a PUBG optimization."
            }
        }
    }

    fn recommendation(self) -> &'static str {
        match self {
            Self::ForcedRenderer => {
                "Remove this flag and compare DirectX modes through the benchmark flow."
            }
            Self::CpuSchedulerMyth
            | Self::MemoryAllocator
            | Self::MemoryLimit
            | Self::ThreadCount
            | Self::DeprecatedEngineFlag
            | Self::SourceEngineFlag => "Remove this flag without adding a replacement.",
            Self::ProcessPriority => {
                "Remove this flag; priority forcing remains blocked outside future benchmarked lab work."
            }
        }
    }
}

fn vdf_blocks_for_key<'a>(source: &'a str, key: &str) -> Vec<&'a str> {
    let needle = format!("\"{key}\"");
    let mut blocks = Vec::new();
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find(&needle) {
        let key_start = search_start + relative_index;
        let after_key = key_start + needle.len();
        let Some(relative_open_brace) = source[after_key..].find('{') else {
            break;
        };
        let open_brace = after_key + relative_open_brace;

        if !source[after_key..open_brace].trim().is_empty() {
            search_start = after_key;
            continue;
        }

        if let Some(close_brace) = matching_vdf_brace(source, open_brace) {
            blocks.push(&source[open_brace + 1..close_brace]);
            search_start = close_brace + 1;
        } else {
            break;
        }
    }

    blocks
}

fn matching_vdf_brace(source: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;

    for (offset, character) in source[open_brace..].char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_quotes = false;
            }
            continue;
        }

        match character {
            '"' => in_quotes = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_brace + offset);
                }
            }
            _ => {}
        }
    }

    None
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
    fn parses_pubg_config_values_and_records_malformed_lines() {
        let file = parse_pubg_config_contents(
            "GameUserSettings.ini",
            r#"; user comment
[/Script/TslGame.TslGameUserSettings]
ResolutionSizeX=1920
sg.ViewDistanceQuality = 2
malformed line
[ScalabilityGroups
sg.AntiAliasingQuality=1
"#,
        );

        assert_eq!(file.file_name, "GameUserSettings.ini");
        assert_eq!(
            file.setting("/Script/TslGame.TslGameUserSettings", "ResolutionSizeX")
                .map(|entry| entry.value.as_str()),
            Some("1920")
        );
        assert_eq!(
            file.setting("/Script/TslGame.TslGameUserSettings", "sg.ViewDistanceQuality")
                .map(|entry| entry.value.as_str()),
            Some("2")
        );
        assert_eq!(
            file.warnings
                .iter()
                .map(|warning| warning.kind)
                .collect::<Vec<_>>(),
            vec![
                PubgConfigWarningKind::MalformedEntry,
                PubgConfigWarningKind::MalformedSection,
            ]
        );
    }

    #[test]
    fn reads_config_snapshot_without_following_unsupported_paths() {
        let fixture = FixtureDir::new("config-snapshot");
        let config_file = fixture.path().join("GameUserSettings.ini");
        let unrelated_file = fixture.path().join("notes.txt");

        write_file(
            &config_file,
            "[/Script/TslGame.TslGameUserSettings]\nFrameRateLimit=237.5\n",
        );
        write_file(&unrelated_file, "not a PUBG config");

        let snapshot =
            read_pubg_config_snapshot_from_paths(&[unrelated_file.clone(), config_file.clone()]);

        assert_eq!(snapshot.files.len(), 1);
        assert_eq!(snapshot.files[0].path, config_file);
        assert_eq!(
            snapshot
                .setting(
                    "GameUserSettings.ini",
                    "/Script/TslGame.TslGameUserSettings",
                    "FrameRateLimit"
                )
                .map(|entry| entry.value.as_str()),
            Some("237.5")
        );
        assert_eq!(snapshot.warnings.len(), 1);
        assert_eq!(
            snapshot.warnings[0].path.as_deref(),
            Some(unrelated_file.as_path())
        );
        assert_eq!(
            snapshot.warnings[0].kind,
            PubgConfigWarningKind::UnsupportedFile
        );
    }

    #[test]
    fn persists_pubg_config_snapshot_for_local_reuse() {
        let store = LocalStore::open_in_memory().expect("store should open");
        let file = parse_pubg_config_contents(
            "GameUserSettings.ini",
            "[/Script/TslGame.TslGameUserSettings]\nResolutionSizeY=1080\n",
        );
        let snapshot = PubgConfigSnapshot {
            files: vec![file],
            warnings: Vec::new(),
        };

        let record = persist_pubg_config_snapshot(
            &store,
            &snapshot,
            "snapshot:pubg-config:001",
            "2026-05-01T12:00:00Z",
        )
        .expect("snapshot should persist");

        assert_eq!(record.snapshot_type, PUBG_CONFIG_SNAPSHOT_TYPE);
        assert_eq!(record.schema_version, PUBG_CONFIG_SNAPSHOT_SCHEMA_VERSION);
        assert!(record.payload_json.contains("ResolutionSizeY"));

        let stored = store
            .pubg_config_snapshots()
            .expect("PUBG snapshots should be listed");
        assert_eq!(stored, vec![record]);
    }

    #[test]
    fn plans_launch_option_cleanup_without_forced_replacements() {
        let source = PubgLaunchOptionsSource::steam_local_config(
            "localconfig.vdf",
            Some("123456".to_owned()),
        );
        let plan = plan_pubg_launch_option_cleanup(
            source,
            "-USEALLAVAILABLECORES -malloc=system -high -dx11 -threads 8 -novid -safe-note",
        );

        assert!(plan.requires_cleanup());
        assert_eq!(plan.tweak_id, PUBG_LAUNCH_OPTIONS_TWEAK_ID);
        assert_eq!(plan.action, PubgLaunchOptionsAction::RecommendCleanup);
        assert_eq!(plan.recommended_options, "-safe-note");
        assert_eq!(plan.backup.current_options, plan.current_options);
        assert!(plan.guidance.contains("do not add replacement force flags"));
        assert_eq!(
            plan.findings
                .iter()
                .map(|finding| finding.kind)
                .collect::<Vec<_>>(),
            vec![
                PubgLaunchOptionFindingKind::CpuSchedulerMyth,
                PubgLaunchOptionFindingKind::MemoryAllocator,
                PubgLaunchOptionFindingKind::ProcessPriority,
                PubgLaunchOptionFindingKind::ForcedRenderer,
                PubgLaunchOptionFindingKind::ThreadCount,
                PubgLaunchOptionFindingKind::DeprecatedEngineFlag,
            ]
        );
        assert!(plan
            .findings
            .iter()
            .all(|finding| finding.recommendation.starts_with("Remove")));
    }

    #[test]
    fn tokenizes_quoted_launch_options_for_cleanup() {
        let tokens = tokenize_pubg_launch_options(r#"-dx11 "-custom value" +mat_queue_mode 2"#);

        assert_eq!(
            tokens,
            vec![
                "-dx11".to_owned(),
                "-custom value".to_owned(),
                "+mat_queue_mode".to_owned(),
                "2".to_owned(),
            ]
        );

        let plan = plan_pubg_launch_option_cleanup(
            PubgLaunchOptionsSource::steam_local_config("localconfig.vdf", None),
            r#"-dx11 "-custom value" +mat_queue_mode 2"#,
        );

        assert_eq!(plan.recommended_options, "\"-custom value\"");
    }

    #[test]
    fn discovers_steam_launch_options_from_localconfig_fixture() {
        let fixture = FixtureDir::new("launch-options");
        let steam_root = fixture.path().join("Steam");
        let local_config = steam_root
            .join("userdata")
            .join("123456")
            .join("config")
            .join("localconfig.vdf");

        write_file(
            &local_config,
            r#""UserLocalConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "apps"
                {
                    "578080"
                    {
                        "LaunchOptions" "-sm4 -USEALLAVAILABLECORES -safe-note"
                    }
                    "730"
                    {
                        "LaunchOptions" "-allow_third_party_software"
                    }
                }
            }
        }
    }
}"#,
        );

        let discovery = discover_pubg_launch_options_from_steam_roots(&[steam_root]);

        assert!(discovery.requires_cleanup());
        assert!(discovery.warnings.is_empty());
        assert_eq!(discovery.plans.len(), 1);
        assert_eq!(
            discovery.plans[0].source.path.as_deref(),
            Some(local_config.as_path())
        );
        assert_eq!(
            discovery.plans[0].source.account_id.as_deref(),
            Some("123456")
        );
        assert_eq!(discovery.plans[0].recommended_options, "-safe-note");
        assert_eq!(
            discovery.plans[0]
                .findings
                .iter()
                .map(|finding| finding.token.as_str())
                .collect::<Vec<_>>(),
            vec!["-sm4", "-USEALLAVAILABLECORES"]
        );
    }

    #[test]
    fn directx_benchmark_plan_has_no_forced_default() {
        let plan = plan_pubg_directx_benchmark(Some(PubgDirectXMode::Dx11));

        assert_eq!(plan.tweak_id, PUBG_DX_MODE_TWEAK_ID);
        assert_eq!(plan.universal_forced_default, None);
        assert_eq!(
            plan.choices
                .iter()
                .map(|choice| choice.mode)
                .collect::<Vec<_>>(),
            vec![PubgDirectXMode::Dx11, PubgDirectXMode::Dx11Enhanced]
        );
        assert!(plan.choices.iter().all(|choice| !choice.apply_by_default));
        assert!(plan
            .guardrails
            .iter()
            .any(|guardrail| guardrail.contains("BattlEye")));
        assert!(plan
            .metadata_requirements
            .iter()
            .any(|requirement| requirement.contains("native frames")));
    }

    #[test]
    fn directx_recommendation_prefers_enhanced_only_after_evidence() {
        let dx11 = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11,
            176.0,
            127.0,
            92.0,
            10.2,
            4,
        );
        let dx11_enhanced = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11Enhanced,
            181.0,
            139.0,
            99.0,
            9.5,
            2,
        );

        let recommendation =
            recommend_pubg_directx_mode(PubgDirectXMode::Dx11, &dx11, &dx11_enhanced);

        assert_eq!(
            recommendation.action,
            PubgDirectXRecommendationAction::RecommendDx11Enhanced
        );
        assert_eq!(recommendation.recommended_mode, PubgDirectXMode::Dx11Enhanced);
        assert!(recommendation
            .rationale
            .contains("improved 1% lows"));
        assert!(recommendation.rollback.contains("DX11"));
    }

    #[test]
    fn directx_recommendation_keeps_current_when_runs_are_inside_variance() {
        let dx11 = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11,
            176.0,
            127.0,
            92.0,
            10.2,
            4,
        );
        let dx11_enhanced = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11Enhanced,
            177.0,
            129.0,
            93.0,
            10.1,
            4,
        );

        let recommendation =
            recommend_pubg_directx_mode(PubgDirectXMode::Dx11, &dx11, &dx11_enhanced);

        assert_eq!(recommendation.action, PubgDirectXRecommendationAction::KeepCurrent);
        assert_eq!(recommendation.recommended_mode, PubgDirectXMode::Dx11);
        assert!(recommendation
            .rationale
            .contains("keeps the current mode"));
    }

    #[test]
    fn directx_recommendation_falls_back_to_dx11_on_enhanced_instability() {
        let dx11 = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11,
            176.0,
            127.0,
            92.0,
            10.2,
            4,
        );
        let dx11_enhanced = PubgDirectXBenchmarkRun::new(
            PubgDirectXMode::Dx11Enhanced,
            181.0,
            139.0,
            99.0,
            9.5,
            2,
        )
        .with_stability_note("hitch cluster detected");

        let recommendation =
            recommend_pubg_directx_mode(PubgDirectXMode::Dx11Enhanced, &dx11, &dx11_enhanced);

        assert_eq!(
            recommendation.action,
            PubgDirectXRecommendationAction::RecommendDx11
        );
        assert_eq!(recommendation.recommended_mode, PubgDirectXMode::Dx11);
        assert!(recommendation.rationale.contains("stability warnings"));
        assert!(recommendation.rollback.contains("DX11 Enhanced"));
    }

    #[test]
    fn ignores_other_steam_app_launch_options() {
        let fixture = FixtureDir::new("other-launch-options");
        let steam_root = fixture.path().join("Steam");

        write_file(
            steam_root
                .join("userdata")
                .join("123456")
                .join("config")
                .join("localconfig.vdf"),
            r#""UserLocalConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "apps"
                {
                    "730"
                    {
                        "LaunchOptions" "-high"
                    }
                }
            }
        }
    }
}"#,
        );

        let discovery = discover_pubg_launch_options_from_steam_roots(&[steam_root]);

        assert!(discovery.plans.is_empty());
        assert!(!discovery.requires_cleanup());
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
