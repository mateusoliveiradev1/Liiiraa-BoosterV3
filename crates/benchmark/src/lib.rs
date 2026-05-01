//! Benchmark capture, parsing, and scoring primitives.

/// Default variance band used when a benchmark comparison has no stronger sample model yet.
pub const DEFAULT_BENCHMARK_VARIANCE_PERCENT: f64 = 3.0;

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
    let dropped_frame_delta = i64::from(candidate.dropped_frames) - i64::from(baseline.dropped_frames);
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
}
