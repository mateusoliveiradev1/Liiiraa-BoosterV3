//! Benchmark capture, parsing, and scoring primitives.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
    time::Duration,
};

/// Default variance band used when a benchmark comparison has no stronger sample model yet.
pub const DEFAULT_BENCHMARK_VARIANCE_PERCENT: f64 = 3.0;

/// Default capture duration for guided before/after benchmark sessions.
pub const DEFAULT_CAPTURE_DURATION: Duration = Duration::from_secs(120);

/// Metadata recorded at the start of a benchmark capture session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSessionMetadata {
    /// Human-readable game or workload name.
    pub game: String,
    /// Optional map, replay, scenario, or other run label.
    pub session_label: Option<String>,
    /// Windows build recorded before capture starts.
    pub windows_build: String,
    /// GPU driver version recorded before capture starts.
    pub driver_version: String,
    /// Active Windows power plan recorded before capture starts.
    pub active_power_plan: String,
    /// Active optimizer profile recorded before capture starts.
    pub active_optimizer_profile: String,
    /// Timestamp supplied by the caller, usually ISO 8601.
    pub timestamp: String,
}

impl BenchmarkSessionMetadata {
    /// Creates benchmark session metadata required by the OpenSpec benchmark contract.
    #[must_use]
    pub fn new(
        game: impl Into<String>,
        windows_build: impl Into<String>,
        driver_version: impl Into<String>,
        active_power_plan: impl Into<String>,
        active_optimizer_profile: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> Self {
        Self {
            game: game.into(),
            session_label: None,
            windows_build: windows_build.into(),
            driver_version: driver_version.into(),
            active_power_plan: active_power_plan.into(),
            active_optimizer_profile: active_optimizer_profile.into(),
            timestamp: timestamp.into(),
        }
    }

    /// Adds a user-facing session label such as a map, replay, or test route.
    #[must_use]
    pub fn with_session_label(mut self, session_label: impl Into<String>) -> Self {
        self.session_label = Some(session_label.into());
        self
    }
}

/// Files emitted by a PresentMon-compatible capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentMonCapturePaths {
    /// Raw frame-event CSV file.
    pub raw_csv: PathBuf,
    /// Optional stats CSV file paired with the raw frame-event CSV.
    pub stats_csv: PathBuf,
}

impl PresentMonCapturePaths {
    /// Creates capture paths from the raw CSV path and derives the conventional stats path.
    pub fn from_raw_csv(raw_csv: impl Into<PathBuf>) -> Result<Self, BenchmarkError> {
        let raw_csv = raw_csv.into();
        ensure_csv_path(&raw_csv)?;

        let file_stem = raw_csv
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or(BenchmarkError::InvalidCapturePath)?;
        let stats_file_name = format!("{file_stem}-stats.csv");
        let stats_csv = raw_csv.with_file_name(stats_file_name);

        Ok(Self { raw_csv, stats_csv })
    }
}

/// PresentMon console capture configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentMonCaptureConfig {
    /// PresentMon console executable path.
    pub executable_path: PathBuf,
    /// Process executable name to capture, for example `TslGame.exe`.
    pub process_name: String,
    /// Raw and stats capture paths.
    pub paths: PresentMonCapturePaths,
    /// Optional fixed capture duration.
    pub duration: Option<Duration>,
    /// Optional delay before capture starts.
    pub delay: Option<Duration>,
    /// Include PresentMon v2 metrics when supported.
    pub use_v2_metrics: bool,
    /// Track frame type so generated/interpolated frames can be disclosed.
    pub track_frame_type: bool,
}

impl PresentMonCaptureConfig {
    /// Creates a PresentMon capture configuration for one process.
    pub fn for_process(
        executable_path: impl Into<PathBuf>,
        process_name: impl Into<String>,
        raw_csv: impl Into<PathBuf>,
    ) -> Result<Self, BenchmarkError> {
        let process_name = process_name.into();
        if process_name.trim().is_empty() {
            return Err(BenchmarkError::MissingRequiredField("process_name"));
        }

        Ok(Self {
            executable_path: executable_path.into(),
            process_name,
            paths: PresentMonCapturePaths::from_raw_csv(raw_csv)?,
            duration: Some(DEFAULT_CAPTURE_DURATION),
            delay: None,
            use_v2_metrics: true,
            track_frame_type: true,
        })
    }

    /// Sets a fixed capture duration.
    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets a delay before the capture starts.
    #[must_use]
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Builds a PresentMon console command without starting a process.
    #[must_use]
    pub fn command(&self) -> PresentMonCaptureCommand {
        let mut arguments = vec![
            "--process_name".to_owned(),
            self.process_name.clone(),
            "--output_file".to_owned(),
            self.paths.raw_csv.to_string_lossy().into_owned(),
            "--no_console_stats".to_owned(),
        ];

        if self.use_v2_metrics {
            arguments.push("--v2_metrics".to_owned());
        }

        if self.track_frame_type {
            arguments.push("--track_frame_type".to_owned());
        }

        if let Some(delay) = self.delay {
            arguments.push("--delay".to_owned());
            arguments.push(duration_seconds_arg(delay));
        }

        if let Some(duration) = self.duration {
            arguments.push("--timed".to_owned());
            arguments.push(duration_seconds_arg(duration));
        }

        PresentMonCaptureCommand {
            executable_path: self.executable_path.clone(),
            arguments,
            output_path: self.paths.raw_csv.clone(),
        }
    }
}

/// Process command needed to start a PresentMon-compatible capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentMonCaptureCommand {
    /// PresentMon console executable path.
    pub executable_path: PathBuf,
    /// Command line arguments.
    pub arguments: Vec<String>,
    /// Raw CSV output path.
    pub output_path: PathBuf,
}

/// Session lifecycle state for a benchmark capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSessionState {
    /// Session has been configured but capture has not started.
    Planned,
    /// PresentMon capture is expected to be running.
    Capturing,
    /// Capture was stopped and output is ready to parse.
    Stopped,
    /// Capture was parsed and attached to the session.
    Completed,
    /// Session ended with an error.
    Failed,
}

impl fmt::Display for BenchmarkSessionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Planned => "planned",
            Self::Capturing => "capturing",
            Self::Stopped => "stopped",
            Self::Completed => "completed",
            Self::Failed => "failed",
        };
        formatter.write_str(label)
    }
}

/// Benchmark capture session with explicit lifecycle transitions.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCaptureSession {
    /// Stable local session ID.
    pub id: String,
    /// Metadata captured before the run starts.
    pub metadata: BenchmarkSessionMetadata,
    /// PresentMon capture configuration.
    pub capture_config: PresentMonCaptureConfig,
    /// Current lifecycle state.
    pub state: BenchmarkSessionState,
    /// Parsed raw capture after completion.
    pub capture: Option<PresentMonCapture>,
    /// Failure reason when `state` is `Failed`.
    pub failure_reason: Option<String>,
}

impl BenchmarkCaptureSession {
    /// Creates a planned benchmark capture session.
    pub fn plan(
        id: impl Into<String>,
        metadata: BenchmarkSessionMetadata,
        capture_config: PresentMonCaptureConfig,
    ) -> Result<Self, BenchmarkError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(BenchmarkError::MissingRequiredField("id"));
        }

        Ok(Self {
            id,
            metadata,
            capture_config,
            state: BenchmarkSessionState::Planned,
            capture: None,
            failure_reason: None,
        })
    }

    /// Moves the session into capturing state and returns the command to execute.
    pub fn start(&mut self) -> Result<PresentMonCaptureCommand, BenchmarkError> {
        self.transition(BenchmarkSessionState::Planned, BenchmarkSessionState::Capturing)?;
        Ok(self.capture_config.command())
    }

    /// Marks capture as stopped so output can be parsed.
    pub fn stop(&mut self) -> Result<(), BenchmarkError> {
        self.transition(BenchmarkSessionState::Capturing, BenchmarkSessionState::Stopped)
    }

    /// Parses and attaches a PresentMon CSV capture, completing the session.
    pub fn complete_from_csv(&mut self, csv: &str) -> Result<(), BenchmarkError> {
        if self.state != BenchmarkSessionState::Stopped {
            return Err(BenchmarkError::InvalidSessionTransition {
                from: self.state,
                to: BenchmarkSessionState::Completed,
            });
        }

        self.capture = Some(parse_presentmon_csv(csv)?);
        self.state = BenchmarkSessionState::Completed;
        Ok(())
    }

    /// Marks the session as failed with a user-facing reason.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.failure_reason = Some(reason.into());
        self.state = BenchmarkSessionState::Failed;
    }

    fn transition(
        &mut self,
        expected: BenchmarkSessionState,
        next: BenchmarkSessionState,
    ) -> Result<(), BenchmarkError> {
        if self.state != expected {
            return Err(BenchmarkError::InvalidSessionTransition {
                from: self.state,
                to: next,
            });
        }

        self.state = next;
        Ok(())
    }
}

/// Parsed PresentMon-compatible CSV capture.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentMonCapture {
    /// CSV headers as supplied by the capture file.
    pub headers: Vec<String>,
    /// Raw frame-event rows with normalized fields for downstream metric parsing.
    pub frames: Vec<PresentMonFrameEvent>,
}

impl PresentMonCapture {
    /// Returns the number of parsed frame rows.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Returns true when at least one frame is generated or interpolated.
    #[must_use]
    pub fn has_generated_or_interpolated_frames(&self) -> bool {
        self.frames.iter().any(|frame| {
            matches!(
                frame.frame_type.as_ref(),
                Some(PresentMonFrameType::Generated | PresentMonFrameType::Interpolated)
            )
        })
    }

    /// Calculates aggregate frametime, FPS, dropped-frame, and busy-time metrics.
    pub fn metrics(&self) -> Result<BenchmarkFrameMetrics, BenchmarkError> {
        calculate_capture_metrics(self)
    }
}

/// One PresentMon raw frame-event row.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentMonFrameEvent {
    /// Process executable name from the capture row.
    pub application: String,
    /// Process ID from the capture row.
    pub process_id: Option<u32>,
    /// Present start time in seconds when available.
    pub time_seconds: Option<f64>,
    /// Frame time in milliseconds where the capture exposes it.
    pub frame_time_ms: Option<f64>,
    /// CPU busy time in milliseconds where available.
    pub cpu_busy_ms: Option<f64>,
    /// GPU busy time in milliseconds where available.
    pub gpu_busy_ms: Option<f64>,
    /// Whether PresentMon marked the frame as dropped.
    pub dropped: Option<bool>,
    /// Dropped frame count represented by this row where available.
    pub dropped_frame_count: Option<u32>,
    /// Delayed frame count represented by this row where available.
    pub delayed_frame_count: Option<u32>,
    /// Whether the row represents an application-rendered or generated/interpolated frame.
    pub frame_type: Option<PresentMonFrameType>,
    /// Best available latency measurement for the row.
    pub latency: Option<PresentMonLatencySample>,
    /// Original row values keyed by header.
    pub raw_values: HashMap<String, String>,
}

/// Frame type classification exposed by PresentMon v2 when enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentMonFrameType {
    /// Frame rendered by the application.
    Native,
    /// Frame generated by a driver, SDK, or interpolation layer.
    Generated,
    /// Frame explicitly labeled as interpolated.
    Interpolated,
    /// Non-empty frame type that is not yet normalized by this crate.
    Other(String),
}

/// Latency sample with source labeling so proxy latency is not overclaimed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresentMonLatencySample {
    /// Measurement source.
    pub source: PresentMonLatencySource,
    /// Latency value in milliseconds.
    pub milliseconds: f64,
}

/// PresentMon latency source labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMonLatencySource {
    /// Click-to-photon latency when instrumentation exposes it.
    ClickToPhoton,
    /// PC latency from input to display handoff.
    PcLatency,
    /// Display latency from frame submission to scanout.
    DisplayLatency,
    /// Render-present latency, which is a proxy rather than end-to-end input latency.
    RenderPresentLatency,
}

impl PresentMonLatencySource {
    /// Returns true when this source is a direct click-to-photon measurement.
    #[must_use]
    pub const fn is_click_to_photon(self) -> bool {
        matches!(self, Self::ClickToPhoton)
    }
}

/// Aggregate metrics parsed from a PresentMon-compatible frame capture.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkFrameMetrics {
    /// Parsed frame-event rows, including rows without a usable frame time.
    pub frame_count: usize,
    /// Rows that supplied a positive finite frame time.
    pub measured_frame_count: usize,
    /// Average FPS derived from measured frame times.
    pub average_fps: f64,
    /// One percent low FPS derived from the p99 frametime.
    pub one_percent_low_fps: f64,
    /// Point-one percent low FPS derived from the p99.9 frametime.
    pub point_one_percent_low_fps: f64,
    /// Median frametime in milliseconds.
    pub p50_frame_time_ms: f64,
    /// P95 frametime in milliseconds.
    pub p95_frame_time_ms: f64,
    /// P99 frametime in milliseconds.
    pub p99_frame_time_ms: f64,
    /// Dropped frame count where capture tooling exposes it.
    pub dropped_frames: u32,
    /// Delayed frame count where capture tooling exposes it.
    pub delayed_frames: u32,
    /// CPU busy timing summary when available.
    pub cpu_busy: Option<BusyTimeMetrics>,
    /// GPU busy timing summary when available.
    pub gpu_busy: Option<BusyTimeMetrics>,
    /// Frames explicitly labeled generated or interpolated.
    pub generated_or_interpolated_frame_count: usize,
    /// Frames not explicitly labeled generated or interpolated.
    pub native_or_unknown_frame_count: usize,
}

impl BenchmarkFrameMetrics {
    /// Returns true when generated or interpolated frame rows are present.
    #[must_use]
    pub const fn has_generated_or_interpolated_frames(&self) -> bool {
        self.generated_or_interpolated_frame_count > 0
    }

    /// Converts aggregate metrics into the comparison summary shape.
    #[must_use]
    pub fn to_run_summary(&self, label: impl Into<String>) -> BenchmarkRunSummary {
        BenchmarkRunSummary::new(
            label,
            self.average_fps,
            self.one_percent_low_fps,
            self.point_one_percent_low_fps,
            self.p95_frame_time_ms,
            self.dropped_frames.saturating_add(self.delayed_frames),
        )
    }
}

/// Aggregate CPU or GPU busy timing metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BusyTimeMetrics {
    /// Number of valid busy-time samples.
    pub sample_count: usize,
    /// Average busy time in milliseconds.
    pub average_ms: f64,
    /// P95 busy time in milliseconds.
    pub p95_ms: f64,
}

/// Benchmark crate error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkError {
    /// Required field was empty.
    MissingRequiredField(&'static str),
    /// Capture output path was not a `.csv` path.
    InvalidCapturePath,
    /// Session transition was requested from the wrong state.
    InvalidSessionTransition {
        /// Current state.
        from: BenchmarkSessionState,
        /// Requested next state.
        to: BenchmarkSessionState,
    },
    /// CSV content had no header row.
    EmptyCsv,
    /// CSV header row was missing a required column.
    MissingCsvColumn(&'static str),
    /// CSV syntax was invalid.
    InvalidCsv(String),
    /// Numeric field could not be parsed.
    InvalidNumber {
        /// Column name.
        column: String,
        /// Raw value.
        value: String,
    },
    /// Boolean field could not be parsed.
    InvalidBoolean {
        /// Column name.
        column: String,
        /// Raw value.
        value: String,
    },
    /// Capture rows did not include positive finite frame-time samples.
    MissingFrameTimeSamples,
}

impl fmt::Display for BenchmarkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredField(field) => {
                write!(formatter, "required benchmark field is empty: {field}")
            }
            Self::InvalidCapturePath => formatter.write_str("capture path must be a CSV file path"),
            Self::InvalidSessionTransition { from, to } => {
                write!(formatter, "cannot transition benchmark session from {from} to {to}")
            }
            Self::EmptyCsv => formatter.write_str("PresentMon CSV content is empty"),
            Self::MissingCsvColumn(column) => {
                write!(formatter, "PresentMon CSV is missing required column: {column}")
            }
            Self::InvalidCsv(message) => write!(formatter, "invalid PresentMon CSV: {message}"),
            Self::InvalidNumber { column, value } => {
                write!(formatter, "invalid numeric value in column {column}: {value}")
            }
            Self::InvalidBoolean { column, value } => {
                write!(formatter, "invalid boolean value in column {column}: {value}")
            }
            Self::MissingFrameTimeSamples => {
                formatter.write_str("PresentMon CSV has no usable frame-time samples")
            }
        }
    }
}

impl Error for BenchmarkError {}

/// Parses a PresentMon-compatible raw frame-event CSV.
pub fn parse_presentmon_csv(csv: &str) -> Result<PresentMonCapture, BenchmarkError> {
    let rows = parse_csv_rows(csv)?;
    let Some(headers) = rows.first() else {
        return Err(BenchmarkError::EmptyCsv);
    };

    if headers.is_empty() {
        return Err(BenchmarkError::EmptyCsv);
    }

    let column_map = PresentMonColumnMap::new(headers);
    let application_column = column_map
        .find(&["Application"])
        .ok_or(BenchmarkError::MissingCsvColumn("Application"))?;

    let mut frames = Vec::with_capacity(rows.len().saturating_sub(1));
    for row in rows.iter().skip(1).filter(|row| !is_empty_row(row)) {
        let dropped_frame_count =
            parse_optional_frame_count(&column_map, row, &["Dropped", "Dropped Frames"])?;
        let delayed_frame_count =
            parse_optional_frame_count(&column_map, row, &["Delayed", "Delayed Frames"])?;

        frames.push(PresentMonFrameEvent {
            application: value_at(row, application_column).trim().to_owned(),
            process_id: parse_optional_u32(&column_map, row, &["ProcessID"])?,
            time_seconds: parse_optional_f64(
                &column_map,
                row,
                &["TimeInSeconds", "CPUStartTime"],
            )?,
            frame_time_ms: parse_optional_f64(
                &column_map,
                row,
                &[
                    "FrameTime",
                    "Displayed Frame Time",
                    "Presented Frame Time",
                    "MsBetweenPresents",
                ],
            )?,
            cpu_busy_ms: parse_optional_f64(&column_map, row, &["CPUBusy"])?,
            gpu_busy_ms: parse_optional_f64(&column_map, row, &["GPUBusy", "MsGPUActive"])?,
            dropped: dropped_frame_count.map(|count| count > 0),
            dropped_frame_count,
            delayed_frame_count,
            frame_type: parse_optional_frame_type(&column_map, row)?,
            latency: parse_latency_sample(&column_map, row)?,
            raw_values: raw_value_map(headers, row),
        });
    }

    Ok(PresentMonCapture {
        headers: headers.clone(),
        frames,
    })
}

/// Parses a PresentMon-compatible CSV and returns aggregate benchmark metrics.
pub fn parse_presentmon_metrics(csv: &str) -> Result<BenchmarkFrameMetrics, BenchmarkError> {
    parse_presentmon_csv(csv)?.metrics()
}

/// One measured benchmark run summarized into stable comparison metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkRunSummary {
    /// User-facing label for the benchmarked variant.
    pub label: String,
    /// Average native frames per second.
    pub average_fps: f64,
    /// One percent low native frames per second.
    pub one_percent_low_fps: f64,
    /// Point-one percent low native frames per second.
    pub point_one_percent_low_fps: f64,
    /// P95 frametime in milliseconds, where lower is better.
    pub p95_frame_ms: f64,
    /// Dropped or delayed frame count where capture tooling exposes it.
    pub dropped_frames: u32,
    /// Stability notes that should prevent overconfident recommendations.
    pub stability_warnings: Vec<String>,
}

impl BenchmarkRunSummary {
    /// Creates a benchmark run summary.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        average_fps: f64,
        one_percent_low_fps: f64,
        point_one_percent_low_fps: f64,
        p95_frame_ms: f64,
        dropped_frames: u32,
    ) -> Self {
        Self {
            label: label.into(),
            average_fps,
            one_percent_low_fps,
            point_one_percent_low_fps,
            p95_frame_ms,
            dropped_frames,
            stability_warnings: Vec::new(),
        }
    }

    /// Adds one stability warning to the run summary.
    #[must_use]
    pub fn with_stability_warning(mut self, warning: impl Into<String>) -> Self {
        self.stability_warnings.push(warning.into());
        self
    }

    /// Returns true when this run includes stability warnings.
    #[must_use]
    pub fn has_stability_warnings(&self) -> bool {
        !self.stability_warnings.is_empty()
    }
}

/// Recommendation selected by a benchmark comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkDecision {
    /// Candidate beat the baseline outside the variance band without stability blockers.
    PreferCandidate,
    /// Baseline should be kept because the candidate regressed or had stability blockers.
    KeepBaseline,
    /// Result is inside the variance band or otherwise too close to call.
    Inconclusive,
}

/// Delta summary for two benchmark runs.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkComparisonSummary {
    /// Average FPS percent delta, positive when candidate is higher.
    pub average_fps_delta_percent: f64,
    /// One percent low FPS percent delta, positive when candidate is higher.
    pub one_percent_low_delta_percent: f64,
    /// Point-one percent low FPS percent delta, positive when candidate is higher.
    pub point_one_percent_low_delta_percent: f64,
    /// P95 frametime percent delta, negative when candidate is faster.
    pub p95_frame_time_delta_percent: f64,
    /// Dropped frame delta, positive when candidate dropped more frames.
    pub dropped_frame_delta: i64,
    /// Decision selected after variance and stability checks.
    pub decision: BenchmarkDecision,
    /// Variance band applied to this comparison.
    pub variance_percent: f64,
}

/// Compares a baseline and candidate benchmark run with a variance band.
#[must_use]
pub fn compare_benchmark_runs(
    baseline: &BenchmarkRunSummary,
    candidate: &BenchmarkRunSummary,
    variance_percent: f64,
) -> BenchmarkComparisonSummary {
    let average_fps_delta_percent = percent_delta(baseline.average_fps, candidate.average_fps);
    let one_percent_low_delta_percent =
        percent_delta(baseline.one_percent_low_fps, candidate.one_percent_low_fps);
    let point_one_percent_low_delta_percent = percent_delta(
        baseline.point_one_percent_low_fps,
        candidate.point_one_percent_low_fps,
    );
    let p95_frame_time_delta_percent =
        percent_delta(baseline.p95_frame_ms, candidate.p95_frame_ms);
    let dropped_frame_delta =
        i64::from(candidate.dropped_frames) - i64::from(baseline.dropped_frames);
    let variance_percent = variance_percent.abs();

    let candidate_regressed = candidate.has_stability_warnings()
        || one_percent_low_delta_percent < -variance_percent
        || point_one_percent_low_delta_percent < -variance_percent
        || p95_frame_time_delta_percent > variance_percent
        || dropped_frame_delta > 0 && one_percent_low_delta_percent <= variance_percent;

    let candidate_improved = one_percent_low_delta_percent > variance_percent
        && point_one_percent_low_delta_percent >= -variance_percent
        && p95_frame_time_delta_percent <= variance_percent
        && average_fps_delta_percent >= -variance_percent
        && dropped_frame_delta <= 0;

    let decision = if candidate_regressed {
        BenchmarkDecision::KeepBaseline
    } else if candidate_improved {
        BenchmarkDecision::PreferCandidate
    } else {
        BenchmarkDecision::Inconclusive
    };

    BenchmarkComparisonSummary {
        average_fps_delta_percent,
        one_percent_low_delta_percent,
        point_one_percent_low_delta_percent,
        p95_frame_time_delta_percent,
        dropped_frame_delta,
        decision,
        variance_percent,
    }
}

fn percent_delta(baseline: f64, candidate: f64) -> f64 {
    if baseline.abs() < f64::EPSILON {
        return 0.0;
    }

    ((candidate - baseline) / baseline) * 100.0
}

fn calculate_capture_metrics(
    capture: &PresentMonCapture,
) -> Result<BenchmarkFrameMetrics, BenchmarkError> {
    let frame_time_samples = positive_finite_values(
        capture
            .frames
            .iter()
            .filter_map(|frame| frame.frame_time_ms),
    );
    if frame_time_samples.is_empty() {
        return Err(BenchmarkError::MissingFrameTimeSamples);
    }

    let sorted_frame_times = sorted_values(&frame_time_samples);
    let average_frame_time_ms = average(&frame_time_samples);
    let p50_frame_time_ms = percentile(&sorted_frame_times, 0.50);
    let p95_frame_time_ms = percentile(&sorted_frame_times, 0.95);
    let p99_frame_time_ms = percentile(&sorted_frame_times, 0.99);
    let p999_frame_time_ms = percentile(&sorted_frame_times, 0.999);
    let generated_or_interpolated_frame_count = capture
        .frames
        .iter()
        .filter(|frame| {
            matches!(
                frame.frame_type.as_ref(),
                Some(PresentMonFrameType::Generated | PresentMonFrameType::Interpolated)
            )
        })
        .count();

    Ok(BenchmarkFrameMetrics {
        frame_count: capture.frame_count(),
        measured_frame_count: frame_time_samples.len(),
        average_fps: fps_from_frame_time_ms(average_frame_time_ms),
        one_percent_low_fps: fps_from_frame_time_ms(p99_frame_time_ms),
        point_one_percent_low_fps: fps_from_frame_time_ms(p999_frame_time_ms),
        p50_frame_time_ms,
        p95_frame_time_ms,
        p99_frame_time_ms,
        dropped_frames: sum_frame_counts(capture.frames.iter().map(|frame| {
            frame.dropped_frame_count
                .or_else(|| frame.dropped.map(|dropped| if dropped { 1 } else { 0 }))
        })),
        delayed_frames: sum_frame_counts(
            capture
                .frames
                .iter()
                .map(|frame| frame.delayed_frame_count),
        ),
        cpu_busy: busy_time_metrics(capture.frames.iter().filter_map(|frame| frame.cpu_busy_ms)),
        gpu_busy: busy_time_metrics(capture.frames.iter().filter_map(|frame| frame.gpu_busy_ms)),
        generated_or_interpolated_frame_count,
        native_or_unknown_frame_count: capture
            .frame_count()
            .saturating_sub(generated_or_interpolated_frame_count),
    })
}

fn sum_frame_counts(counts: impl Iterator<Item = Option<u32>>) -> u32 {
    counts.fold(0_u32, |total, count| {
        total.saturating_add(count.unwrap_or_default())
    })
}

fn busy_time_metrics(samples: impl Iterator<Item = f64>) -> Option<BusyTimeMetrics> {
    let samples = non_negative_finite_values(samples);
    if samples.is_empty() {
        return None;
    }

    let sorted_samples = sorted_values(&samples);
    Some(BusyTimeMetrics {
        sample_count: samples.len(),
        average_ms: average(&samples),
        p95_ms: percentile(&sorted_samples, 0.95),
    })
}

fn positive_finite_values(samples: impl Iterator<Item = f64>) -> Vec<f64> {
    samples
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect()
}

fn non_negative_finite_values(samples: impl Iterator<Item = f64>) -> Vec<f64> {
    samples
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect()
}

fn sorted_values(samples: &[f64]) -> Vec<f64> {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
}

fn average(samples: &[f64]) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

fn percentile(sorted_samples: &[f64], percentile: f64) -> f64 {
    debug_assert!(!sorted_samples.is_empty());

    if sorted_samples.len() == 1 {
        return sorted_samples[0];
    }

    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (sorted_samples.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted_samples[lower_index]
    } else {
        let lower = sorted_samples[lower_index];
        let upper = sorted_samples[upper_index];
        lower + (upper - lower) * (position - lower_index as f64)
    }
}

fn fps_from_frame_time_ms(frame_time_ms: f64) -> f64 {
    if frame_time_ms <= 0.0 {
        return 0.0;
    }

    1_000.0 / frame_time_ms
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

/// Benchmark crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "benchmark",
    responsibility: "capture benchmark sessions, parse frame metrics, and score before-after runs",
    requires_live_windows: false,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

#[derive(Debug)]
struct PresentMonColumnMap {
    by_normalized_header: HashMap<String, usize>,
}

impl PresentMonColumnMap {
    fn new(headers: &[String]) -> Self {
        let by_normalized_header = headers
            .iter()
            .enumerate()
            .map(|(index, header)| (normalize_header(header), index))
            .collect();

        Self {
            by_normalized_header,
        }
    }

    fn find(&self, candidates: &[&str]) -> Option<usize> {
        candidates
            .iter()
            .find_map(|candidate| self.by_normalized_header.get(&normalize_header(candidate)))
            .copied()
    }
}

fn parse_optional_u32(
    column_map: &PresentMonColumnMap,
    row: &[String],
    candidates: &[&str],
) -> Result<Option<u32>, BenchmarkError> {
    let Some((column, value)) = optional_value(column_map, row, candidates) else {
        return Ok(None);
    };

    value
        .parse::<u32>()
        .map(Some)
        .map_err(|_| BenchmarkError::InvalidNumber {
            column,
            value: value.to_owned(),
        })
}

fn parse_optional_f64(
    column_map: &PresentMonColumnMap,
    row: &[String],
    candidates: &[&str],
) -> Result<Option<f64>, BenchmarkError> {
    let Some((column, value)) = optional_value(column_map, row, candidates) else {
        return Ok(None);
    };

    value
        .parse::<f64>()
        .map(Some)
        .map_err(|_| BenchmarkError::InvalidNumber {
            column,
            value: value.to_owned(),
        })
}

fn parse_optional_frame_count(
    column_map: &PresentMonColumnMap,
    row: &[String],
    candidates: &[&str],
) -> Result<Option<u32>, BenchmarkError> {
    let Some((column, value)) = optional_value(column_map, row, candidates) else {
        return Ok(None);
    };

    if let Some(boolean) = parse_bool_literal(value) {
        return Ok(Some(if boolean { 1 } else { 0 }));
    }

    value
        .parse::<u32>()
        .map(Some)
        .map_err(|_| BenchmarkError::InvalidNumber {
            column,
            value: value.to_owned(),
        })
}

fn parse_bool_literal(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "0" | "false" | "no" => Some(false),
        "1" | "true" | "yes" => Some(true),
        _ => None,
    }
}

fn parse_optional_frame_type(
    column_map: &PresentMonColumnMap,
    row: &[String],
) -> Result<Option<PresentMonFrameType>, BenchmarkError> {
    let Some((_, value)) = optional_value(column_map, row, &["FrameType"]) else {
        return Ok(None);
    };

    let lowered = value.to_ascii_lowercase();
    let frame_type = if lowered.contains("interpolated") {
        PresentMonFrameType::Interpolated
    } else if lowered.contains("generated") {
        PresentMonFrameType::Generated
    } else if lowered.contains("application") || lowered.contains("native") {
        PresentMonFrameType::Native
    } else {
        PresentMonFrameType::Other(value.to_owned())
    };

    Ok(Some(frame_type))
}

fn parse_latency_sample(
    column_map: &PresentMonColumnMap,
    row: &[String],
) -> Result<Option<PresentMonLatencySample>, BenchmarkError> {
    const SOURCES: &[(&str, PresentMonLatencySource)] = &[
        ("MsClickToPhotonLatency", PresentMonLatencySource::ClickToPhoton),
        ("MsPCLatency", PresentMonLatencySource::PcLatency),
        ("DisplayLatency", PresentMonLatencySource::DisplayLatency),
        (
            "MsRenderPresentLatency",
            PresentMonLatencySource::RenderPresentLatency,
        ),
    ];

    for (column, source) in SOURCES {
        if let Some(milliseconds) = parse_optional_f64(column_map, row, &[*column])? {
            return Ok(Some(PresentMonLatencySample {
                source: *source,
                milliseconds,
            }));
        }
    }

    Ok(None)
}

fn optional_value<'row>(
    column_map: &PresentMonColumnMap,
    row: &'row [String],
    candidates: &[&str],
) -> Option<(String, &'row str)> {
    for candidate in candidates {
        let Some(index) = column_map.find(&[*candidate]) else {
            continue;
        };
        let value = value_at(row, index).trim();
        if !value.is_empty() {
            return Some(((*candidate).to_owned(), value));
        }
    }

    None
}

fn parse_csv_rows(csv: &str) -> Result<Vec<Vec<String>>, BenchmarkError> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = csv.chars().peekable();
    let mut in_quotes = false;

    while let Some(character) = chars.next() {
        match character {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                let _ = chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                row.push(std::mem::take(&mut field));
            }
            '\n' if !in_quotes => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            '\r' if !in_quotes => {
                if chars.peek() == Some(&'\n') {
                    let _ = chars.next();
                }
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(character),
        }
    }

    if in_quotes {
        return Err(BenchmarkError::InvalidCsv(
            "unterminated quoted field".to_owned(),
        ));
    }

    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }

    Ok(rows)
}

fn raw_value_map(headers: &[String], row: &[String]) -> HashMap<String, String> {
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.clone(), value_at(row, index).to_owned()))
        .collect()
}

fn value_at(row: &[String], index: usize) -> &str {
    row.get(index).map_or("", String::as_str)
}

fn normalize_header(header: &str) -> String {
    header
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_empty_row(row: &[String]) -> bool {
    row.iter().all(|value| value.trim().is_empty())
}

fn duration_seconds_arg(duration: Duration) -> String {
    duration.as_secs().max(1).to_string()
}

fn ensure_csv_path(path: &Path) -> Result<(), BenchmarkError> {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return Err(BenchmarkError::InvalidCapturePath);
    };

    if extension.eq_ignore_ascii_case("csv") {
        Ok(())
    } else {
        Err(BenchmarkError::InvalidCapturePath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "benchmark");
        assert!(info.responsibility.contains("frame metrics"));
        assert!(!info.requires_live_windows);
    }

    #[test]
    fn builds_presentmon_console_command_with_capture_path() {
        let config = PresentMonCaptureConfig::for_process(
            "PresentMon-2.4.1-x64.exe",
            "TslGame.exe",
            "captures/pubg-before.csv",
        )
        .expect("capture config should be valid")
        .with_duration(Duration::from_secs(90))
        .with_delay(Duration::from_secs(5));

        let command = config.command();

        assert_eq!(command.executable_path, PathBuf::from("PresentMon-2.4.1-x64.exe"));
        assert_eq!(command.output_path, PathBuf::from("captures/pubg-before.csv"));
        assert_eq!(
            command.arguments,
            vec![
                "--process_name",
                "TslGame.exe",
                "--output_file",
                "captures/pubg-before.csv",
                "--no_console_stats",
                "--v2_metrics",
                "--track_frame_type",
                "--delay",
                "5",
                "--timed",
                "90",
            ]
        );
        assert_eq!(
            config.paths.stats_csv,
            PathBuf::from("captures/pubg-before-stats.csv")
        );
    }

    #[test]
    fn enforces_benchmark_session_lifecycle() {
        let metadata = BenchmarkSessionMetadata::new(
            "PUBG",
            "Windows 11 26100",
            "551.86",
            "Liiiraa Boost - Competitive",
            "pubg-competitive-v1",
            "2026-05-02T15:00:00Z",
        )
        .with_session_label("Training mode route");
        let config = PresentMonCaptureConfig::for_process(
            "PresentMon.exe",
            "TslGame.exe",
            "captures/session.csv",
        )
        .expect("capture config should be valid");
        let mut session = BenchmarkCaptureSession::plan("session-001", metadata, config)
            .expect("session should plan");

        assert_eq!(session.state, BenchmarkSessionState::Planned);
        let command = session.start().expect("planned session should start");
        assert_eq!(command.output_path, PathBuf::from("captures/session.csv"));
        assert_eq!(session.state, BenchmarkSessionState::Capturing);
        assert!(matches!(
            session.start(),
            Err(BenchmarkError::InvalidSessionTransition {
                from: BenchmarkSessionState::Capturing,
                to: BenchmarkSessionState::Capturing
            })
        ));

        session.stop().expect("capturing session should stop");
        session
            .complete_from_csv(
                "Application,ProcessID,FrameTime,Dropped\nTslGame.exe,42,6.94,false\n",
            )
            .expect("stopped session should complete from csv");

        assert_eq!(session.state, BenchmarkSessionState::Completed);
        assert_eq!(session.capture.as_ref().map(PresentMonCapture::frame_count), Some(1));
    }

    #[test]
    fn parses_presentmon_v2_frame_rows_and_latency_source() {
        let csv = concat!(
            "Application,ProcessID,FrameTime,CPUBusy,GPUBusy,Dropped,FrameType,MsPCLatency\n",
            "\"TslGame.exe\",128,6.94,2.10,5.80,0,Application,14.2\n",
            "\"TslGame.exe\",128,7.20,2.22,6.00,1,\"Generated Frame\",15.4\n",
        );

        let capture = parse_presentmon_csv(csv).expect("PresentMon CSV should parse");

        assert_eq!(capture.frame_count(), 2);
        assert!(capture.has_generated_or_interpolated_frames());
        assert_eq!(capture.frames[0].application, "TslGame.exe");
        assert_eq!(capture.frames[0].process_id, Some(128));
        assert_eq!(capture.frames[0].frame_time_ms, Some(6.94));
        assert_eq!(capture.frames[0].cpu_busy_ms, Some(2.10));
        assert_eq!(capture.frames[0].gpu_busy_ms, Some(5.80));
        assert_eq!(capture.frames[0].dropped, Some(false));
        assert_eq!(capture.frames[0].frame_type, Some(PresentMonFrameType::Native));
        assert_eq!(
            capture.frames[0].latency,
            Some(PresentMonLatencySample {
                source: PresentMonLatencySource::PcLatency,
                milliseconds: 14.2,
            })
        );
        assert_eq!(
            capture.frames[1].frame_type,
            Some(PresentMonFrameType::Generated)
        );
        assert_eq!(capture.frames[1].dropped, Some(true));
    }

    #[test]
    fn parses_presentmon_v1_compatible_frame_rows() {
        let csv = concat!(
            "Application,ProcessID,TimeInSeconds,MsBetweenPresents,MsGPUActive,Dropped\n",
            "TslGame.exe,128,10.5,8.33,7.80,false\n",
        );

        let capture = parse_presentmon_csv(csv).expect("PresentMon v1 CSV should parse");
        let frame = &capture.frames[0];

        assert_eq!(frame.time_seconds, Some(10.5));
        assert_eq!(frame.frame_time_ms, Some(8.33));
        assert_eq!(frame.gpu_busy_ms, Some(7.80));
        assert_eq!(frame.dropped, Some(false));
    }

    #[test]
    fn calculates_frametime_metrics_from_presentmon_csv_fixture() {
        let csv = concat!(
            "Application,FrameTime,CPUBusy,GPUBusy,Dropped,Delayed Frames,FrameType\n",
            "TslGame.exe,5.0,2.0,4.0,false,0,Application\n",
            "TslGame.exe,10.0,4.0,8.0,false,0,Application\n",
            "TslGame.exe,15.0,6.0,12.0,true,2,Application\n",
            "TslGame.exe,20.0,8.0,16.0,false,0,\"Generated Frame\"\n",
            "TslGame.exe,25.0,10.0,20.0,false,0,Application\n",
            "TslGame.exe,40.0,12.0,24.0,false,1,Interpolated\n",
        );

        let metrics = parse_presentmon_metrics(csv).expect("fixture metrics should parse");

        assert_eq!(metrics.frame_count, 6);
        assert_eq!(metrics.measured_frame_count, 6);
        assert_close(metrics.average_fps, 52.1739);
        assert_close(metrics.one_percent_low_fps, 25.4777);
        assert_close(metrics.point_one_percent_low_fps, 25.0470);
        assert_close(metrics.p50_frame_time_ms, 17.5);
        assert_close(metrics.p95_frame_time_ms, 36.25);
        assert_close(metrics.p99_frame_time_ms, 39.25);
        assert_eq!(metrics.dropped_frames, 1);
        assert_eq!(metrics.delayed_frames, 3);
        assert_eq!(metrics.generated_or_interpolated_frame_count, 2);
        assert_eq!(metrics.native_or_unknown_frame_count, 4);
        assert!(metrics.has_generated_or_interpolated_frames());

        let cpu_busy = metrics.cpu_busy.expect("CPU Busy should be available");
        assert_eq!(cpu_busy.sample_count, 6);
        assert_close(cpu_busy.average_ms, 7.0);
        assert_close(cpu_busy.p95_ms, 11.5);

        let gpu_busy = metrics.gpu_busy.expect("GPU Busy should be available");
        assert_eq!(gpu_busy.sample_count, 6);
        assert_close(gpu_busy.average_ms, 14.0);
        assert_close(gpu_busy.p95_ms, 23.0);

        let summary = metrics.to_run_summary("fixture");
        assert_eq!(summary.label, "fixture");
        assert_close(summary.p95_frame_ms, 36.25);
        assert_eq!(summary.dropped_frames, 4);
    }

    #[test]
    fn parses_numeric_dropped_frame_counts() {
        let csv = concat!(
            "Application,FrameTime,Dropped Frames\n",
            "TslGame.exe,16.67,2\n",
        );

        let capture = parse_presentmon_csv(csv).expect("numeric dropped count should parse");
        let frame = &capture.frames[0];

        assert_eq!(frame.dropped, Some(true));
        assert_eq!(frame.dropped_frame_count, Some(2));
        assert_eq!(
            capture.metrics().expect("metrics should calculate").dropped_frames,
            2
        );
    }

    #[test]
    fn rejects_metrics_without_frame_time_samples() {
        let error = parse_presentmon_metrics(
            "Application,ProcessID,CPUBusy,GPUBusy\nTslGame.exe,128,2.0,5.0\n",
        )
        .expect_err("metrics need at least one frame time sample");

        assert_eq!(error, BenchmarkError::MissingFrameTimeSamples);
    }

    #[test]
    fn rejects_capture_paths_without_csv_extension() {
        let error = PresentMonCapturePaths::from_raw_csv("captures/session.txt")
            .expect_err("non-csv capture path should fail");

        assert_eq!(error, BenchmarkError::InvalidCapturePath);
    }

    #[test]
    fn prefers_candidate_when_stability_metrics_improve_outside_variance() {
        let baseline = BenchmarkRunSummary::new("DX11", 176.0, 127.0, 92.0, 10.2, 4);
        let candidate = BenchmarkRunSummary::new("DX11 Enhanced", 181.0, 139.0, 99.0, 9.5, 2);

        let comparison =
            compare_benchmark_runs(&baseline, &candidate, DEFAULT_BENCHMARK_VARIANCE_PERCENT);

        assert_eq!(comparison.decision, BenchmarkDecision::PreferCandidate);
        assert!(comparison.one_percent_low_delta_percent > DEFAULT_BENCHMARK_VARIANCE_PERCENT);
        assert!(comparison.p95_frame_time_delta_percent < 0.0);
    }

    #[test]
    fn keeps_baseline_when_candidate_adds_stability_warning() {
        let baseline = BenchmarkRunSummary::new("DX11", 176.0, 127.0, 92.0, 10.2, 4);
        let candidate = BenchmarkRunSummary::new("DX11 Enhanced", 181.0, 139.0, 99.0, 9.5, 2)
            .with_stability_warning("Crash marker detected");

        let comparison =
            compare_benchmark_runs(&baseline, &candidate, DEFAULT_BENCHMARK_VARIANCE_PERCENT);

        assert_eq!(comparison.decision, BenchmarkDecision::KeepBaseline);
    }

    #[test]
    fn treats_close_results_as_inconclusive() {
        let baseline = BenchmarkRunSummary::new("DX11", 176.0, 127.0, 92.0, 10.2, 4);
        let candidate = BenchmarkRunSummary::new("DX11 Enhanced", 177.0, 129.0, 93.0, 10.1, 4);

        let comparison =
            compare_benchmark_runs(&baseline, &candidate, DEFAULT_BENCHMARK_VARIANCE_PERCENT);

        assert_eq!(comparison.decision, BenchmarkDecision::Inconclusive);
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
