import { useMemo, useState, type CSSProperties, type ReactNode } from "react";
import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { BenchmarkWorkflowView } from "../components/OptimizationWorkflow";
import { createDefaultPrivacyConsentState, evaluateDesktopPrivacyGate } from "../privacyConsent";

type WorkflowTone = "active" | "danger" | "lab" | "neutral" | "success" | "warning";

type BenchmarkPoint = {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  p95FrameMs: number;
  tone: WorkflowTone;
};

type BenchmarkData = {
  chart: BenchmarkPoint[];
  metadata: Array<[string, string]>;
  summary: {
    confidence: string;
    decision: string;
    varianceBand: string;
  };
};

type FrameSample = {
  index: number;
  timeSeconds: number;
  baselineFps: number;
  safePlanFps: number;
  profileFps: number;
  p95FrameMs: number;
  droppedFrames: number;
};

const CHART_POINT_BUDGET = 96;
const SAMPLE_COUNT = 720;
const VIRTUAL_ROW_COUNT = 18;

const toneAccent: Record<WorkflowTone, string> = {
  active: "#27d7ff",
  danger: "#ff5a67",
  lab: "#9b7cff",
  neutral: "#9aa8b8",
  success: "#3af28f",
  warning: "#ffbd5a"
};

export function BenchmarksRoute() {
  const benchmarkData = optimizationWorkflow.gaming.benchmarks as BenchmarkData;
  const benchmarkSyncGate = evaluateDesktopPrivacyGate({
    consent: createDefaultPrivacyConsentState(),
    kind: "benchmark-sync"
  });
  const frameSamples = useMemo(() => createFrameSamples(benchmarkData.chart), [benchmarkData.chart]);
  const chartSamples = useMemo(
    () => downsampleFrameSamples(frameSamples, CHART_POINT_BUDGET),
    [frameSamples]
  );
  const [sampleWindowStart, setSampleWindowStart] = useState(0);
  const maxWindowStart = Math.max(0, frameSamples.length - VIRTUAL_ROW_COUNT);
  const visibleSamples = frameSamples.slice(sampleWindowStart, sampleWindowStart + VIRTUAL_ROW_COUNT);

  const moveSampleWindow = (offset: number) => {
    setSampleWindowStart((current: number) => clamp(current + offset, 0, maxWindowStart));
  };

  return (
    <div style={viewGridStyle}>
      <BenchmarkWorkflowView data={optimizationWorkflow.gaming.benchmarks} />
      <BenchmarkResultCharts
        chartSamples={chartSamples}
        data={benchmarkData}
        sampleCount={frameSamples.length}
        visibleSamples={visibleSamples}
        windowStart={sampleWindowStart}
        onMoveWindow={moveSampleWindow}
      />
      <section className="surface" data-tone={benchmarkSyncGate.tone}>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Privacy gate</p>
            <h2>Benchmark cloud sync</h2>
          </div>
          <span className="pill pill--active">{benchmarkSyncGate.value}</span>
        </div>
        <p className="workflow-muted">{benchmarkSyncGate.message}</p>
      </section>
    </div>
  );
}

function BenchmarkResultCharts({
  chartSamples,
  data,
  sampleCount,
  visibleSamples,
  windowStart,
  onMoveWindow
}: {
  chartSamples: FrameSample[];
  data: BenchmarkData;
  sampleCount: number;
  visibleSamples: FrameSample[];
  windowStart: number;
  onMoveWindow: (offset: number) => void;
}) {
  const lastVisibleIndex = Math.min(windowStart + visibleSamples.length, sampleCount);

  return (
    <section style={viewGridStyle} aria-label="Benchmark result charts">
      <RouteSurface
        eyebrow={`${chartSamples.length}/${sampleCount} chart points`}
        title="Frame-time result charts"
        badge={data.summary.confidence}
      >
        <div style={resultSummaryStyle}>
          <ResultStat label="Decision" value={data.summary.decision} tone="active" />
          <ResultStat label="Variance band" value={data.summary.varianceBand} tone="warning" />
          <ResultStat label="Capture rows" value={sampleCount.toLocaleString("en-US")} tone="neutral" />
          <ResultStat label="Visible rows" value={`${windowStart + 1}-${lastVisibleIndex}`} tone="success" />
        </div>
      </RouteSurface>

      <div style={chartGridStyle}>
        <RouteSurface eyebrow="FPS over capture" title="Capture trend">
          <FrameRateTrend samples={chartSamples} />
        </RouteSurface>
        <RouteSurface eyebrow="Average, lows, p95" title="Result comparison">
          <ResultComparisonBars points={data.chart} />
        </RouteSurface>
      </div>

      <div style={chartGridStyle}>
        <RouteSurface eyebrow="Frame samples" title="Sample ledger">
          <SampleLedger
            sampleCount={sampleCount}
            samples={visibleSamples}
            windowStart={windowStart}
            onMoveWindow={onMoveWindow}
          />
        </RouteSurface>
        <RouteSurface eyebrow="Required capture context" title="Run metadata">
          <RunMetadata data={data} sampleCount={sampleCount} chartPointCount={chartSamples.length} />
        </RouteSurface>
      </div>
    </section>
  );
}

function RouteSurface({
  title,
  eyebrow,
  badge,
  children
}: {
  title: string;
  eyebrow: string;
  badge?: string;
  children: ReactNode;
}) {
  return (
    <section className="surface">
      <div className="section-heading">
        <div>
          <p className="eyebrow">{eyebrow}</p>
          <h2>{title}</h2>
        </div>
        {badge ? <span className="pill pill--active">{badge}</span> : null}
      </div>
      {children}
    </section>
  );
}

function ResultStat({
  label,
  value,
  tone
}: {
  label: string;
  value: string;
  tone: WorkflowTone;
}) {
  return (
    <div style={{ ...resultStatStyle, borderColor: toneAccent[tone] }} data-tone={tone}>
      <span className="workflow-muted">{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function FrameRateTrend({ samples }: { samples: FrameSample[] }) {
  const width = 760;
  const height = 260;
  const pad = { top: 18, right: 20, bottom: 34, left: 48 };
  const allValues = samples.flatMap((sample) => [
    sample.baselineFps,
    sample.safePlanFps,
    sample.profileFps
  ]);
  const minFps = Math.floor(Math.min(...allValues) / 10) * 10;
  const maxFps = Math.ceil(Math.max(...allValues) / 10) * 10;
  const baselinePoints = toPolyline(samples, "baselineFps", minFps, maxFps, width, height, pad);
  const safePlanPoints = toPolyline(samples, "safePlanFps", minFps, maxFps, width, height, pad);
  const profilePoints = toPolyline(samples, "profileFps", minFps, maxFps, width, height, pad);

  return (
    <div style={chartShellStyle}>
      <svg
        aria-label="Downsampled FPS trend comparing baseline, safe plan, and PUBG profile"
        role="img"
        viewBox={`0 0 ${width} ${height}`}
        style={svgStyle}
      >
        <line
          x1={pad.left}
          x2={width - pad.right}
          y1={pad.top}
          y2={pad.top}
          stroke="#344252"
          strokeWidth="1"
        />
        <line
          x1={pad.left}
          x2={width - pad.right}
          y1={height - pad.bottom}
          y2={height - pad.bottom}
          stroke="#344252"
          strokeWidth="1"
        />
        <line
          x1={pad.left}
          x2={pad.left}
          y1={pad.top}
          y2={height - pad.bottom}
          stroke="#344252"
          strokeWidth="1"
        />
        <text x="0" y={pad.top + 4} fill="#9aa8b8" fontSize="12">
          {maxFps} FPS
        </text>
        <text x="0" y={height - pad.bottom + 4} fill="#9aa8b8" fontSize="12">
          {minFps} FPS
        </text>
        <polyline fill="none" points={baselinePoints} stroke={toneAccent.neutral} strokeWidth="3" />
        <polyline fill="none" points={safePlanPoints} stroke={toneAccent.active} strokeWidth="3" />
        <polyline fill="none" points={profilePoints} stroke={toneAccent.success} strokeWidth="3" />
        <text x={pad.left} y={height - 8} fill="#9aa8b8" fontSize="12">
          0s
        </text>
        <text x={width - pad.right - 46} y={height - 8} fill="#9aa8b8" fontSize="12">
          300s
        </text>
      </svg>
      <div style={legendStyle} aria-label="Trend legend">
        <LegendItem label="Baseline" tone="neutral" />
        <LegendItem label="Safe plan" tone="active" />
        <LegendItem label="PUBG profile" tone="success" />
      </div>
    </div>
  );
}

function ResultComparisonBars({ points }: { points: BenchmarkPoint[] }) {
  const maxFps = Math.max(...points.flatMap((point) => [point.averageFps, point.onePercentLow]), 1);
  const maxFrameMs = Math.max(...points.map((point) => point.p95FrameMs), 1);

  return (
    <div style={comparisonStyle}>
      {points.map((point) => (
        <div style={comparisonRowStyle} data-tone={point.tone} key={point.id}>
          <div style={comparisonLabelStyle}>
            <strong>{point.label}</strong>
            <span className="workflow-muted">p95 {point.p95FrameMs} ms</span>
          </div>
          <MetricBar
            label="Average"
            tone={point.tone}
            value={`${point.averageFps} FPS`}
            width={point.averageFps / maxFps}
          />
          <MetricBar
            label="1% low"
            tone={point.tone}
            value={`${point.onePercentLow} FPS`}
            width={point.onePercentLow / maxFps}
          />
          <MetricBar
            label="p95 frame"
            tone={point.tone}
            value={`${point.p95FrameMs} ms`}
            width={point.p95FrameMs / maxFrameMs}
          />
        </div>
      ))}
    </div>
  );
}

function MetricBar({
  label,
  value,
  width,
  tone
}: {
  label: string;
  value: string;
  width: number;
  tone: WorkflowTone;
}) {
  return (
    <div style={metricBarStyle}>
      <span className="workflow-muted">{label}</span>
      <span style={metricBarTrackStyle} aria-hidden="true">
        <span
          style={{
            ...metricBarFillStyle,
            background: toneAccent[tone],
            width: `${Math.max(10, Math.round(width * 100))}%`
          }}
        />
      </span>
      <strong>{value}</strong>
    </div>
  );
}

function SampleLedger({
  samples,
  sampleCount,
  windowStart,
  onMoveWindow
}: {
  samples: FrameSample[];
  sampleCount: number;
  windowStart: number;
  onMoveWindow: (offset: number) => void;
}) {
  const firstRow = windowStart + 1;
  const lastRow = Math.min(windowStart + samples.length, sampleCount);
  const canMoveBack = windowStart > 0;
  const canMoveForward = lastRow < sampleCount;

  return (
    <div style={viewGridStyle}>
      <div className="action-bar" aria-label="Frame sample window controls">
        <button
          className="button button--secondary"
          disabled={!canMoveBack}
          onClick={() => onMoveWindow(-VIRTUAL_ROW_COUNT)}
          title="Show previous frame samples"
          type="button"
        >
          Previous
        </button>
        <button
          className="button button--secondary"
          disabled={!canMoveForward}
          onClick={() => onMoveWindow(VIRTUAL_ROW_COUNT)}
          title="Show next frame samples"
          type="button"
        >
          Next
        </button>
        <span className="workflow-muted" style={windowSummaryStyle}>
          Rows {firstRow}-{lastRow} of {sampleCount.toLocaleString("en-US")}
        </span>
      </div>
      <div style={sampleTableShellStyle}>
        <div
          role="table"
          aria-label="Frame sample ledger"
          aria-rowcount={sampleCount + 1}
          style={sampleTableStyle}
        >
          <div role="row" style={sampleHeaderStyle}>
            <span role="columnheader">Frame</span>
            <span role="columnheader">Time</span>
            <span role="columnheader">Baseline</span>
            <span role="columnheader">Safe plan</span>
            <span role="columnheader">PUBG profile</span>
            <span role="columnheader">Dropped</span>
          </div>
          {samples.map((sample) => (
            <div role="row" style={sampleRowStyle} key={sample.index}>
              <span role="cell">#{sample.index + 1}</span>
              <span role="cell">{sample.timeSeconds.toFixed(1)}s</span>
              <span role="cell">{sample.baselineFps.toFixed(1)} FPS</span>
              <span role="cell">{sample.safePlanFps.toFixed(1)} FPS</span>
              <span role="cell">{sample.profileFps.toFixed(1)} FPS</span>
              <span role="cell">{sample.droppedFrames}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function RunMetadata({
  data,
  sampleCount,
  chartPointCount
}: {
  data: BenchmarkData;
  sampleCount: number;
  chartPointCount: number;
}) {
  const items: Array<[string, string]> = [
    ...data.metadata,
    ["Rendered chart points", `${chartPointCount} of ${sampleCount.toLocaleString("en-US")}`],
    ["Capture overhead", "Manual start only, clean stop required"],
    ["App/profile version", "Recorded with report metadata"]
  ];

  return (
    <dl style={metadataStyle}>
      {items.map(([label, value]) => (
        <div key={label} style={metadataRowStyle}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function LegendItem({ label, tone }: { label: string; tone: WorkflowTone }) {
  return (
    <span style={legendItemStyle}>
      <span style={{ ...legendSwatchStyle, background: toneAccent[tone] }} aria-hidden="true" />
      {label}
    </span>
  );
}

function createFrameSamples(points: BenchmarkPoint[], sampleCount = SAMPLE_COUNT): FrameSample[] {
  const baseline = points[0] ?? {
    averageFps: 160,
    onePercentLow: 110,
    p95FrameMs: 10.8
  };
  const safePlan = points[1] ?? baseline;
  const profile = points[points.length - 1] ?? safePlan;

  return Array.from({ length: sampleCount }, (_, index) => {
    const phase = index / Math.max(sampleCount - 1, 1);
    const wave = Math.sin(phase * Math.PI * 8);
    const microStutter = index % 137 === 0 ? 9 : 0;
    const scenePressure = Math.cos(phase * Math.PI * 5) * 4;
    const baselineFps = deriveFps(baseline, wave, scenePressure, microStutter);
    const safePlanFps = deriveFps(safePlan, wave * 0.75, scenePressure * 0.65, microStutter * 0.55);
    const profileFps = deriveFps(profile, wave * 0.55, scenePressure * 0.45, microStutter * 0.35);
    const p95FrameMs = roundTo(1000 / Math.max(profileFps * 0.94, 1), 2);

    return {
      index,
      timeSeconds: roundTo(phase * 300, 1),
      baselineFps,
      safePlanFps,
      profileFps,
      p95FrameMs,
      droppedFrames: p95FrameMs > profile.p95FrameMs + 1.8 ? 1 : 0
    };
  });
}

function deriveFps(
  point: Pick<BenchmarkPoint, "averageFps" | "onePercentLow">,
  wave: number,
  pressure: number,
  stutter: number
) {
  const stabilityFloor = point.onePercentLow + (point.averageFps - point.onePercentLow) * 0.44;
  const fps = stabilityFloor + wave * 5 + pressure - stutter;

  return roundTo(clamp(fps, point.onePercentLow * 0.82, point.averageFps * 1.04), 1);
}

function downsampleFrameSamples(samples: FrameSample[], pointBudget: number) {
  if (samples.length <= pointBudget) {
    return samples;
  }

  const bucketSize = Math.ceil(samples.length / pointBudget);
  const downsampled: FrameSample[] = [];

  for (let start = 0; start < samples.length; start += bucketSize) {
    const bucket = samples.slice(start, start + bucketSize);
    downsampled.push(averageFrameBucket(bucket));
  }

  return downsampled;
}

function averageFrameBucket(bucket: FrameSample[]): FrameSample {
  const totals = bucket.reduce(
    (sum, sample) => ({
      baselineFps: sum.baselineFps + sample.baselineFps,
      droppedFrames: sum.droppedFrames + sample.droppedFrames,
      p95FrameMs: sum.p95FrameMs + sample.p95FrameMs,
      profileFps: sum.profileFps + sample.profileFps,
      safePlanFps: sum.safePlanFps + sample.safePlanFps,
      timeSeconds: sum.timeSeconds + sample.timeSeconds
    }),
    {
      baselineFps: 0,
      droppedFrames: 0,
      p95FrameMs: 0,
      profileFps: 0,
      safePlanFps: 0,
      timeSeconds: 0
    }
  );
  const count = Math.max(bucket.length, 1);

  return {
    index: bucket[0]?.index ?? 0,
    timeSeconds: roundTo(totals.timeSeconds / count, 1),
    baselineFps: roundTo(totals.baselineFps / count, 1),
    safePlanFps: roundTo(totals.safePlanFps / count, 1),
    profileFps: roundTo(totals.profileFps / count, 1),
    p95FrameMs: roundTo(totals.p95FrameMs / count, 2),
    droppedFrames: totals.droppedFrames
  };
}

function toPolyline(
  samples: FrameSample[],
  key: "baselineFps" | "safePlanFps" | "profileFps",
  minFps: number,
  maxFps: number,
  width: number,
  height: number,
  pad: { top: number; right: number; bottom: number; left: number }
) {
  const plotWidth = width - pad.left - pad.right;
  const plotHeight = height - pad.top - pad.bottom;
  const domain = Math.max(maxFps - minFps, 1);

  return samples
    .map((sample, index) => {
      const x = pad.left + (index / Math.max(samples.length - 1, 1)) * plotWidth;
      const y = pad.top + (1 - (sample[key] - minFps) / domain) * plotHeight;

      return `${roundTo(x, 1)},${roundTo(y, 1)}`;
    })
    .join(" ");
}

function roundTo(value: number, decimals: number) {
  const multiplier = 10 ** decimals;

  return Math.round(value * multiplier) / multiplier;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

const viewGridStyle: CSSProperties = {
  display: "grid",
  gap: "1rem"
};

const chartGridStyle: CSSProperties = {
  alignItems: "start",
  display: "grid",
  gap: "0.9rem",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 27rem), 1fr))"
};

const resultSummaryStyle: CSSProperties = {
  display: "grid",
  gap: "0.75rem",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 10rem), 1fr))"
};

const resultStatStyle: CSSProperties = {
  borderLeft: "3px solid",
  display: "grid",
  gap: "0.35rem",
  minHeight: "4.2rem",
  padding: "0.45rem 0 0.45rem 0.75rem"
};

const chartShellStyle: CSSProperties = {
  display: "grid",
  gap: "0.75rem",
  minWidth: 0
};

const svgStyle: CSSProperties = {
  display: "block",
  height: "auto",
  maxWidth: "100%",
  overflow: "visible"
};

const legendStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "0.75rem"
};

const legendItemStyle: CSSProperties = {
  alignItems: "center",
  color: "#b6c2cf",
  display: "inline-flex",
  fontSize: "0.82rem",
  gap: "0.4rem"
};

const legendSwatchStyle: CSSProperties = {
  borderRadius: "999px",
  display: "inline-block",
  height: "0.65rem",
  width: "0.65rem"
};

const comparisonStyle: CSSProperties = {
  display: "grid",
  gap: "0.85rem"
};

const comparisonRowStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem"
};

const comparisonLabelStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "0.45rem 0.8rem",
  justifyContent: "space-between"
};

const metricBarStyle: CSSProperties = {
  alignItems: "center",
  display: "grid",
  gap: "0.55rem",
  gridTemplateColumns: "minmax(4.8rem, 0.45fr) minmax(4rem, 1fr) minmax(4.5rem, auto)"
};

const metricBarTrackStyle: CSSProperties = {
  background: "#202b37",
  border: "1px solid #344252",
  borderRadius: "999px",
  display: "block",
  height: "0.72rem",
  overflow: "hidden"
};

const metricBarFillStyle: CSSProperties = {
  display: "block",
  height: "100%"
};

const windowSummaryStyle: CSSProperties = {
  alignSelf: "center",
  flex: "1 1 11rem",
  textAlign: "right"
};

const sampleTableShellStyle: CSSProperties = {
  border: "1px solid #2a3541",
  borderRadius: "8px",
  overflowX: "auto"
};

const sampleTableStyle: CSSProperties = {
  display: "grid",
  minWidth: "43rem"
};

const sampleHeaderStyle: CSSProperties = {
  background: "#10161d",
  color: "#7d8a99",
  display: "grid",
  fontSize: "0.72rem",
  fontWeight: 800,
  gap: "0.75rem",
  gridTemplateColumns: "5rem repeat(5, minmax(6.5rem, 1fr))",
  minHeight: "2.4rem",
  padding: "0.55rem 0.7rem",
  textTransform: "uppercase"
};

const sampleRowStyle: CSSProperties = {
  borderTop: "1px solid #2a3541",
  color: "#b6c2cf",
  display: "grid",
  fontFamily: "\"JetBrains Mono\", \"Geist Mono\", Consolas, monospace",
  fontSize: "0.78rem",
  gap: "0.75rem",
  gridTemplateColumns: "5rem repeat(5, minmax(6.5rem, 1fr))",
  minHeight: "2.35rem",
  padding: "0.55rem 0.7rem"
};

const metadataStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem",
  margin: 0
};

const metadataRowStyle: CSSProperties = {
  display: "grid",
  gap: "0.75rem",
  gridTemplateColumns: "minmax(8.5rem, 0.7fr) minmax(0, 1fr)",
  margin: 0
};
