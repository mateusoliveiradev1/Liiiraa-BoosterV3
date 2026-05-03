import { useEffect, useMemo, useState, type CSSProperties, type ReactNode } from "react";
import {
  CheckCircle2,
  CircleStop,
  Cpu,
  Gamepad2,
  HardDrive,
  HeartPulse,
  MemoryStick,
  Network,
  Power,
  Rocket,
  Search,
  Trash2,
  type LucideIcon
} from "lucide-react";
import { optimizerGlossaryKeys, tOptimizer, type OptimizerLocaleKey } from "../../../../packages/ui/src/localization";
import { runDesktopAction, type DesktopActionDescriptor } from "../actionRuntime";
import { desktopToneCssVars } from "../designTokens";
import {
  canUseLiveDashboardTelemetry,
  collectLiveDashboardTelemetry,
  getLiveDashboardTelemetryPreference,
  LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS,
  subscribeLiveDashboardTelemetryPreference,
  type LiveResourceSnapshot
} from "../liveDashboardTelemetry";
import {
  actionIconForLabel,
  ActionButton,
  ApplyTimeline,
  BenchmarkProofChart,
  DiffPanel,
  IconToolbar,
  MetricReadout,
  ModeSegmentedControl,
  RollbackSessionLog,
  TweakLedger,
  modeSegmentsFromGroups,
  riskLevelFromLabel,
  type CoreAction,
  type CoreActionVariant,
  type CoreTone,
  type DiffItem,
  type RollbackSessionLogData,
  type TimelineStep,
  type TweakLedgerRowData
} from "./CorePrimitives";

type WorkflowTone = CoreTone;
type ButtonVariant = "ghost" | "primary" | "secondary";

type DashboardMetric = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: WorkflowTone;
};

type ReadinessSignal = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: WorkflowTone;
};

type OptimizerLane = {
  id: string;
  label: string;
  eyebrow: string;
  summary: string;
  status: string;
  trustSignal: string;
  tone: WorkflowTone;
  primaryAction: CoreAction;
  detailAction?: CoreAction;
  details?: Array<[string, string]>;
};

type ScanScope = {
  id: string;
  label: string;
  detail: string;
  checked: boolean;
};

type ScanState = {
  id: string;
  label: string;
  detail: string;
  state: "active" | "complete" | "pending";
};

type ScanFinding = {
  id: string;
  group: string;
  risk: string;
  title: string;
  detail: string;
  tone: WorkflowTone;
};

type PlanAction = {
  id: string;
  label: string;
  variant: ButtonVariant;
};

type PlanTweak = {
  id: string;
  change: string;
  expectedImpact: string;
  risk: string;
  rollback: string;
  reboot: string;
  confidence: string;
  why: string;
};

type PlanGroup = {
  id: string;
  label: string;
  summary: string;
  tone: WorkflowTone;
  applyEnabled: boolean;
  tweaks: PlanTweak[];
};

type ApplyStep = {
  id: string;
  label: string;
  state: "active" | "complete" | "pending";
  detail: string;
};

type RollbackItem = {
  id: string;
  label: string;
  before: string;
  after: string;
  rollback: string;
  state: string;
};

type RollbackSession = {
  id: string;
  time: string;
  label: string;
  state: string;
  rebootRequired: boolean;
  summary: string;
  items: RollbackItem[];
};

type PowerPlan = {
  id: string;
  label: string;
  mode: string;
  state: string;
  detail: string;
  rollback: string;
  defaults: string;
  tone: WorkflowTone;
};

type NvidiaProfile = {
  id: string;
  label: string;
  scope: string;
  state: string;
  recommendation: string;
  rollback: string;
  tone: WorkflowTone;
};

type PubgDxChoice = {
  id: string;
  label: string;
  evidence: string;
  rollback: string;
  state: string;
  tone: WorkflowTone;
};

type PubgDxBenchmarkStep = {
  id: string;
  label: string;
  detail: string;
  state: "active" | "complete" | "pending";
  tone: WorkflowTone;
};

type PubgDxBenchmarkResult = {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  pointOnePercentLow: number;
  p95FrameMs: number;
  droppedFrames: number;
  verdict: string;
  tone: WorkflowTone;
};

type PubgDxBenchmark = {
  currentMode: string;
  selectedMode: string;
  rationale: string;
  varianceBand: string;
  steps: PubgDxBenchmarkStep[];
  results: PubgDxBenchmarkResult[];
  metadata: Array<[string, string]>;
};

type PubgLaunchOption = {
  id: string;
  token: string;
  reason: string;
  recommendation: string;
  backup: string;
  tone: WorkflowTone;
};

type BenchmarkPoint = {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  pointOnePercentLow: number;
  p95FrameMs: number;
  tone: WorkflowTone;
};

type BenchmarkSummary = {
  score: string;
  confidence: string;
  decision: string;
  varianceBand: string;
  detail: string;
  warnings: ReadinessSignal[];
};

type DashboardData = {
  readinessScore: number;
  activeMode: string;
  activePowerPlan: string;
  driverState: string;
  pubgReadiness: string;
  lastBenchmarkDelta: string;
  rollbackAvailability: string;
  trustState: string;
  metrics: DashboardMetric[];
  readinessSignals: ReadinessSignal[];
};

type ScanData = {
  scopes: ScanScope[];
  states: ScanState[];
  progress: {
    label: string;
    percent: number;
    current: string;
    completed: string[];
  };
  findings: ScanFinding[];
};

type OptimizeData = {
  actions: PlanAction[];
  groups: PlanGroup[];
  applySteps: ApplyStep[];
};

type RollbackData = {
  sessions: RollbackSession[];
};

type PowerData = {
  metrics: DashboardMetric[];
  actions: PlanAction[];
  plans: PowerPlan[];
  rules: ReadinessSignal[];
};

type NvidiaData = {
  metrics: DashboardMetric[];
  actions: PlanAction[];
  profiles: NvidiaProfile[];
  policies: ReadinessSignal[];
  capLogic: Array<[string, string]>;
};

type PubgData = {
  metrics: DashboardMetric[];
  actions: PlanAction[];
  detections: ReadinessSignal[];
  launchOptions: PubgLaunchOption[];
  dxChoices: PubgDxChoice[];
  dxBenchmark: PubgDxBenchmark;
  checklist: ReadinessSignal[];
};

type BenchmarkData = {
  metrics: DashboardMetric[];
  actions: PlanAction[];
  summary: BenchmarkSummary;
  chart: BenchmarkPoint[];
  metadata: Array<[string, string]>;
  sessions: ReadinessSignal[];
};

type DashboardViewProps = {
  data: DashboardData;
  optimizeData?: OptimizeData;
  rollbackData?: RollbackData;
  scanData?: ScanData;
  actions?: ReactNode;
};

type ScanRunPhase = "cancelled" | "complete" | "failed" | "idle" | "scanning";

type ScanRunState = {
  current: string;
  percent: number;
  phase: ScanRunPhase;
  running: boolean;
};

type ScanViewProps = {
  data: ScanData;
  optimizeData?: OptimizeData;
};

type OptimizeViewProps = {
  data: OptimizeData;
};

type RollbackViewProps = {
  data: RollbackData;
};

type PowerViewProps = {
  data: PowerData;
};

type NvidiaViewProps = {
  data: NvidiaData;
};

type PubgViewProps = {
  data: PubgData;
};

type BenchmarkViewProps = {
  data: BenchmarkData;
};

const toneAccent: Record<WorkflowTone, string> = desktopToneCssVars;

const viewGridStyle: CSSProperties = {
  display: "grid",
  gap: "1rem"
};

const twoColumnStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 22rem), 1fr))",
  gap: "0.9rem",
  alignItems: "start"
};

const compactRowStyle: CSSProperties = {
  display: "grid",
  gap: "0.55rem",
  minWidth: 0
};

const completedChecksStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "0.45rem"
};

const policyGridStyle: CSSProperties = {
  display: "grid",
  gap: "0.7rem",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 13rem), 1fr))"
};

const policyItemStyle: CSSProperties = {
  alignItems: "start",
  display: "grid",
  gap: "0.65rem",
  gridTemplateColumns: "0.85rem minmax(0, 1fr)",
  minWidth: 0
};

const policyShapeStyle: CSSProperties = {
  border: "2px solid var(--tone, var(--neutral))",
  borderRadius: "var(--radius-sm)",
  display: "block",
  height: "0.85rem",
  marginTop: "0.15rem",
  transform: "rotate(45deg)",
  width: "0.85rem"
};

const planGroupPolicyStyle: CSSProperties = {
  display: "grid",
  gap: "0.75rem",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 13.5rem), 1fr))"
};

const liveDashboardTelemetryHistoryLimit = 60;

export function DashboardWorkflowView({
  data,
  optimizeData,
  rollbackData,
  scanData
}: DashboardViewProps) {
  const [liveSnapshots, setLiveSnapshots] = useState<LiveResourceSnapshot[]>([]);
  const [liveTelemetryEnabled, setLiveTelemetryEnabled] = useState(getLiveDashboardTelemetryPreference);
  const [telemetryError, setTelemetryError] = useState<string | null>(null);
  const liveTelemetryAvailable = useMemo(() => canUseLiveDashboardTelemetry(), []);
  const safeChanges = optimizeData?.groups.find((group) => group.id === "safe")?.tweaks.length ?? 0;
  const rollbackCount = rollbackData?.sessions.length ?? 0;
  const scanPercent = scanData?.progress.percent ?? 0;
  const nextAction = scanData
    ? getScanNextAction(scanData)
    : {
        detail: "Run the read-only inventory before generating a plan or applying changes.",
        label: tOptimizer("actions.startScan"),
        tone: "active" as WorkflowTone,
        value: tOptimizer("labels.ready")
      };
  const bottleneck =
    data.readinessSignals.find((signal) => signal.tone === "danger" || signal.tone === "warning") ??
    data.readinessSignals[0];
  const scoreStyle = {
    "--needle-rotate": `${-116 + Math.max(0, Math.min(100, data.readinessScore)) * 2.32}deg`,
    "--score-angle": `${Math.max(0, Math.min(100, data.readinessScore)) * 2.55}deg`
  } as CSSProperties;
  const dashboardTelemetry = useMemo(
    () =>
      createDashboardTelemetry(
        data,
        optimizeData,
        rollbackData,
        scanData,
        liveSnapshots,
        telemetryError,
        liveTelemetryEnabled,
        liveTelemetryAvailable
      ),
    [
      data,
      liveSnapshots,
      liveTelemetryAvailable,
      liveTelemetryEnabled,
      optimizeData,
      rollbackData,
      scanData,
      telemetryError
    ]
  );
  const activityItems = createDashboardActivityItems(data, optimizeData, rollbackData, scanData, safeChanges);

  useEffect(() => subscribeLiveDashboardTelemetryPreference(setLiveTelemetryEnabled), []);

  useEffect(() => {
    if (!liveTelemetryEnabled) {
      setTelemetryError(null);
      return undefined;
    }

    if (!liveTelemetryAvailable) {
      return undefined;
    }

    let cancelled = false;
    let inFlight = false;
    const refreshTelemetry = async () => {
      if (inFlight || document.visibilityState === "hidden") {
        return;
      }

      inFlight = true;

      try {
        const snapshot = await collectLiveDashboardTelemetry();

        if (cancelled) {
          return;
        }

        setTelemetryError(null);
        setLiveSnapshots((current) => [...current.slice(-(liveDashboardTelemetryHistoryLimit - 1)), snapshot]);
      } catch (error) {
        if (!cancelled) {
          setTelemetryError(error instanceof Error ? error.message : "Live desktop telemetry is unavailable.");
        }
      } finally {
        inFlight = false;
      }
    };

    void refreshTelemetry();
    const interval = window.setInterval(() => void refreshTelemetry(), LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS);
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        void refreshTelemetry();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [liveTelemetryAvailable, liveTelemetryEnabled]);

  return (
    <div className="booster-dashboard" aria-label="Dashboard optimization overview">
      <section className="boost-stage" aria-label="System boost overview">
        <div className="speedometer" style={scoreStyle} aria-label={`Performance score ${data.readinessScore} of 100`}>
          <span className="speedometer__arc" aria-hidden="true" />
          <span className="speedometer__needle" aria-hidden="true" />
          <span className="speedometer__tick speedometer__tick--zero">0</span>
          <span className="speedometer__tick speedometer__tick--mid">50</span>
          <span className="speedometer__tick speedometer__tick--high">75</span>
          <span className="speedometer__tick speedometer__tick--max">100</span>
          <strong>{data.readinessScore}</strong>
          <span className="speedometer__total">/ 100</span>
          <small>Performance score</small>
          <b>Excellent</b>
        </div>

        <div className="boost-stage__copy">
          <span className="boost-stage__kicker">Next action</span>
          <h2>Safe boost is staged.</h2>
          <p>
            {safeChanges} reversible tweaks are ready, rollback is armed, and the current scan is
            {scanData ? ` ${scanPercent}% complete` : " ready to run"}.
          </p>
          <div className="boost-stage__actions">
            <button
              className="boost-button"
              type="button"
              onClick={() =>
                void runDesktopAction({
                  command: "apply-safe-plan",
                  feedback: `${safeChanges} safe tweaks are selected. Opening Smart Boost with rollback checks visible.`,
                  id: "dashboard-run-smart-boost",
                  label: "Run Smart Boost",
                  targetRoute: "optimize"
                })
              }
            >
              <span>Run Smart Boost</span>
              <Rocket aria-hidden="true" size={25} strokeWidth={2.4} />
            </button>
            <button
              className="smart-boost-link"
              type="button"
              onClick={() =>
                void runDesktopAction({
                  command: "review-plan",
                  feedback: "Opening the full tweak plan with every available Safe, Competitive, Lab, and Blocked row.",
                  id: "dashboard-review-tweak-plan",
                  label: "Review tweak plan",
                  targetRoute: "optimize"
                })
              }
            >
              Review tweak plan
              <span aria-hidden="true">&gt;</span>
            </button>
          </div>
        </div>

        <aside className="health-card" aria-label="System readiness">
          <p className="eyebrow">Optimization guardrails</p>
          <div className="health-card__score">
            <HeartPulse aria-hidden="true" size={58} strokeWidth={2.2} />
            <span>
              <strong>Protected</strong>
              <small>Signed updates, backups, and safe defaults are active.</small>
            </span>
          </div>
          <ul>
            {data.readinessSignals.map((signal) => (
              <li data-tone={signal.tone} key={signal.id}>
                <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2.5} />
                <span>{signal.label}: {signal.value}</span>
              </li>
            ))}
          </ul>
        </aside>
      </section>

      <section className="dashboard-live-strip" aria-label="Dashboard live resource metrics">
        <div className="section-heading dashboard-live-strip__heading">
          <div>
            <p className="eyebrow">Live telemetry</p>
            <h2>Resource map</h2>
          </div>
          <span className="pill pill--active" data-tone={dashboardTelemetry.sourceTone}>
            {dashboardTelemetry.sourceLabel}
          </span>
        </div>
        <p className="dashboard-source-note">{dashboardTelemetry.sourceDetail}</p>
        <div className="resource-grid" aria-label="Dashboard metrics">
          {dashboardTelemetry.resources.map((metric) => (
            <ResourceCard key={metric.id} metric={metric} />
          ))}
        </div>
      </section>

      <aside className="dashboard-side-rail" aria-label="Dashboard recommendations">
        <section className="rail-section">
          <p className="eyebrow">Recommended actions</p>
          <RecommendationCard
            action="Review"
            actionIntent={{
              command: "apply-safe-plan",
              feedback: `${safeChanges} safe changes are ready in the dashboard tweak matrix.`,
              id: "dashboard-recommend-safe-boost",
              label: "Review safe boost",
              targetRoute: "optimize"
            }}
            detail={`${safeChanges} safe changes can be applied with backup and readback verification.`}
            icon={Rocket}
            label="Apply safe boost"
          />
          <RecommendationCard
            action="Inspect"
            actionIntent={{
              feedback: bottleneck
                ? `Inspecting ${bottleneck.label} with the related optimization route.`
                : "No blocking signal is active.",
              id: "dashboard-recommend-bottleneck",
              label: bottleneck ? bottleneck.label : "System clean",
              targetRoute: bottleneck?.id === "pubg" ? "pubg" : "optimize"
            }}
            detail={bottleneck?.detail ?? "No blocking signal is currently active."}
            icon={Trash2}
            label={bottleneck ? bottleneck.label : "System clean"}
          />
        </section>

        <section className="rail-section rail-section--activity">
          <p className="eyebrow">Recorded actions</p>
          {activityItems.map((item) => (
            <RecentAction icon={item.icon} key={item.id} label={item.label} meta={item.meta} />
          ))}
        </section>

        <section className="rail-section rail-section--summary">
          <p className="eyebrow">Current bottleneck</p>
          {bottleneck ? (
            <StatusRow
              detail={bottleneck.detail}
              label={bottleneck.label}
              tone={bottleneck.tone}
              value={bottleneck.value}
            />
          ) : null}
          <DefinitionGrid
            items={[
              ["Safe changes", String(safeChanges)],
              ["Rollback", rollbackCount > 0 ? `${rollbackCount} sessions` : data.rollbackAvailability],
              ["Scan", scanData ? `${scanPercent}% complete` : tOptimizer("labels.ready")],
              ["Trust", data.trustState]
            ]}
          />
        </section>
      </aside>

      <div className="dashboard-operations">
        <section className="performance-panel" aria-label="Performance overview">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Performance overview</p>
              <h2>Live system profile</h2>
            </div>
            <div className="chart-legend" aria-label="Chart legend">
              <span className="chart-legend__cpu">CPU</span>
              <span className="chart-legend__ram">RAM</span>
              <span className="chart-legend__disk">DISK</span>
              <span className="chart-legend__network">NET</span>
            </div>
          </div>
          <PerformanceOverviewChart series={dashboardTelemetry.chartSeries} />
          <div className="environment-readouts">
            {dashboardTelemetry.readouts.map((readout) => {
              const Icon = readout.icon;

              return (
                <span data-tone={readout.tone} key={readout.id}>
                  <Icon aria-hidden="true" size={20} strokeWidth={2.2} />
                  <strong>{readout.value}</strong>
                  <b>{readout.label}</b>
                  <small>{readout.detail}</small>
                </span>
              );
            })}
          </div>
        </section>

        <section className="quick-actions-panel" aria-label="Quick actions">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Quick actions</p>
              <h2>Next action</h2>
            </div>
            <span className="pill pill--active">{nextAction.value}</span>
          </div>
          <div className="quick-action-grid">
            <QuickAction
              action={{
                command: "apply-safe-plan",
                feedback: `${safeChanges} reversible safe tweaks are queued for review before apply.`,
                id: "dashboard-quick-apply-safe",
                label: "Apply safe tweaks",
                targetRoute: "optimize"
              }}
              icon={Rocket}
              label="Apply safe tweaks"
              detail={`${safeChanges} reversible changes queued`}
            />
            <QuickAction
              action={{
                command: "run-read-only-scan",
                feedback: "Continuing Smart Scan before Smart Boost opens.",
                id: "dashboard-quick-scan",
                label: "Continue smart scan",
                targetRoute: "scan"
              }}
              icon={Search}
              label="Continue smart scan"
              detail={scanData?.progress.current ?? "Read-only inventory"}
            />
            <QuickAction
              action={{
                command: "rollback",
                feedback: `${rollbackCount} rollback sessions are ready to inspect.`,
                id: "dashboard-quick-recovery",
                label: "Open recovery",
                targetRoute: "rollback"
              }}
              icon={Power}
              label="Open recovery"
              detail={`${rollbackCount} rollback sessions ready`}
            />
            <QuickAction
              action={{
                command: "benchmark",
                feedback: "Opening benchmark proof and before/after comparison state.",
                id: "dashboard-quick-benchmark",
                label: "Benchmark",
                targetRoute: "benchmarks"
              }}
              icon={Gamepad2}
              label="Benchmark"
              detail={data.lastBenchmarkDelta}
            />
          </div>
        </section>
      </div>

      <DashboardOptimizerLanes
        data={data}
        optimizeData={optimizeData}
        rollbackData={rollbackData}
        scanData={scanData}
      />
      {optimizeData ? <DashboardTweakMatrix groups={optimizeData.groups} /> : null}
    </div>
  );
}

type DashboardResourceMetric = {
  id: string;
  label: string;
  value: string;
  detail: string;
  secondaryDetail?: string;
  icon: LucideIcon;
  tone: WorkflowTone;
  usage: number;
  sparkline: number[];
};

type DashboardTelemetry = {
  sourceLabel: string;
  sourceDetail: string;
  sourceTone: WorkflowTone;
  resources: DashboardResourceMetric[];
  chartSeries: {
    cpu: number[];
    ram: number[];
    disk: number[];
    network: number[];
  };
  readouts: Array<{
    id: string;
    icon: LucideIcon;
    label: string;
    value: string;
    detail: string;
    tone: WorkflowTone;
  }>;
};

type DashboardActivityItem = {
  id: string;
  icon: LucideIcon;
  label: string;
  meta: string;
};

function createDashboardTelemetry(
  data: DashboardData,
  optimizeData: OptimizeData | undefined,
  rollbackData: RollbackData | undefined,
  scanData: ScanData | undefined,
  liveSnapshots: LiveResourceSnapshot[],
  telemetryError: string | null,
  liveTelemetryEnabled: boolean,
  liveTelemetryAvailable: boolean
): DashboardTelemetry {
  const latest = liveSnapshots.length > 0 ? liveSnapshots[liveSnapshots.length - 1] : undefined;

  if (latest) {
    const snapshotTime = formatSnapshotTime(latest.collectedAtUtc);
    const sourceDetail = liveTelemetryEnabled
      ? `Updated ${snapshotTime}. Live cards refresh every ${formatSeconds(LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS)} while this screen is open. Read-only monitoring.`
      : `Paused in Settings. Last live reading was ${snapshotTime}.`;

    return {
      chartSeries: createLiveChartSeries(liveSnapshots),
      readouts: createLiveReadouts(latest),
      resources: createLiveResourceCards(latest, liveSnapshots),
      sourceDetail,
      sourceLabel: liveTelemetryEnabled ? "Live system view" : "Live paused",
      sourceTone: liveTelemetryEnabled ? "success" : "neutral"
    };
  }

  if (!liveTelemetryEnabled) {
    return createInactiveDashboardTelemetry(
      "Live paused",
      "Live hardware view is off in Settings. Turn it on when you want current CPU, memory, storage, and network activity.",
      "neutral"
    );
  }

  if (liveTelemetryAvailable) {
    return createInactiveDashboardTelemetry(
      telemetryError ? "Live unavailable" : "Reading system",
      telemetryError
        ? `Live hardware data is unavailable: ${telemetryErrorMessage(telemetryError)} The dashboard is waiting for a desktop reading.`
        : `Reading your system for the first live sample. Cards refresh every ${formatSeconds(LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS)} while this screen is open.`,
      telemetryError ? "warning" : "active"
    );
  }

  return {
    chartSeries: createPreviewChartSeries(scanData),
    readouts: createPreviewReadouts(data, optimizeData, rollbackData, scanData),
    resources: createPreviewResourceCards(),
    sourceDetail: "Live hardware data appears in the desktop app. Browser preview uses safe demo readings.",
    sourceLabel: "Browser preview",
    sourceTone: "neutral"
  };
}

function createInactiveDashboardTelemetry(
  sourceLabel: string,
  sourceDetail: string,
  sourceTone: WorkflowTone
): DashboardTelemetry {
  return {
    chartSeries: createEmptyChartSeries(),
    readouts: [
      {
        detail: "Controlled in Settings",
        icon: Cpu,
        id: "live-monitor-state",
        label: "Live hardware view",
        tone: sourceTone,
        value: sourceLabel
      },
      {
        detail: "Memory stays untracked until live view returns",
        icon: MemoryStick,
        id: "live-monitor-memory",
        label: "Memory",
        tone: "neutral",
        value: "Paused"
      },
      {
        detail: "Storage activity stays untracked",
        icon: HardDrive,
        id: "live-monitor-disk",
        label: "Storage",
        tone: "neutral",
        value: "Paused"
      },
      {
        detail: "Network activity stays untracked",
        icon: Network,
        id: "live-monitor-network",
        label: "Network",
        tone: "neutral",
        value: "Paused"
      }
    ],
    resources: createInactiveResourceCards(sourceTone),
    sourceDetail,
    sourceLabel,
    sourceTone
  };
}

function createDashboardActivityItems(
  data: DashboardData,
  optimizeData: OptimizeData | undefined,
  rollbackData: RollbackData | undefined,
  scanData: ScanData | undefined,
  safeChanges: number
): DashboardActivityItem[] {
  const activeStep = optimizeData?.applySteps.find((step) => step.state === "active");
  const latestRollback = rollbackData?.sessions[0];

  return [
    {
      icon: Rocket,
      id: "safe-queue",
      label: `${safeChanges} safe tweaks staged`,
      meta: activeStep ? `${activeStep.label}: ${activeStep.detail}` : "Awaiting plan review"
    },
    {
      icon: Search,
      id: "scan-state",
      label: scanData ? scanData.progress.label : tOptimizer(optimizerGlossaryKeys.scan),
      meta: scanData ? `${scanData.progress.percent}% - ${scanData.progress.current}` : tOptimizer("labels.ready")
    },
    {
      icon: Power,
      id: "rollback-state",
      label: latestRollback?.label ?? tOptimizer(optimizerGlossaryKeys.rollback),
      meta: latestRollback ? `${latestRollback.state} - ${latestRollback.summary}` : data.rollbackAvailability
    },
    {
      icon: Gamepad2,
      id: "benchmark-state",
      label: "Benchmark proof",
      meta: data.lastBenchmarkDelta
    }
  ];
}

function createLiveResourceCards(
  snapshot: LiveResourceSnapshot,
  history: LiveResourceSnapshot[]
): DashboardResourceMetric[] {
  const series = createLiveChartSeries(history);
  const cpuUsage = clampPercent(snapshot.cpu.usagePercent);
  const memoryUsagePercent = resolveUsedPercent(
    snapshot.memory.usedPercent,
    snapshot.memory.usedBytes,
    snapshot.memory.totalBytes
  );
  const memoryUsage = clampPercent(memoryUsagePercent);
  const diskUsage = clampPercent(snapshot.disk.usedPercent);
  const diskBytesPerSecond = latestDiskBytesPerSecond(history);
  const networkBytesPerSecond = latestNetworkBytesPerSecond(history);
  const networkUsage = networkThroughputPercent(snapshot, networkBytesPerSecond);
  const processorName = formatProcessorName(snapshot.cpu.name);
  const cpuCapability = [
    snapshot.cpu.logicalProcessors ? `${snapshot.cpu.logicalProcessors} threads` : null,
    snapshot.cpu.maxClockMhz ? `up to ${formatCompactClock(snapshot.cpu.maxClockMhz)}` : null
  ]
    .filter(Boolean)
    .join(" - ");

  return [
    {
      detail: processorName ?? "Processor detected",
      icon: Cpu,
      id: "cpu",
      label: "CPU",
      secondaryDetail: cpuCapability || "Usage right now",
      sparkline: series.cpu,
      tone: cpuUsage >= 85 ? "warning" : "success",
      usage: cpuUsage,
      value: formatPercent(snapshot.cpu.usagePercent)
    },
    {
      detail: formatMemoryDetail(snapshot.memory.usedBytes, snapshot.memory.totalBytes),
      icon: MemoryStick,
      id: "ram",
      label: "Memory",
      sparkline: series.ram,
      tone: memoryUsage >= 82 ? "warning" : "benchmark",
      usage: memoryUsage,
      value: formatPercent(memoryUsagePercent)
    },
    {
      detail: `${snapshot.disk.primaryVolume ?? "Storage"} activity now`,
      icon: HardDrive,
      id: "disk",
      label: "Storage",
      secondaryDetail: `${formatBytesPerSecond(diskBytesPerSecond)} moving through storage`,
      sparkline: series.disk,
      tone: diskUsage >= 88 ? "warning" : "rollback",
      usage: diskUsage,
      value: formatPercent(snapshot.disk.usedPercent)
    },
    {
      detail:
        snapshot.network.adapterName ??
        `${snapshot.network.activeAdapters} active adapter${snapshot.network.activeAdapters === 1 ? "" : "s"}`,
      icon: Network,
      id: "network",
      label: "Network",
      secondaryDetail: `${formatBytesPerSecond(networkBytesPerSecond)} now - ${formatBitsPerSecond(snapshot.network.linkSpeedBitsPerSecond)} link`,
      sparkline: series.network,
      tone: "trust",
      usage: networkUsage,
      value: formatBytesPerSecond(networkBytesPerSecond)
    }
  ];
}

function createLiveReadouts(snapshot: LiveResourceSnapshot): DashboardTelemetry["readouts"] {
  return [
    {
      detail: formatProcessorName(snapshot.cpu.name) ?? "Processor detected",
      icon: Cpu,
      id: "cpu-clock",
      label: "Processor speed",
      tone: "success",
      value: snapshot.cpu.maxClockMhz ? formatClock(snapshot.cpu.maxClockMhz) : "Unknown"
    },
    {
      detail: `${formatBytes(snapshot.memory.freeBytes)} free`,
      icon: MemoryStick,
      id: "memory-free",
      label: "Free memory",
      tone: "benchmark",
      value: formatBytes(snapshot.memory.freeBytes)
    },
    {
      detail: snapshot.disk.health ?? "Health unavailable",
      icon: HardDrive,
      id: "disk-health",
      label: "Storage health",
      tone: snapshot.disk.health && /healthy/i.test(snapshot.disk.health) ? "success" : "neutral",
      value: snapshot.disk.health ?? "Unknown"
    },
    {
      detail: snapshot.network.adapterName ?? "Network adapter",
      icon: Network,
      id: "network-link",
      label: "Link speed",
      tone: "trust",
      value: formatBitsPerSecond(snapshot.network.linkSpeedBitsPerSecond)
    }
  ];
}

function createPreviewReadouts(
  data: DashboardData,
  optimizeData: OptimizeData | undefined,
  rollbackData: RollbackData | undefined,
  scanData: ScanData | undefined
): DashboardTelemetry["readouts"] {
  return [
    {
      detail: data.driverState,
      icon: Cpu,
      id: "preview-mode",
      label: "Active mode",
      tone: "success",
      value: data.activeMode
    },
    {
      detail: scanData?.progress.current ?? "Read-only scan preview",
      icon: Search,
      id: "preview-scan",
      label: tOptimizer(optimizerGlossaryKeys.scan),
      tone: "active",
      value: scanData ? `${scanData.progress.percent}%` : tOptimizer("labels.ready")
    },
    {
      detail: optimizeData ? `${optimizeData.groups.length} tweak groups` : "Plan preview",
      icon: Rocket,
      id: "preview-plan",
      label: "Plan state",
      tone: "success",
      value: `${optimizeData?.groups.find((group) => group.id === "safe")?.tweaks.length ?? 0} safe`
    },
    {
      detail: data.rollbackAvailability,
      icon: Power,
      id: "preview-rollback",
      label: tOptimizer(optimizerGlossaryKeys.rollback),
      tone: "rollback",
      value: `${rollbackData?.sessions.length ?? 0} sessions`
    }
  ];
}

function createLiveChartSeries(history: LiveResourceSnapshot[]): DashboardTelemetry["chartSeries"] {
  return {
    cpu: normalizeSeries(history.map((snapshot) => clampPercent(snapshot.cpu.usagePercent))),
    disk: normalizeSeries(createDiskThroughputSeries(history)),
    network: normalizeSeries(createNetworkThroughputSeries(history)),
    ram: createMemoryUsageSeries(history)
  };
}

function createEmptyChartSeries(): DashboardTelemetry["chartSeries"] {
  const emptySeries = normalizeSeries([]);

  return {
    cpu: emptySeries,
    disk: emptySeries,
    network: emptySeries,
    ram: emptySeries
  };
}

function createPreviewChartSeries(scanData: ScanData | undefined): DashboardTelemetry["chartSeries"] {
  const scanPercent = scanData?.progress.percent ?? 52;

  return {
    cpu: [34, 38, 41, 43, 39, 45, 48, 44, 42, 46, 50, 47, 45, 49, 52, 48, 46, 51, 49, scanPercent],
    disk: [16, 18, 15, 21, 17, 19, 14, 24, 18, 20, 17, 22, 19, 21, 18, 26, 22, 20, 19, 23],
    network: [12, 14, 16, 13, 15, 18, 17, 21, 19, 22, 20, 24, 23, 20, 18, 21, 25, 24, 22, 20],
    ram: [46, 48, 47, 49, 50, 51, 49, 52, 53, 52, 54, 55, 53, 56, 55, 54, 56, 57, 55, 56]
  };
}

function normalizeSeries(points: number[], targetLength = liveDashboardTelemetryHistoryLimit) {
  if (points.length === 0) {
    return Array.from({ length: targetLength }, () => 0);
  }

  const lastPoints = points.slice(-targetLength);
  const first = lastPoints[0] ?? 0;

  return [...Array.from({ length: targetLength - lastPoints.length }, () => first), ...lastPoints];
}

function createMemoryUsageSeries(history: LiveResourceSnapshot[]) {
  return normalizeRelativeSeries(
    history.map((snapshot) => {
      if (typeof snapshot.memory.usedBytes === "number" && Number.isFinite(snapshot.memory.usedBytes)) {
        return snapshot.memory.usedBytes;
      }

      return resolveUsedPercent(snapshot.memory.usedPercent, snapshot.memory.usedBytes, snapshot.memory.totalBytes);
    })
  );
}

function normalizeRelativeSeries(
  points: Array<number | null | undefined>,
  targetLength = liveDashboardTelemetryHistoryLimit
) {
  const lastPoints = points.slice(-targetLength);
  const numericPoints = lastPoints.filter((point): point is number => typeof point === "number" && Number.isFinite(point));

  if (numericPoints.length === 0) {
    return normalizeSeries([], targetLength);
  }

  const min = Math.min(...numericPoints);
  const max = Math.max(...numericPoints);
  const range = max - min;
  const scaledPoints =
    range <= 0
      ? lastPoints.map((point) => (typeof point === "number" && Number.isFinite(point) ? 50 : 0))
      : lastPoints.map((point) => {
          if (typeof point !== "number" || !Number.isFinite(point)) {
            return 0;
          }

          return 30 + ((point - min) / range) * 42;
        });

  return normalizeSeries(scaledPoints, targetLength);
}

function ResourceCard({ metric }: { metric: DashboardResourceMetric }) {
  const Icon = metric.icon;
  const usage = clampPercent(metric.usage);
  const meterStyle = {
    "--usage": `${usage}%`,
    "--usage-angle": `${usage * 3.6}deg`
  } as CSSProperties;

  return (
    <article
      className="resource-card"
      data-tone={metric.tone}
      aria-label={`${metric.label}: ${metric.value}`}
    >
      <div className="resource-card__header">
        <Icon aria-hidden="true" size={24} strokeWidth={2.1} />
        <span>{metric.label}</span>
      </div>
      <strong>{metric.value}</strong>
      <span className="resource-card__copy">
        <small>{metric.detail}</small>
        {metric.secondaryDetail ? <small>{metric.secondaryDetail}</small> : null}
      </span>
      <span className="resource-card__meter" style={meterStyle} aria-hidden="true" />
      <Sparkline points={metric.sparkline} />
    </article>
  );
}

function Sparkline({ points }: { points: number[] }) {
  const polyline = points
    .map((point, index) => {
      const x = (index / Math.max(points.length - 1, 1)) * 100;
      const y = 42 - Math.max(0, Math.min(100, point)) * 0.32;

      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  return (
    <svg className="sparkline" viewBox="0 0 100 46" preserveAspectRatio="none" aria-hidden="true">
      <polyline points={polyline} />
    </svg>
  );
}

function PerformanceOverviewChart({ series }: { series: DashboardTelemetry["chartSeries"] }) {
  return (
    <svg
      className="performance-chart"
      viewBox="0 0 620 210"
      role="img"
      aria-label="CPU, RAM, disk, and network activity over recent live samples"
    >
      <g className="performance-chart__grid" aria-hidden="true">
        <line x1="44" x2="596" y1="24" y2="24" />
        <line x1="44" x2="596" y1="72" y2="72" />
        <line x1="44" x2="596" y1="120" y2="120" />
        <line x1="44" x2="596" y1="168" y2="168" />
        <line x1="44" x2="44" y1="24" y2="168" />
      </g>
      <g className="performance-chart__labels" aria-hidden="true">
        <text x="8" y="29">100%</text>
        <text x="14" y="77">75%</text>
        <text x="14" y="125">50%</text>
        <text x="14" y="173">0%</text>
        <text x="42" y="195">60s</text>
        <text x="580" y="195">Now</text>
      </g>
      <polyline className="performance-chart__cpu" points={toChartPolyline(series.cpu)} />
      <polyline className="performance-chart__ram" points={toChartPolyline(series.ram)} />
      <polyline className="performance-chart__disk" points={toChartPolyline(series.disk)} />
      <polyline className="performance-chart__network" points={toChartPolyline(series.network)} />
    </svg>
  );
}

function QuickAction({
  action,
  detail,
  icon: Icon,
  label
}: {
  action: DesktopActionDescriptor;
  detail: string;
  icon: LucideIcon;
  label: string;
}) {
  return (
    <button
      className="quick-action"
      type="button"
      onClick={() => void runDesktopAction(action)}
      aria-label={`${label}: ${detail}`}
    >
      <Icon aria-hidden="true" size={34} strokeWidth={2.1} />
      <span>
        <strong>{label}</strong>
        <small>{detail}</small>
      </span>
    </button>
  );
}

function RecommendationCard({
  action,
  actionIntent,
  detail,
  icon: Icon,
  label
}: {
  action: string;
  actionIntent: DesktopActionDescriptor;
  detail: string;
  icon: LucideIcon;
  label: string;
}) {
  return (
    <article className="recommendation-card">
      <span className="recommendation-card__icon" aria-hidden="true">
        <Icon size={32} strokeWidth={2.2} />
      </span>
      <span>
        <strong>{label}</strong>
        <small>{detail}</small>
      </span>
      <button type="button" onClick={() => void runDesktopAction(actionIntent)}>
        {action}
      </button>
    </article>
  );
}

function DashboardTweakMatrix({ groups }: { groups: PlanGroup[] }) {
  const actionableTweaks = groups
    .filter((group) => group.id !== "blocked")
    .reduce((count, group) => count + group.tweaks.length, 0);

  return (
    <section className="dashboard-tweak-matrix" aria-label="Dashboard active tweak matrix">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Active tweak matrix</p>
          <h2>All available tweaks on the dashboard</h2>
        </div>
        <button
          className="button button--secondary"
          type="button"
          onClick={() =>
            void runDesktopAction({
              command: "review-plan",
              feedback: `${actionableTweaks} actionable changes are ready; blocked items stay protected.`,
              id: "dashboard-open-all-boost-changes",
              label: "Open all boost changes",
              targetRoute: "optimize"
            })
          }
        >
          <span>Open all boost changes</span>
        </button>
      </div>
      <PlanGroupGrid groups={groups} />
    </section>
  );
}

function RecentAction({ icon: Icon, label, meta }: { icon: LucideIcon; label: string; meta: string }) {
  return (
    <div className="recent-action">
      <Icon aria-hidden="true" size={25} strokeWidth={2.1} />
      <span>
        <strong>{label}</strong>
        <small>{meta}</small>
      </span>
      <CheckCircle2 aria-hidden="true" size={18} strokeWidth={2.5} />
    </div>
  );
}

function createInactiveResourceCards(tone: WorkflowTone): DashboardResourceMetric[] {
  const sparkline = normalizeSeries([]);

  return [
    {
      detail: "Live usage is paused",
      icon: Cpu,
      id: "cpu",
      label: "CPU",
      sparkline,
      tone,
      usage: 0,
      value: "Paused"
    },
    {
      detail: "Live memory view is paused",
      icon: MemoryStick,
      id: "ram",
      label: "Memory",
      sparkline,
      tone,
      usage: 0,
      value: "Paused"
    },
    {
      detail: "Live storage view is paused",
      icon: HardDrive,
      id: "disk",
      label: "Storage",
      sparkline,
      tone,
      usage: 0,
      value: "Paused"
    },
    {
      detail: "Live network view is paused",
      icon: Network,
      id: "network",
      label: "Network",
      sparkline,
      tone,
      usage: 0,
      value: "Paused"
    }
  ];
}

function createPreviewResourceCards(): DashboardResourceMetric[] {
  return [
    {
      detail: "AMD Ryzen 7 7800X3D",
      icon: Cpu,
      id: "cpu",
      label: "CPU",
      secondaryDetail: "16 threads - up to 5.0GHz",
      sparkline: [22, 31, 28, 39, 25, 27, 35, 24, 30, 26, 41, 23, 28, 24, 34, 27],
      tone: "success",
      usage: 28,
      value: "28%"
    },
    {
      detail: "8.3 GB used of 16 GB",
      icon: MemoryStick,
      id: "ram",
      label: "Memory",
      sparkline: [45, 49, 44, 46, 42, 42, 48, 43, 51, 45, 49, 43, 40, 44, 41, 46],
      tone: "benchmark",
      usage: 52,
      value: "52%"
    },
    {
      detail: "Main SSD activity now",
      icon: HardDrive,
      id: "disk",
      label: "Storage",
      secondaryDetail: "185 MB/s moving through storage",
      sparkline: [14, 18, 15, 24, 13, 16, 12, 27, 11, 17, 15, 19, 12, 13, 18, 20],
      tone: "warning",
      usage: 18,
      value: "18%"
    },
    {
      detail: "Gaming Ethernet link",
      icon: Network,
      id: "network",
      label: "Network",
      secondaryDetail: "124.6 Mbps now",
      sparkline: [19, 22, 11, 17, 16, 21, 14, 24, 13, 18, 29, 27, 22, 16, 14, 19],
      tone: "trust",
      usage: 16,
      value: "16%"
    }
  ];
}

function createDiskThroughputSeries(history: LiveResourceSnapshot[]) {
  return history.map((snapshot, index) => {
    const previous = index > 0 ? history[index - 1] : undefined;
    const bytesPerSecond = diskBytesPerSecondBetween(previous, snapshot);

    return bytesPerSecondPercent(bytesPerSecond, 250_000_000);
  });
}

function latestDiskBytesPerSecond(history: LiveResourceSnapshot[]) {
  const latest = history.length > 0 ? history[history.length - 1] : undefined;
  const previous = history.length > 1 ? history[history.length - 2] : undefined;

  return latest ? diskBytesPerSecondBetween(previous, latest) : null;
}

function diskBytesPerSecondBetween(
  previous: LiveResourceSnapshot | undefined,
  snapshot: LiveResourceSnapshot
) {
  if (snapshot.disk.bytesPerSecond !== null) {
    return snapshot.disk.bytesPerSecond;
  }

  if (!previous || previous.disk.totalBytes === null || snapshot.disk.totalBytes === null) {
    return null;
  }

  const elapsedSeconds =
    (new Date(snapshot.collectedAtUtc).getTime() - new Date(previous.collectedAtUtc).getTime()) / 1000;

  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return null;
  }

  return Math.max(0, (snapshot.disk.totalBytes - previous.disk.totalBytes) / elapsedSeconds);
}

function createNetworkThroughputSeries(history: LiveResourceSnapshot[]) {
  return history.map((snapshot, index) => {
    const previous = index > 0 ? history[index - 1] : undefined;
    const bytesPerSecond = networkBytesPerSecondBetween(previous, snapshot);

    return networkThroughputPercent(snapshot, bytesPerSecond);
  });
}

function latestNetworkBytesPerSecond(history: LiveResourceSnapshot[]) {
  const latest = history.length > 0 ? history[history.length - 1] : undefined;
  const previous = history.length > 1 ? history[history.length - 2] : undefined;

  return latest ? networkBytesPerSecondBetween(previous, latest) : null;
}

function networkBytesPerSecondBetween(
  previous: LiveResourceSnapshot | undefined,
  snapshot: LiveResourceSnapshot
) {
  if (snapshot.network.bytesPerSecond !== null) {
    return snapshot.network.bytesPerSecond;
  }

  if (!previous || previous.network.totalBytes === null || snapshot.network.totalBytes === null) {
    return null;
  }

  const elapsedSeconds =
    (new Date(snapshot.collectedAtUtc).getTime() - new Date(previous.collectedAtUtc).getTime()) / 1000;

  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return null;
  }

  return Math.max(0, (snapshot.network.totalBytes - previous.network.totalBytes) / elapsedSeconds);
}

function networkThroughputPercent(snapshot: LiveResourceSnapshot, bytesPerSecond: number | null) {
  const linkBytesPerSecond =
    snapshot.network.linkSpeedBitsPerSecond && snapshot.network.linkSpeedBitsPerSecond > 0
      ? snapshot.network.linkSpeedBitsPerSecond / 8
      : null;

  return bytesPerSecondPercent(bytesPerSecond, linkBytesPerSecond ?? 125_000_000);
}

function bytesPerSecondPercent(bytesPerSecond: number | null, ceilingBytesPerSecond: number) {
  if (bytesPerSecond === null || ceilingBytesPerSecond <= 0) {
    return 0;
  }

  return clampPercent((bytesPerSecond / ceilingBytesPerSecond) * 100);
}

function clampPercent(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return 0;
  }

  return Math.max(0, Math.min(100, value));
}

function resolveUsedPercent(
  percent: number | null | undefined,
  usedBytes: number | null | undefined,
  totalBytes: number | null | undefined
) {
  if (typeof percent === "number" && !Number.isNaN(percent)) {
    return clampPercent(percent);
  }

  if (
    typeof usedBytes !== "number" ||
    Number.isNaN(usedBytes) ||
    typeof totalBytes !== "number" ||
    Number.isNaN(totalBytes) ||
    totalBytes <= 0
  ) {
    return null;
  }

  return clampPercent((usedBytes / totalBytes) * 100);
}

function formatPercent(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "N/A";
  }

  if (value > 0 && value < 1) {
    return "<1%";
  }

  return value < 10 ? `${value.toFixed(1)}%` : `${Math.round(value)}%`;
}

function formatSeconds(milliseconds: number) {
  return `${Math.max(1, Math.round(milliseconds / 1000))}s`;
}

function formatClock(valueMhz: number) {
  if (valueMhz >= 1000) {
    return `${(valueMhz / 1000).toFixed(2)} GHz`;
  }

  return `${Math.round(valueMhz)} MHz`;
}

function formatCompactClock(valueMhz: number) {
  if (valueMhz >= 1000) {
    return `${(valueMhz / 1000).toFixed(2)}GHz`;
  }

  return `${Math.round(valueMhz)}MHz`;
}

function formatBytes(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "Unknown";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  let scaled = value;
  let unitIndex = 0;

  while (scaled >= 1024 && unitIndex < units.length - 1) {
    scaled /= 1024;
    unitIndex += 1;
  }

  const decimals = scaled >= 10 || unitIndex === 0 ? 0 : 1;

  return `${scaled.toFixed(decimals)} ${units[unitIndex] ?? "B"}`;
}

function formatBytesPerSecond(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "0 B/s";
  }

  return `${formatBytes(value)}/s`;
}

function formatBitsPerSecond(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "Unknown";
  }

  const units = ["bps", "Kbps", "Mbps", "Gbps"];
  let scaled = value;
  let unitIndex = 0;

  while (scaled >= 1000 && unitIndex < units.length - 1) {
    scaled /= 1000;
    unitIndex += 1;
  }

  const decimals = scaled >= 10 || unitIndex === 0 ? 0 : 1;

  return `${scaled.toFixed(decimals)} ${units[unitIndex] ?? "bps"}`;
}

function formatMemoryDetail(usedBytes: number | null, totalBytes: number | null) {
  if (usedBytes === null || totalBytes === null) {
    return "Memory usage now";
  }

  return `${formatBytes(usedBytes)} used of ${formatBytes(totalBytes)}`;
}

function formatProcessorName(value: string | null | undefined) {
  if (!value) {
    return null;
  }

  const cleaned = value
    .replace(/\(R\)|\(TM\)/gi, "")
    .replace(/\s+CPU\s*@\s*[\d.]+\s*GHz/gi, "")
    .replace(/\s+\d+\s*-?\s*Core Processor/gi, "")
    .replace(/\s+Processor$/i, "")
    .replace(/\s{2,}/g, " ")
    .trim();

  return cleaned || null;
}

function formatSnapshotTime(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "just now";
  }

  return date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit"
  });
}

function telemetryErrorMessage(value: string) {
  return value.length > 140 ? `${value.slice(0, 137)}...` : value;
}

function toChartPolyline(points: number[]) {
  return points
    .map((point, index) => {
      const x = 44 + (index / Math.max(points.length - 1, 1)) * 552;
      const y = 168 - Math.max(0, Math.min(100, point)) * 1.44;

      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function DashboardPriorityBoard({
  data,
  optimizeData,
  rollbackData,
  scanData
}: {
  data: DashboardData;
  optimizeData: OptimizeData | undefined;
  rollbackData: RollbackData | undefined;
  scanData: ScanData | undefined;
}) {
  const nextAction = scanData
    ? getScanNextAction(scanData)
    : {
        badge: tOptimizer("labels.noMutation"),
        detail: "Run the read-only inventory before generating a plan or applying changes.",
        eyebrow: "Read-only scan required",
        label: tOptimizer("actions.startScan"),
        tone: "active" as WorkflowTone,
        value: tOptimizer("labels.ready")
      };
  const activeStep = optimizeData?.applySteps.find((step) => step.state === "active");
  const safeGroup = optimizeData?.groups.find((group) => group.id === "safe");
  const rollbackCount = rollbackData?.sessions.length ?? 0;
  const bottleneck =
    data.readinessSignals.find((signal) => signal.tone === "danger" || signal.tone === "warning") ??
    data.readinessSignals[0];
  const scoreStyle = {
    "--score-angle": `${Math.max(0, Math.min(100, data.readinessScore)) * 3.6}deg`
  } as CSSProperties;

  const runwayItems = [
    {
      detail: scanData?.progress.current ?? "Inventory has not started; all collection is read-only.",
      id: "scan",
      label: tOptimizer(optimizerGlossaryKeys.scan),
      tone: "active" as WorkflowTone,
      value: scanData ? `${scanData.progress.percent}%` : tOptimizer("labels.ready")
    },
    {
      detail: activeStep?.detail ?? `${safeGroup?.tweaks.length ?? 0} safe changes can be reviewed first.`,
      id: "apply",
      label: "Apply state",
      tone: activeStep ? stateToTone(activeStep.state) : "success",
      value: activeStep?.label ?? "Safe review"
    },
    {
      detail: data.rollbackAvailability,
      id: "rollback",
      label: tOptimizer(optimizerGlossaryKeys.rollback),
      tone: "warning" as WorkflowTone,
      value: `${rollbackCount} sessions`
    },
    {
      detail: "Benchmark metadata required before calling the result final.",
      id: "benchmark",
      label: tOptimizer(optimizerGlossaryKeys.benchmark),
      tone: "benchmark" as const,
      value: data.lastBenchmarkDelta
    }
  ];

  return (
    <section className="dashboard-priority-board" data-tone={nextAction.tone} aria-label="Dashboard command cockpit">
      <div className="dashboard-priority-board__cell dashboard-priority-board__score">
        <div
          className="dashboard-priority-board__gauge"
          style={scoreStyle}
          aria-label={`Readiness ${data.readinessScore} of 100`}
        >
          <strong>{data.readinessScore}</strong>
          <span>100</span>
        </div>
        <div>
          <p className="eyebrow">Operational readiness</p>
          <h2>{data.activeMode} mode armed</h2>
          <small>{data.activePowerPlan} power, {data.driverState}, {data.pubgReadiness}</small>
        </div>
      </div>

      <div className="dashboard-priority-board__cell">
        <div className="dashboard-priority-board__section-heading">
          <span>{nextAction.eyebrow}</span>
          <strong>{nextAction.value}</strong>
        </div>
        <div className="dashboard-runway" aria-label="Scan apply rollback benchmark runway">
          {runwayItems.map((item) => (
            <div className="dashboard-runway__item" data-tone={item.tone} key={item.id}>
              <span aria-hidden="true" />
              <div>
                <strong>{item.label}</strong>
                <small>{item.detail}</small>
              </div>
              <b>{item.value}</b>
            </div>
          ))}
        </div>
      </div>

      <div className="dashboard-priority-board__cell dashboard-priority-board__action">
        <p className="eyebrow">{tOptimizer("commandHeader.nextAction")}</p>
        <h2>{nextAction.label}</h2>
        <p>{nextAction.detail}</p>
        <div className="dashboard-priority-board__buttons">
          <button
            className="button button--primary"
            type="button"
            onClick={() =>
              void runDesktopAction({
                command: "run-read-only-scan",
                feedback: "Opening the read-only scan from the dashboard priority board.",
                id: "dashboard-priority-start-scan",
                label: tOptimizer("actions.startScan"),
                targetRoute: "scan"
              })
            }
          >
            <span>{tOptimizer("actions.startScan")}</span>
          </button>
          <button
            className="button button--secondary"
            type="button"
            onClick={() =>
              void runDesktopAction({
                command: "review-plan",
                feedback: "Opening the safety-gated tweak plan from the dashboard priority board.",
                id: "dashboard-priority-review-plan",
                label: tOptimizer("actions.reviewPlan"),
                targetRoute: "optimize"
              })
            }
          >
            <span>{tOptimizer("actions.reviewPlan")}</span>
          </button>
        </div>
        {bottleneck ? (
          <StatusRow
            label={bottleneck.label}
            value={bottleneck.value}
            detail={bottleneck.detail}
            tone={bottleneck.tone}
          />
        ) : null}
      </div>
    </section>
  );
}

function DashboardOptimizerLanes({
  data,
  optimizeData,
  rollbackData,
  scanData
}: {
  data: DashboardData;
  optimizeData: OptimizeData | undefined;
  rollbackData: RollbackData | undefined;
  scanData: ScanData | undefined;
}) {
  const lanes = createDashboardOptimizerLanes(data, optimizeData, rollbackData, scanData);

  return (
    <section className="optimizer-lane-section" aria-label="Dashboard optimizer category lanes">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Category lanes</p>
          <h2>Optimizer cockpit</h2>
        </div>
        <span className="pill pill--active">{lanes.length} lanes</span>
      </div>
      <div className="optimizer-lane-grid">
        {lanes.map((lane) => (
          <OptimizerLaneCard key={lane.id} lane={lane} />
        ))}
      </div>
    </section>
  );
}

function OptimizerLaneCard({ lane }: { lane: OptimizerLane }) {
  return (
    <article className="optimizer-lane" data-lane={lane.id} data-tone={lane.tone}>
      <div className="optimizer-lane__header">
        <span>{lane.eyebrow}</span>
        <strong>{lane.label}</strong>
      </div>
      <p>{lane.summary}</p>
      <div className="optimizer-lane__meta" aria-label={`${lane.label} status`}>
        <span>{lane.status}</span>
        <span>{lane.trustSignal}</span>
      </div>
      {lane.details && lane.details.length > 0 ? <DefinitionGrid items={lane.details} /> : null}
      <div className="optimizer-lane__actions">
        <ActionButton action={lane.primaryAction} className="optimizer-lane__button" />
        {lane.detailAction ? <ActionButton action={lane.detailAction} className="optimizer-lane__button" /> : null}
      </div>
    </article>
  );
}

function DashboardBottleneckPanel({ data }: { data: DashboardData }) {
  const bottleneck =
    data.readinessSignals.find((signal) => signal.tone === "danger" || signal.tone === "warning") ??
    data.readinessSignals[0];

  return (
    <Surface title="Current bottleneck" eyebrow="Highest attention signal" badge={`${data.readinessScore}/100`}>
      {bottleneck ? (
        <StatusRow
          label={bottleneck.label}
          value={bottleneck.value}
          detail={bottleneck.detail}
          tone={bottleneck.tone}
        />
      ) : null}
      <DefinitionGrid
        items={[
          ["Active mode", data.activeMode],
          ["Power plan", data.activePowerPlan],
          ["Benchmark delta", data.lastBenchmarkDelta],
          ["Rollback", data.rollbackAvailability],
          ["Trust", data.trustState]
        ]}
      />
    </Surface>
  );
}

function DashboardNoScanPanel({ data }: { data: DashboardData }) {
  return (
    <Surface title="Next action" eyebrow="Read-only scan required" badge={tOptimizer("actions.startScan")}>
      <StatusRow
        label={tOptimizer("actions.startScan")}
        value={tOptimizer("labels.ready")}
        detail="Checks hardware, Windows, GPU, game, and recovery context before Smart Boost can apply changes."
        tone="active"
      />
      <DefinitionGrid
        items={[
          ["Write boundary", tOptimizer("labels.noMutation")],
          ["Rollback policy", data.rollbackAvailability],
          ["Trust state", data.trustState]
        ]}
      />
    </Surface>
  );
}

function DashboardSnapshot({ data }: { data: DashboardData }) {
  const snapshotItems = [
    {
      id: "pubg",
      label: "PUBG",
      value: data.pubgReadiness,
      detail: "Game profile and anti-cheat boundary",
      tone: "success" as WorkflowTone
    },
    {
      id: "gpu",
      label: "GPU",
      value: data.driverState,
      detail: "Driver and display profile",
      tone: "active" as WorkflowTone
    },
    {
      id: "benchmark",
      label: "Benchmark",
      value: data.lastBenchmarkDelta,
      detail: "Measured 1% low change",
      tone: "active" as WorkflowTone
    },
    {
      id: "rollback",
      label: "Rollback",
      value: data.rollbackAvailability,
      detail: "Restore path ready",
      tone: "warning" as WorkflowTone
    },
    {
      id: "trust",
      label: "Trust",
      value: data.trustState,
      detail: "Signed release channel",
      tone: "success" as WorkflowTone
    }
  ];

  return (
    <section className="dashboard-snapshot" aria-label="System readiness snapshot">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Snapshot</p>
          <h2>System readiness</h2>
        </div>
      </div>
      <div className="dashboard-snapshot__grid">
        {snapshotItems.map((item) => (
          <article className="snapshot-card" data-tone={item.tone} key={item.id}>
            <span>{item.label}</span>
            <strong>{item.value}</strong>
            <small>{item.detail}</small>
          </article>
        ))}
      </div>
    </section>
  );
}

export function ScanWorkflowView({ data, optimizeData }: ScanViewProps) {
  const scanRun = useSmartScanRun(data);
  const activeData = scanRun.data;
  const selectedScopes = activeData.scopes.filter((scope) => scope.checked).length;
  const nextAction = getScanNextAction(activeData);
  const blockedFindings = activeData.findings.filter((finding) => finding.tone === "danger").length;
  const scanComplete = activeData.progress.percent >= 100;
  const scanStyle = {
    "--scan-progress": `${activeData.progress.percent}%`,
    "--scan-angle": `${activeData.progress.percent * 3.6}deg`
  } as CSSProperties;

  return (
    <div className="smart-scan-view" aria-label="Scan workflow">
      <WorkflowHeader
        eyebrow="Smart Scan"
        title="Safe PC checkup"
        actions={
          <SmartScanActionBar
            complete={scanComplete}
            onCancel={scanRun.cancel}
            onGeneratePlan={scanRun.generatePlan}
            onStart={scanRun.start}
            running={scanRun.running}
          />
        }
      />
      <section className="smart-scan-summary" data-tone={nextAction.tone} aria-label="Smart Scan execution summary">
        <div className="smart-scan-summary__meter" style={scanStyle}>
          <div
            className="smart-scan-ring"
            aria-label={`${activeData.progress.current}: ${activeData.progress.percent}% complete`}
            aria-valuemax={100}
            aria-valuemin={0}
            aria-valuenow={activeData.progress.percent}
            role="progressbar"
          >
            <strong>{activeData.progress.percent}%</strong>
            <span>{activeData.progress.label}</span>
          </div>
        </div>
        <div className="smart-scan-summary__copy">
          <p className="eyebrow">{nextAction.eyebrow}</p>
          <h2>{nextAction.label}</h2>
          <p>{nextAction.detail}</p>
          <DefinitionGrid
            items={[
              ["Selected checks", `${selectedScopes}/${activeData.scopes.length}`],
              ["Finished checks", activeData.progress.completed.length.toString()],
              ["Recommendations", activeData.findings.length.toString()],
              ["Blocked for safety", blockedFindings > 0 ? `${blockedFindings} item` : "No"]
            ]}
          />
        </div>
        <div className="smart-scan-summary__guardrails">
          <StatusRow
            label="Protection"
            value="Scan only"
            detail="Smart Scan checks your PC and prepares recommendations. Changes happen only in Smart Boost."
            tone="success"
          />
          <StatusRow
            label="Smart Boost"
            value={scanComplete ? "Ready" : "Waiting"}
            detail={
              scanComplete
                ? "Safe, Competitive, Lab, and Blocked recommendations are ready to review."
                : "Smart Boost unlocks after the selected checks finish."
            }
            tone={scanComplete ? "success" : "warning"}
          />
        </div>
      </section>
      <div className="smart-scan-layout">
        <Surface title="Choose checks" eyebrow="Smart Scan scope">
          <ScanScopeGrid
            disabled={scanRun.running}
            onToggle={scanRun.toggleScope}
            scopes={activeData.scopes}
          />
        </Surface>
        <Surface title={activeData.progress.label} eyebrow={`${activeData.progress.percent}% complete`}>
          <ProgressBar percent={activeData.progress.percent} label={activeData.progress.current} />
          <CompletedChecks checks={activeData.progress.completed} />
          <FlowList items={activeData.states} />
        </Surface>
      </div>
      <ScanNextActionPanel data={activeData} />
      {optimizeData ? (
        <Surface title="Smart Boost preview" eyebrow="What unlocks after scan">
          <ScanPlanBridge groups={optimizeData.groups} scanComplete={scanComplete} />
        </Surface>
      ) : null}
      <Surface title="Recommendations" eyebrow="Grouped by impact and risk">
        <FindingGrid findings={activeData.findings} />
      </Surface>
    </div>
  );
}

export function OptimizeWorkflowView({ data }: OptimizeViewProps) {
  return (
    <div className="smart-boost-view" style={viewGridStyle} aria-label="Smart Boost workflow">
      <WorkflowHeader
        eyebrow="Smart Boost"
        title="Safe Boost control"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <SmartBoostOverview data={data} />
      <OptimizerCategoryLaneDeck groups={data.groups} />
      <ApplyProgressBoard groups={data.groups} steps={data.applySteps} />
      <Surface title={tOptimizer("modes.optimizationMode")} eyebrow="Safe defaults stay selected">
        <ModeSegmentedControl
          label={tOptimizer("modes.optimizationMode")}
          options={modeSegmentsFromGroups(data.groups)}
          value="safe"
        />
      </Surface>
      <div style={twoColumnStyle}>
        <Surface title="Boost sequence" eyebrow="Backup first">
          <ApplyTimeline label="Smart Boost safety timeline" steps={toTimelineSteps(data.applySteps)} />
        </Surface>
        <Surface title="Protection checks" eyebrow="Apply, verify, benchmark, recovery">
          <ApplyStateVisibility steps={data.applySteps} groups={data.groups} />
        </Surface>
        <Surface title="Before and after" eyebrow="Restore point beside planned impact">
          <DiffPanel items={createPlanDiffPreview(data.groups)} label="Smart Boost before and after preview" />
        </Surface>
      </div>
      <Surface title="Safety rules" eyebrow="Consent and apply boundaries">
        <PlanPolicyLegend groups={data.groups} />
      </Surface>
      <PlanGroupGrid groups={data.groups} />
    </div>
  );
}

function SmartBoostOverview({ data }: { data: OptimizeData }) {
  const safeGroup = data.groups.find((group) => group.id === "safe");
  const competitiveGroup = data.groups.find((group) => group.id === "competitive");
  const labGroup = data.groups.find((group) => group.id === "lab");
  const blockedGroup = data.groups.find((group) => group.id === "blocked");
  const activeStep = data.applySteps.find((step) => step.state === "active");
  const safeCount = safeGroup?.tweaks.length ?? 0;
  const reviewCount = (competitiveGroup?.tweaks.length ?? 0) + (labGroup?.tweaks.length ?? 0);
  const blockedCount = blockedGroup?.tweaks.length ?? 0;

  return (
    <section className="smart-boost-overview" data-tone="success" aria-label="Smart Boost ready summary">
      <div className="smart-boost-overview__copy">
        <p className="eyebrow">Ready now</p>
        <h2>Safe Boost is ready to apply.</h2>
        <p>
          {safeCount} reversible changes can run with backup and verification. Competitive and Lab
          recommendations stay in review until you choose them.
        </p>
        <DefinitionGrid
          items={[
            ["Safe Boost", `${safeCount} changes`],
            ["Review queue", `${reviewCount} optional`],
            ["Recovery", "Restore point prepared"],
            ["Blocked", `${blockedCount} protected`]
          ]}
        />
      </div>
      <div className="smart-boost-overview__status">
        <StatusRow
          label="Current step"
          value={activeStep?.label ?? "Ready"}
          detail={activeStep?.detail ?? "Safe Boost is waiting for your confirmation."}
          tone={activeStep ? stateToTone(activeStep.state) : "success"}
        />
        <StatusRow
          label="Apply policy"
          value="Safe first"
          detail="Only reversible Safe Boost changes are selected by default."
          tone="success"
        />
        <StatusRow
          label="More performance"
          value="Review only"
          detail="Competitive and Lab lanes stay optional with their tradeoffs visible."
          tone="warning"
        />
      </div>
    </section>
  );
}

export function RollbackWorkflowView({ data }: RollbackViewProps) {
  const gpuProfileSessions = data.sessions.filter(hasGpuProfileRollback);

  return (
    <div style={viewGridStyle} aria-label="Rollback workflow">
      <WorkflowHeader
        eyebrow="Rollback"
        title="Session recovery timeline"
        actions={<RollbackActionBar sessions={data.sessions} />}
      />
      <div style={twoColumnStyle}>
        <Surface title="Restore queue" eyebrow="Session-level recovery">
          <DefinitionGrid
            items={[
              ["Sessions ready", String(data.sessions.length)],
              ["GPU profile backups", String(gpuProfileSessions.length)],
              ["Restore mode", "All changed values in a session"],
              ["Verification", "Readback before completion"]
            ]}
          />
        </Surface>
        <Surface title="GPU profile rollback" eyebrow="NVIDIA export restore">
          <GpuRollbackFlow sessions={gpuProfileSessions} />
        </Surface>
      </div>
      <div style={viewGridStyle}>
        {data.sessions.map((session) => (
          <RollbackSessionLog key={session.id} session={createRollbackSessionLog(session)} />
        ))}
      </div>
    </div>
  );
}

export function PowerWorkflowView({ data }: PowerViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Power plan workflow">
      <WorkflowHeader
        eyebrow="Power"
        title="Liiiraa power plan control"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <MetricGrid metrics={data.metrics} />
      <div style={twoColumnStyle}>
        <Surface title="Plan ladder" eyebrow="Scoped Windows power changes">
          <PowerPlanTable plans={data.plans} />
        </Surface>
        <Surface title="Desktop and laptop policy" eyebrow="Default safety model">
          <SignalList items={data.rules} />
        </Surface>
      </div>
    </div>
  );
}

export function NvidiaWorkflowView({ data }: NvidiaViewProps) {
  return (
    <div style={viewGridStyle} aria-label="NVIDIA profile workflow">
      <WorkflowHeader
        eyebrow="NVIDIA"
        title="Profile safety and PUBG readiness"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <MetricGrid metrics={data.metrics} />
      <div style={twoColumnStyle}>
        <Surface title="Profile states" eyebrow="Backup before changes">
          <NvidiaProfileTable profiles={data.profiles} />
        </Surface>
        <Surface title="Refresh and cap logic" eyebrow="VRR-aware profile policy">
          <DefinitionGrid items={data.capLogic} />
        </Surface>
      </div>
      <Surface title="Safety policy" eyebrow="Driver settings guardrails">
        <SignalList items={data.policies} />
      </Surface>
    </div>
  );
}

export function PubgWorkflowView({ data }: PubgViewProps) {
  return (
    <div style={viewGridStyle} aria-label="PUBG optimization workflow">
      <WorkflowHeader
        eyebrow="PUBG"
        title="Competitive checklist and anti-cheat boundary"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <PubgFocusRail data={data} />
      <MetricGrid metrics={data.metrics} />
      <div style={twoColumnStyle}>
        <Surface title="Detection" eyebrow="Read-only install and config state">
          <SignalList items={data.detections} />
        </Surface>
        <Surface title="Competitive checklist" eyebrow="Automatic vs manual recommendations">
          <SignalList items={data.checklist} />
        </Surface>
      </div>
      <Surface title="Launch option cleanup" eyebrow="Remove legacy flags, keep backup">
        <PubgLaunchOptionsTable options={data.launchOptions} />
      </Surface>
      <Surface title="DirectX benchmark choice" eyebrow="No universal forced mode">
        <PubgDxTable choices={data.dxChoices} />
      </Surface>
      <div style={twoColumnStyle}>
        <Surface title="DX benchmark flow" eyebrow="Config snapshot before recommendation">
          <PubgDxBenchmarkFlow benchmark={data.dxBenchmark} />
        </Surface>
        <Surface title="Measured rationale" eyebrow="Variance-aware recommendation">
          <PubgDxBenchmarkResults benchmark={data.dxBenchmark} />
        </Surface>
      </div>
    </div>
  );
}

function PubgFocusRail({ data }: PubgViewProps) {
  const install = data.metrics.find((metric) => metric.id === "install");
  const config = data.metrics.find((metric) => metric.id === "config");
  const battleye = data.metrics.find((metric) => metric.id === "battleye");
  const profile = data.metrics.find((metric) => /profile/i.test(metric.id));
  const benchmarkStep = data.dxBenchmark.steps.find((step) => step.state === "active") ?? data.dxBenchmark.steps[0];

  const focusItems = [
    {
      action: createLaneAction("pubg-optimize", "Optimize game", "secondary"),
      detail: install?.detail ?? "Supported game path is checked before any profile action.",
      id: "supported-game",
      label: "Supported game",
      tone: install?.tone ?? "success",
      value: install?.value ?? "Detected"
    },
    {
      action: createLaneAction("pubg-profile", tOptimizer("actions.openNvidiaProfile"), "secondary"),
      detail: profile?.detail ?? "PUBG profile stays linked to the NVIDIA route and backup state.",
      id: "profile-status",
      label: "Profile status",
      tone: profile?.tone ?? "warning",
      value: profile?.value ?? "Review"
    },
    {
      action: createLaneAction("pubg-config", tOptimizer("actions.snapshotConfig"), "secondary"),
      detail: config?.detail ?? "Configuration is snapped before launch cleanup or renderer tests.",
      id: "launch-config",
      label: "Launch/config state",
      tone: config?.tone ?? "active",
      value: config?.value ?? "Snapshot"
    },
    {
      action: createLaneAction("pubg-benchmark", tOptimizer("actions.startDxBenchmark"), "secondary"),
      detail: benchmarkStep?.detail ?? data.dxBenchmark.rationale,
      id: "benchmark",
      label: "Benchmark prompt",
      tone: benchmarkStep?.tone ?? "benchmark",
      value: benchmarkStep?.label ?? "Ready"
    },
    {
      action: createLaneAction("pubg-boundary", "Review boundary", "locked", true),
      detail: battleye?.detail ?? "No file, memory, or anti-cheat tamper action is exposed.",
      id: "anti-cheat",
      label: "Anti-cheat boundary",
      tone: battleye?.tone ?? "trust",
      value: battleye?.value ?? "Protected"
    }
  ];

  return (
    <section className="game-focus-grid" aria-label="PUBG supported game and profile state">
      {focusItems.map((item) => (
        <article className="game-focus-card" data-tone={item.tone} key={item.id}>
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <small>{item.detail}</small>
          <ActionButton action={item.action} className="game-focus-card__button" />
        </article>
      ))}
    </section>
  );
}

export function BenchmarkWorkflowView({ data }: BenchmarkViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Benchmark comparison workflow">
      <WorkflowHeader
        eyebrow="Benchmarks"
        title="Before and after proof"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <MetricGrid metrics={data.metrics} />
      <Surface title="Comparison score" eyebrow={data.summary.detail} badge={data.summary.confidence}>
        <DefinitionGrid
          items={[
            ["Score", data.summary.score],
            ["Decision", data.summary.decision],
            ["Variance", data.summary.varianceBand]
          ]}
        />
        <SignalList items={data.summary.warnings} />
      </Surface>
      <div style={twoColumnStyle}>
        <Surface title="Average and low comparison" eyebrow="Native frames only">
          <BenchmarkProofChart label={tOptimizer("primitives.benchmarkProofAria")} points={data.chart} />
        </Surface>
        <Surface title="Run metadata" eyebrow="Required comparison context">
          <DefinitionGrid items={data.metadata} />
        </Surface>
      </div>
      <Surface title="Capture state" eyebrow="Variance-aware reporting">
        <SignalList items={data.sessions} />
      </Surface>
    </div>
  );
}

function DashboardNextActionPanel({
  optimizeData,
  rollbackData,
  scanData
}: {
  optimizeData: OptimizeData | undefined;
  rollbackData: RollbackData | undefined;
  scanData: ScanData;
}) {
  const selectedScopes = scanData.scopes.filter((scope) => scope.checked).length;
  const nextAction = getScanNextAction(scanData);
  const safeGroup = optimizeData?.groups.find((group) => group.id === "safe");
  const visibleFindings = scanData.findings.slice(0, 3);

  return (
    <Surface title="Next action" eyebrow={nextAction.eyebrow} badge={nextAction.badge}>
      <div style={twoColumnStyle}>
        <div style={compactRowStyle}>
          <ProgressBar percent={scanData.progress.percent} label={scanData.progress.current} />
          <DefinitionGrid
            items={[
              ["Scan scope", `${selectedScopes}/${scanData.scopes.length} selected`],
              ["Completed checks", scanData.progress.completed.length.toString()],
              ["Visible findings", scanData.findings.length.toString()],
              ["Safe changes", safeGroup ? safeGroup.tweaks.length.toString() : "Not generated"],
              ["Rollback sessions", rollbackData ? rollbackData.sessions.length.toString() : "Not loaded"]
            ]}
          />
        </div>
        <div style={compactRowStyle}>
          <StatusRow
            label={nextAction.label}
            value={nextAction.value}
            detail={nextAction.detail}
            tone={nextAction.tone}
          />
          {visibleFindings.map((finding) => (
            <StatusRow
              key={finding.id}
              label={finding.title}
              value={`${finding.group}, ${finding.risk} risk`}
              detail={finding.detail}
              tone={finding.tone}
            />
          ))}
        </div>
      </div>
    </Surface>
  );
}

function SmartScanActionBar({
  complete,
  onCancel,
  onGeneratePlan,
  onStart,
  running
}: {
  complete: boolean;
  onCancel: () => void;
  onGeneratePlan: () => void;
  onStart: () => void;
  running: boolean;
}) {
  const startTooltip = running
    ? "Smart Scan is already checking the selected areas."
    : complete
      ? "Run Smart Scan again before refreshing Smart Boost."
      : tOptimizer("tooltips.startScan");
  const startLabel = running ? "Scanning..." : tOptimizer("actions.startScan");
  const actionHint = running
    ? "Checking selected areas. No changes are being applied."
    : complete
      ? "Scan complete. Smart Boost is ready to open."
      : "Smart Scan only checks your PC. Boost actions stay locked until it finishes.";

  return (
    <div className="action-bar action-bar--pattern smart-scan-actions" aria-label="Smart Scan actions">
      <button
        aria-busy={running}
        aria-describedby="smart-scan-start-tooltip"
        className="button action-bar__button button--primary"
        data-state={running ? "loading" : complete ? "success" : "idle"}
        disabled={running}
        onClick={onStart}
        type="button"
      >
        <Search aria-hidden="true" size={16} strokeWidth={2.2} />
        <span>{startLabel}</span>
        <span className="primitive-tooltip" id="smart-scan-start-tooltip" role="tooltip">
          {startTooltip}
        </span>
      </button>
      <button
        aria-describedby="smart-scan-cancel-tooltip"
        className="button action-bar__button button--ghost"
        disabled={!running}
        onClick={onCancel}
        type="button"
      >
        <CircleStop aria-hidden="true" size={16} strokeWidth={2.2} />
        <span>{tOptimizer("actions.cancelScan")}</span>
        <span className="primitive-tooltip" id="smart-scan-cancel-tooltip" role="tooltip">
          {tOptimizer("tooltips.cancelScan")}
        </span>
      </button>
      <button
        aria-describedby="smart-scan-plan-tooltip"
        className="button action-bar__button button--secondary"
        disabled={!complete}
        onClick={onGeneratePlan}
        type="button"
      >
        <Rocket aria-hidden="true" size={16} strokeWidth={2.2} />
        <span>{tOptimizer("actions.generatePlan")}</span>
        <span className="primitive-tooltip" id="smart-scan-plan-tooltip" role="tooltip">
          {complete ? "Open Smart Boost from completed recommendations." : "Finish Smart Scan before opening Smart Boost."}
        </span>
      </button>
      <p className="smart-scan-actions__hint" aria-live="polite">
        {actionHint}
      </p>
    </div>
  );
}

function ScanScopeGrid({
  disabled,
  onToggle,
  scopes
}: {
  disabled: boolean;
  onToggle: (scopeId: string) => void;
  scopes: ScanScope[];
}) {
  return (
    <div className="scan-scope-grid" aria-label="Smart Scan selected checks">
      {scopes.map((scope) => (
        <label className="workflow-check scan-scope-card" data-selected={scope.checked} key={scope.id}>
          <input
            checked={scope.checked}
            disabled={disabled}
            onChange={() => onToggle(scope.id)}
            type="checkbox"
          />
          <span>
            <strong>{scope.label}</strong>
            <small>{scope.detail}</small>
          </span>
        </label>
      ))}
    </div>
  );
}

function ScanPlanBridge({ groups, scanComplete }: { groups: PlanGroup[]; scanComplete: boolean }) {
  return (
    <div className="scan-plan-bridge" aria-label="Smart Boost tweak readiness after scan">
      {groups.map((group) => (
        <article className="scan-plan-card" data-tone={scanComplete ? group.tone : "locked"} key={group.id}>
          <span>{group.label}</span>
          <strong>{group.tweaks.length} changes</strong>
          <small>{scanComplete ? getPlanGroupApplyState(group) : "Finish Smart Scan to unlock"}</small>
          <b>{scanComplete ? getPlanGroupRiskLabel(group) : "Waiting for scan"}</b>
        </article>
      ))}
    </div>
  );
}

function ScanNextActionPanel({ data }: { data: ScanData }) {
  const selectedScopes = data.scopes.filter((scope) => scope.checked);
  const nextAction = getScanNextAction(data);
  const blockedFindings = data.findings.filter((finding) => finding.tone === "danger");

  return (
    <Surface title="Next action" eyebrow={nextAction.eyebrow} badge={nextAction.badge}>
      <div style={twoColumnStyle}>
        <div style={compactRowStyle}>
          <StatusRow
            label={nextAction.label}
            value={nextAction.value}
            detail={nextAction.detail}
            tone={nextAction.tone}
          />
          <StatusRow
            label="Protection"
            value="Scan only"
            detail="Smart Scan shows checks and recommendations only. Changes happen in Smart Boost."
            tone="success"
          />
          <StatusRow
            label="Smart Boost"
            value={data.progress.percent >= 100 ? "Ready" : "Waiting"}
            detail={
              data.progress.percent >= 100
                ? "Completed recommendations are ready for Safe Boost review."
                : "Smart Boost stays locked until the selected checks finish."
            }
            tone={data.progress.percent >= 100 ? "success" : "warning"}
          />
        </div>
        <DefinitionGrid
          items={[
            ["Selected checks", `${selectedScopes.length}/${data.scopes.length}`],
            ["Finished checks", data.progress.completed.join(", ")],
            ["Recommendations", `${data.findings.length} total`],
            ["Blocked for safety", blockedFindings.length > 0 ? blockedFindings.map((item) => item.title).join(", ") : "No"]
          ]}
        />
      </div>
    </Surface>
  );
}

function CompletedChecks({ checks }: { checks: string[] }) {
  if (checks.length === 0) {
    return (
      <StatusRow
        label="Completed checks"
        value="Waiting"
        detail="No Smart Scan checks have finished yet."
        tone="neutral"
      />
    );
  }

  return (
    <div style={completedChecksStyle} aria-label="Completed scan checks">
      {checks.map((check) => (
        <span className="pill" key={check}>
          {check}
        </span>
      ))}
    </div>
  );
}

function ApplyStateVisibility({
  groups,
  steps
}: {
  groups: PlanGroup[];
  steps: ApplyStep[];
}) {
  const activeStep = steps.find((step) => step.state === "active");
  const backupStep = findWorkflowStep(steps, "backup");
  const benchmarkStep = findWorkflowStep(steps, "benchmark");
  const rollbackStep = findWorkflowStep(steps, "rollback");
  const rebootTweaks = groups.flatMap((group) =>
    group.tweaks.filter((tweak) => !/^(no|n\/a)$/i.test(tweak.reboot))
  );
  const blockedTweaks = groups.find((group) => group.id === "blocked")?.tweaks.length ?? 0;

  return (
    <div style={compactRowStyle}>
      <StatusRow
        label="Current step"
        value={activeStep?.label ?? "Ready"}
        detail={activeStep?.detail ?? "No boost step is running."}
        tone={activeStep ? stateToTone(activeStep.state) : "neutral"}
      />
      <StatusRow
        label="Backup state"
        value={backupStep?.state ?? "Missing"}
        detail={backupStep?.detail ?? "Backup must be ready before changes are applied."}
        tone={backupStep ? stateToTone(backupStep.state) : "danger"}
      />
      <StatusRow
        label="Blocked changes"
        value="Protected"
        detail={`${blockedTweaks} blocked changes stay informational; failed changes stop before verification and keep recovery ready.`}
        tone={blockedTweaks > 0 ? "danger" : "success"}
      />
      <StatusRow
        label="Restart notes"
        value={`${rebootTweaks.length} marked`}
        detail={rebootTweaks.length > 0 ? rebootTweaks.map((tweak) => tweak.change).join(", ") : "No restart note is queued."}
        tone={rebootTweaks.length > 0 ? "warning" : "success"}
      />
      <StatusRow
        label="Benchmark prompt"
        value={benchmarkStep?.state ?? "Pending"}
        detail={benchmarkStep?.detail ?? "Benchmark prompt appears after verification."}
        tone={benchmarkStep ? stateToTone(benchmarkStep.state) : "warning"}
      />
      <StatusRow
        label="Recovery availability"
        value={rollbackStep?.state === "pending" ? "Armed after backup" : (rollbackStep?.state ?? "Ready")}
        detail={rollbackStep?.detail ?? "Restore actions stay attached to this boost session."}
        tone="warning"
      />
    </div>
  );
}

function ApplyProgressBoard({ groups, steps }: { groups: PlanGroup[]; steps: ApplyStep[] }) {
  const cards = createApplyProgressCards(groups, steps);

  return (
    <section className="optimizer-state-board" aria-label="Apply, backup, verification, benchmark, and rollback states">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Boost progress</p>
          <h2>Protected apply sequence</h2>
        </div>
        <span className="pill pill--active">{steps.length} stages</span>
      </div>
      <div className="optimizer-state-grid">
        {cards.map((card) => (
          <article className="optimizer-state-card" data-tone={card.tone} key={card.id}>
            <span>{card.label}</span>
            <strong>{card.value}</strong>
            <small>{card.detail}</small>
          </article>
        ))}
      </div>
    </section>
  );
}

function PlanPolicyLegend({ groups }: { groups: PlanGroup[] }) {
  return (
    <div style={policyGridStyle} aria-label={tOptimizer("workflow.plan.gatePolicyAria")}>
      {groups.map((group) => (
        <div style={policyItemStyle} data-tone={group.tone} key={group.id}>
          <span style={policyShapeStyle} aria-hidden="true" />
          <div>
            <strong>{group.label}</strong>
            <small>
              {getPlanGroupApplyState(group)}. {getPlanGroupConsent(group)}
            </small>
          </div>
        </div>
      ))}
    </div>
  );
}

function OptimizerCategoryLaneDeck({ groups }: { groups: PlanGroup[] }) {
  const lanes = createOptimizerCategoryLanes(groups);

  return (
    <section className="optimizer-lane-section" aria-label="Optimization category lanes">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Boost lanes</p>
          <h2>What Smart Boost will handle</h2>
        </div>
        <span className="pill pill--active">{lanes.length} areas</span>
      </div>
      <div className="optimizer-lane-grid optimizer-lane-grid--compact">
        {lanes.map((lane) => (
          <OptimizerLaneCard key={lane.id} lane={lane} />
        ))}
      </div>
    </section>
  );
}

const planActionLocaleKeys: Partial<Record<string, { label: OptimizerLocaleKey; tooltip: OptimizerLocaleKey }>> = {
  "apply-safe": { label: "actions.applySafeOnly", tooltip: "tooltips.applySafeOnly" },
  "include-competitive": { label: "actions.includeCompetitive", tooltip: "tooltips.includeCompetitive" },
  "inspect-lab": { label: "actions.inspectLab", tooltip: "tooltips.inspectLab" },
  "export-plan": { label: "actions.exportPlan", tooltip: "tooltips.exportPlan" },
  cancel: { label: "actions.cancel", tooltip: "tooltips.cancel" },
  "stage-balanced": { label: "actions.stageBalanced", tooltip: "tooltips.stageBalanced" },
  "review-competitive": { label: "actions.reviewCompetitive", tooltip: "tooltips.reviewCompetitive" },
  "export-power-plan": { label: "actions.exportPlan", tooltip: "tooltips.exportPlan" },
  "backup-profiles": { label: "actions.backupProfiles", tooltip: "tooltips.backupProfiles" },
  "stage-pubg-profile": { label: "actions.stagePubgProfile", tooltip: "tooltips.stagePubgProfile" },
  "open-benchmark": { label: "actions.openBenchmark", tooltip: "tooltips.openBenchmark" },
  "snapshot-config": { label: "actions.snapshotConfig", tooltip: "tooltips.snapshotConfig" },
  "start-dx-benchmark": { label: "actions.startDxBenchmark", tooltip: "tooltips.startDxBenchmark" },
  "open-nvidia-profile": { label: "actions.openNvidiaProfile", tooltip: "tooltips.openNvidiaProfile" },
  "capture-before": { label: "actions.captureBefore", tooltip: "tooltips.captureBefore" },
  "compare-after": { label: "actions.compareAfter", tooltip: "tooltips.compareAfter" },
  "export-benchmark": { label: "actions.exportReport", tooltip: "tooltips.exportReport" },
  "start-scan": { label: "actions.startScan", tooltip: "tooltips.startScan" },
  "cancel-scan": { label: "actions.cancelScan", tooltip: "tooltips.cancelScan" }
};

function toCoreAction(action: PlanAction): CoreAction {
  const actionCopy = planActionLocaleKeys[action.id];
  const label = actionCopy ? tOptimizer(actionCopy.label) : action.label;
  const tooltip = actionCopy ? tOptimizer(actionCopy.tooltip) : label;

  return {
    icon: actionIconForLabel(label),
    id: action.id,
    label,
    tooltip,
    variant: action.variant as CoreActionVariant
  };
}

export function PlanActionBar({ actions }: { actions: PlanAction[] }) {
  const coreActions = actions.map((action) => toCoreAction(action));

  return (
    <ActionCluster actions={coreActions} label={tOptimizer("workflow.actions.optimizationPlanAria")} />
  );
}

function RollbackActionBar({ sessions }: { sessions: RollbackSession[] }) {
  const hasGpuSession = sessions.some(hasGpuProfileRollback);
  const actions: CoreAction[] = [
    {
      icon: "rollback",
      id: "restore-all",
      label: tOptimizer("actions.restoreAll"),
      tooltip: tOptimizer("tooltips.restoreSelectedSession"),
      variant: "primary"
    },
    ...(hasGpuSession
      ? [
          {
            icon: "gpu" as const,
            id: "restore-gpu-profiles",
            label: tOptimizer("actions.restoreGpuProfiles"),
            tooltip: tOptimizer("tooltips.restoreNvidiaProfileBackup"),
            variant: "secondary" as const
          }
        ]
      : []),
    {
      icon: "file",
      id: "export-rollback-audit",
      label: tOptimizer("actions.exportAudit"),
      tooltip: tOptimizer("tooltips.exportRollbackAudit"),
      variant: "ghost"
    }
  ];

  return <ActionCluster actions={actions} label={tOptimizer("workflow.actions.rollbackAria")} />;
}

function ActionCluster({ actions, label }: { actions: CoreAction[]; label: string }) {
  const visibleActions = actions.slice(0, 2);
  const utilityActions = actions.slice(2);

  return (
    <div className="action-bar action-bar--pattern" aria-label={label}>
      {visibleActions.map((action) => (
        <ActionButton action={action} className="action-bar__button" key={action.id} />
      ))}
      {utilityActions.length > 0 ? <IconToolbar actions={utilityActions} label={label} /> : null}
    </div>
  );
}

function SessionRollbackActions({ session }: { session: RollbackSession }) {
  const hasGpuSession = hasGpuProfileRollback(session);

  return (
    <div
      className="rollback-actions"
      aria-label={tOptimizer("workflow.actions.sessionRollbackAria", { session: session.label })}
    >
      <IconToolbar
        actions={createSessionRollbackActions(session, hasGpuSession)}
        label={tOptimizer("workflow.actions.sessionRollbackAria", { session: session.label })}
      />
    </div>
  );
}

function GpuRollbackFlow({ sessions }: { sessions: RollbackSession[] }) {
  if (sessions.length === 0) {
    return (
      <StatusRow
        label="Profile backup"
        value="Unavailable"
        detail="No GPU profile snapshot is attached to the visible sessions."
        tone="neutral"
      />
    );
  }

  const profileItems = sessions.flatMap((session) =>
    session.items.filter(isGpuProfileRollbackItem).map((item) => ({
      ...item,
      sessionLabel: session.label
    }))
  );

  return (
    <div style={compactRowStyle}>
      {profileItems.map((item) => (
        <StatusRow
          key={`${item.sessionLabel}-${item.id}`}
          label={item.label}
          value={item.state}
          detail={`${item.before} -> ${item.after}; ${item.rollback}`}
          tone={item.state.toLowerCase().includes("ready") ? "success" : "warning"}
        />
      ))}
    </div>
  );
}

function hasGpuProfileRollback(session: RollbackSession) {
  return session.items.some(isGpuProfileRollbackItem);
}

function isGpuProfileRollbackItem(item: RollbackItem) {
  return (
    item.id.startsWith("nvidia.") ||
    /gpu|nvidia|profile/i.test(`${item.label} ${item.rollback}`)
  );
}

function WorkflowHeader({ eyebrow, title, actions }: { eyebrow: string; title: string; actions?: ReactNode }) {
  return (
    <header className="page-header">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
      </div>
      {actions}
    </header>
  );
}

function Surface({
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

function MetricGrid({ metrics }: { metrics: DashboardMetric[] }) {
  return (
    <section className="metric-grid" aria-label="Dashboard metrics">
      {metrics.map((metric) => (
        <MetricReadout key={metric.id} metric={metric} />
      ))}
    </section>
  );
}

function StatusRow({
  label,
  value,
  detail,
  tone
}: {
  label: string;
  value: string;
  detail: string;
  tone: WorkflowTone;
}) {
  return (
    <div style={statusRowStyle} data-tone={tone}>
      <span style={{ borderColor: toneAccent[tone] }} aria-hidden="true" />
      <div style={statusRowBodyStyle}>
        <strong>{label}</strong>
        <small>
          {value} - {detail}
        </small>
      </div>
    </div>
  );
}

function SignalList({ items }: { items: ReadinessSignal[] }) {
  return (
    <div style={compactRowStyle}>
      {items.map((item) => (
        <StatusRow
          key={item.id}
          label={item.label}
          value={item.value}
          detail={item.detail}
          tone={item.tone}
        />
      ))}
    </div>
  );
}

function DefinitionGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <dl style={definitionGridStyle}>
      {items.map(([label, value]) => (
        <div key={label} style={definitionRowStyle}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function PowerPlanTable({ plans }: { plans: PowerPlan[] }) {
  return (
    <div className="bucket-table" role="table" aria-label="Power plan ladder">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Plan</span>
        <span role="columnheader">Mode</span>
        <span role="columnheader">State</span>
        <span role="columnheader">Rollback</span>
      </div>
      {plans.map((plan) => (
        <div className="bucket-row" data-tone={plan.tone} role="row" key={plan.id}>
          <span role="cell">
            <strong>{plan.label}</strong>
            <small>{plan.detail}</small>
          </span>
          <span role="cell">
            {plan.mode}
            <small>Default: {plan.defaults}</small>
          </span>
          <span role="cell">{plan.state}</span>
          <span role="cell">{plan.rollback}</span>
        </div>
      ))}
    </div>
  );
}

function NvidiaProfileTable({ profiles }: { profiles: NvidiaProfile[] }) {
  return (
    <div className="bucket-table" role="table" aria-label="NVIDIA profile states">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Profile</span>
        <span role="columnheader">Scope</span>
        <span role="columnheader">State</span>
        <span role="columnheader">Rollback</span>
      </div>
      {profiles.map((profile) => (
        <div className="bucket-row" data-tone={profile.tone} role="row" key={profile.id}>
          <span role="cell">
            <strong>{profile.label}</strong>
            <small>{profile.recommendation}</small>
          </span>
          <span role="cell">{profile.scope}</span>
          <span role="cell">{profile.state}</span>
          <span role="cell">{profile.rollback}</span>
        </div>
      ))}
    </div>
  );
}

function PubgDxTable({ choices }: { choices: PubgDxChoice[] }) {
  return (
    <div className="bucket-table" role="table" aria-label="PUBG DirectX benchmark choices">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Mode</span>
        <span role="columnheader">Evidence</span>
        <span role="columnheader">State</span>
        <span role="columnheader">Rollback</span>
      </div>
      {choices.map((choice) => (
        <div className="bucket-row" data-tone={choice.tone} role="row" key={choice.id}>
          <span role="cell">
            <strong>{choice.label}</strong>
          </span>
          <span role="cell">{choice.evidence}</span>
          <span role="cell">{choice.state}</span>
          <span role="cell">{choice.rollback}</span>
        </div>
      ))}
    </div>
  );
}

function PubgDxBenchmarkFlow({ benchmark }: { benchmark: PubgDxBenchmark }) {
  return (
    <div style={compactRowStyle}>
      <DefinitionGrid
        items={[
          ["Current", benchmark.currentMode],
          ["Selected", benchmark.selectedMode],
          ["Variance", benchmark.varianceBand],
          ["Policy", benchmark.rationale]
        ]}
      />
      <FlowList items={benchmark.steps} />
      <DefinitionGrid items={benchmark.metadata} />
    </div>
  );
}

function PubgDxBenchmarkResults({ benchmark }: { benchmark: PubgDxBenchmark }) {
  return (
    <div className="bucket-table" role="table" aria-label="PUBG DirectX benchmark result rationale">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Mode</span>
        <span role="columnheader">FPS</span>
        <span role="columnheader">Frame time</span>
        <span role="columnheader">Verdict</span>
      </div>
      {benchmark.results.map((result) => (
        <div className="bucket-row" data-tone={result.tone} role="row" key={result.id}>
          <span role="cell">
            <strong>{result.label}</strong>
            <small>0.1% low {result.pointOnePercentLow} FPS</small>
          </span>
          <span role="cell">
            {result.averageFps} avg
            <small>{result.onePercentLow} FPS 1% low</small>
          </span>
          <span role="cell">
            p95 {result.p95FrameMs} ms
            <small>{result.droppedFrames} dropped frames</small>
          </span>
          <span role="cell">{result.verdict}</span>
        </div>
      ))}
    </div>
  );
}

function PubgLaunchOptionsTable({ options }: { options: PubgLaunchOption[] }) {
  if (options.length === 0) {
    return (
      <StatusRow
        label="Launch options"
        value="Clean"
        detail="No legacy launch flags were detected in the current planner state."
        tone="success"
      />
    );
  }

  return (
    <div className="bucket-table" role="table" aria-label="PUBG launch option cleanup plan">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Token</span>
        <span role="columnheader">Reason</span>
        <span role="columnheader">Recommendation</span>
        <span role="columnheader">Backup</span>
      </div>
      {options.map((option) => (
        <div className="bucket-row" data-tone={option.tone} role="row" key={option.id}>
          <span role="cell">
            <strong>{option.token}</strong>
          </span>
          <span role="cell">{option.reason}</span>
          <span role="cell">{option.recommendation}</span>
          <span role="cell">{option.backup}</span>
        </div>
      ))}
    </div>
  );
}

function BenchmarkChart({ points }: { points: BenchmarkPoint[] }) {
  const maxLow = Math.max(...points.map((point) => point.onePercentLow), 1);

  return (
    <div className="benchmark-chart" role="img" aria-label="Benchmark 1 percent low FPS comparison">
      {points.map((point) => {
        const width = `${Math.max(12, Math.round((point.onePercentLow / maxLow) * 100))}%`;

        return (
          <div className="benchmark-row" data-tone={point.tone} key={point.id}>
            <div>
              <strong>{point.label}</strong>
              <span>
                {point.averageFps} avg FPS - p95 {point.p95FrameMs} ms
              </span>
            </div>
            <span className="benchmark-row__track" aria-hidden="true">
              <span className="benchmark-row__fill" style={{ width }} />
            </span>
            <strong>{point.onePercentLow} FPS 1% low</strong>
          </div>
        );
      })}
    </div>
  );
}

function ProgressBar({ percent, label }: { percent: number; label: string }) {
  return (
    <div style={compactRowStyle}>
      <div
        aria-label={`${label}: ${percent}% complete`}
        aria-valuemax={100}
        aria-valuemin={0}
        aria-valuenow={percent}
        role="progressbar"
        style={progressTrackStyle}
      >
        <span style={{ ...progressBarStyle, width: `${percent}%` }} />
      </div>
      <small className="workflow-muted">{label}</small>
    </div>
  );
}

function FlowList({ items }: { items: Array<ScanState | ApplyStep> }) {
  return (
    <ol className="flow-rail" aria-label="Workflow steps">
      {items.map((item) => (
        <li className="flow-step" data-state={item.state} key={item.id}>
          <span className="flow-step__marker" aria-hidden="true" />
          <div>
            <strong>{item.label}</strong>
            <span>{item.detail}</span>
          </div>
        </li>
      ))}
    </ol>
  );
}

function FindingGrid({ findings }: { findings: ScanFinding[] }) {
  return (
    <div style={findingGridStyle}>
      {findings.map((finding) => (
        <article className="metric-tile" data-tone={finding.tone} key={finding.id}>
          <span>{finding.group}</span>
          <strong>{finding.title}</strong>
          <small>
            {tOptimizer(optimizerGlossaryKeys.risk)}: {finding.risk}. {finding.detail}
          </small>
        </article>
      ))}
    </div>
  );
}

function PlanGroupGrid({ groups }: { groups: PlanGroup[] }) {
  return (
    <div className="plan-group-grid" style={viewGridStyle}>
      {groups.map((group) => (
        <Surface
          key={group.id}
          title={group.label}
          eyebrow={group.summary}
          badge={getPlanGroupBadge(group)}
        >
          <PlanGroupPolicy group={group} />
          <details className="optimizer-inspector" open={group.id === "safe"}>
            <summary className="optimizer-inspector__summary">
              <span>
                <strong>{group.label} changes</strong>
                <small>
                  Impact, risk, confidence, restart notes, and recovery details
                </small>
              </span>
              <span className="pill">{group.tweaks.length} rows</span>
            </summary>
            <PlanTable tweaks={group.tweaks} tone={group.tone} />
          </details>
        </Surface>
      ))}
    </div>
  );
}

function PlanGroupPolicy({ group }: { group: PlanGroup }) {
  const rebootTweaks = group.tweaks.filter((tweak) => !/^(no|n\/a)$/i.test(tweak.reboot));

  return (
    <div style={planGroupPolicyStyle} aria-label={`${group.label} policy`}>
      <StatusRow
        label={tOptimizer("workflow.plan.riskLabel")}
        value={getPlanGroupRiskLabel(group)}
        detail={tOptimizer("workflow.plan.bucketChangeSummaries", { count: group.tweaks.length })}
        tone={group.tone}
      />
      <StatusRow
        label={tOptimizer("workflow.plan.consent")}
        value={getPlanGroupConsent(group)}
        detail={getPlanGroupApplyState(group)}
        tone={group.tone}
      />
      <StatusRow
        label={tOptimizer(optimizerGlossaryKeys.reboot)}
        value={
          rebootTweaks.length > 0
            ? tOptimizer("workflow.plan.rebootMarked", { count: rebootTweaks.length })
            : tOptimizer("labels.none")
        }
        detail={
          rebootTweaks.length > 0
            ? rebootTweaks.map((tweak) => tweak.reboot).join(", ")
            : tOptimizer("workflow.plan.noRebootQueued")
        }
        tone={rebootTweaks.length > 0 ? "warning" : "success"}
      />
      <StatusRow
        label={tOptimizer(optimizerGlossaryKeys.rollback)}
        value={group.id === "blocked" ? tOptimizer("labels.noMutation") : tOptimizer("labels.required")}
        detail={
          group.id === "blocked"
            ? tOptimizer("workflow.plan.blockedRollbackDetail")
            : tOptimizer("workflow.plan.writeRollbackDetail")
        }
        tone={group.id === "blocked" ? "danger" : "success"}
      />
    </div>
  );
}

function PlanTable({ tweaks, tone }: { tweaks: PlanTweak[]; tone: WorkflowTone }) {
  return <TweakLedger rows={createTweakLedgerRows(tweaks, tone)} />;
}

function RollbackTable({ items }: { items: RollbackItem[] }) {
  return (
    <div className="bucket-table" role="table" aria-label={tOptimizer("workflow.rollback.valuesAria")}>
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">{tOptimizer("labels.item")}</span>
        <span role="columnheader">{tOptimizer("labels.before")}</span>
        <span role="columnheader">{tOptimizer("labels.after")}</span>
        <span role="columnheader">{tOptimizer(optimizerGlossaryKeys.rollback)}</span>
      </div>
      {items.map((item) => (
        <div className="bucket-row" role="row" key={item.id}>
          <span role="cell">
            <strong>{item.label}</strong>
            <small>{item.state}</small>
          </span>
          <span role="cell">{item.before}</span>
          <span role="cell">{item.after}</span>
          <span role="cell">{item.rollback}</span>
        </div>
      ))}
    </div>
  );
}

function createDashboardOptimizerLanes(
  data: DashboardData,
  optimizeData: OptimizeData | undefined,
  rollbackData: RollbackData | undefined,
  scanData: ScanData | undefined
): OptimizerLane[] {
  const safeGroup = optimizeData?.groups.find((group) => group.id === "safe");
  const competitiveGroup = optimizeData?.groups.find((group) => group.id === "competitive");
  const labGroup = optimizeData?.groups.find((group) => group.id === "lab");
  const blockedGroup = optimizeData?.groups.find((group) => group.id === "blocked");
  const scanPercent = scanData?.progress.percent ?? 0;
  const rollbackCount = rollbackData?.sessions.length ?? 0;

  return [
    {
      details: [
        ["Scope", scanData ? `${scanData.scopes.filter((scope) => scope.checked).length}/${scanData.scopes.length}` : "Pending"],
        ["Recommendations", scanData ? String(scanData.findings.length) : "Run scan"],
        ["Changes", tOptimizer("labels.noMutation")]
      ],
      eyebrow: "Scan only",
      id: "dashboard-scan",
      label: tOptimizer(optimizerGlossaryKeys.scan),
      primaryAction: createLaneAction("dashboard-start-scan", tOptimizer("actions.startScan"), "secondary"),
      status: scanData ? `${scanPercent}% complete` : tOptimizer("labels.ready"),
      summary: scanData?.progress.current ?? "Check system, GPU, game, and recovery context first.",
      tone: "active",
      trustSignal: "No changes applied"
    },
    {
      details: [
        ["Safe changes", String(safeGroup?.tweaks.length ?? 0)],
        ["Tradeoffs", String(competitiveGroup?.tweaks.length ?? 0)],
        ["Blocked", String(blockedGroup?.tweaks.length ?? 0)]
      ],
      detailAction: createLaneAction("dashboard-review-plan", tOptimizer("actions.reviewPlan"), "ghost"),
      eyebrow: "Default path",
      id: "dashboard-safe",
      label: tOptimizer(optimizerGlossaryKeys.safe),
      primaryAction: createLaneAction("dashboard-apply-safe", tOptimizer("actions.applySafeOnly"), "secondary"),
      status: safeGroup ? `${safeGroup.tweaks.length} ready` : "Plan pending",
      summary: "Reversible recommendations stay separated from Competitive, Lab, and Blocked changes.",
      tone: "success",
      trustSignal: "Recovery ready"
    },
    {
      details: [
        ["PUBG", data.pubgReadiness],
        ["GPU", data.driverState],
        ["Benchmark", data.lastBenchmarkDelta]
      ],
      detailAction: createLaneAction("dashboard-open-benchmark", tOptimizer("actions.openBenchmark"), "ghost"),
      eyebrow: "Game mode",
      id: "dashboard-game",
      label: "Game Mode",
      primaryAction: createLaneAction("dashboard-review-game", "Review game plan", "secondary"),
      status: data.pubgReadiness,
      summary: "Game detection, profile state, launch config, and benchmark prompt live as one lane.",
      tone: "benchmark",
      trustSignal: "Anti-cheat safe"
    },
    {
      details: [
        ["Power", data.activePowerPlan],
        ["Background load", data.readinessSignals.find((signal) => signal.id === "background-load")?.value ?? "Review"],
        ["Lab", `${labGroup?.tweaks.length ?? 0} experiments`]
      ],
      eyebrow: "System lanes",
      id: "dashboard-system",
      label: "System, Power, Startup",
      primaryAction: createLaneAction("dashboard-open-system", tOptimizer("actions.reviewPlan"), "secondary"),
      status: data.activePowerPlan,
      summary: "System, power, startup/services, and Lab work stay visible without crowding the first action.",
      tone: "warning",
      trustSignal: "Review gated"
    },
    {
      details: [
        ["Sessions", String(rollbackCount)],
        ["Availability", data.rollbackAvailability],
        ["Trust", data.trustState]
      ],
      detailAction: createLaneAction("dashboard-export-rollback", tOptimizer("actions.exportAudit"), "ghost"),
      eyebrow: "Recovery",
      id: "dashboard-rollback",
      label: tOptimizer(optimizerGlossaryKeys.rollback),
      primaryAction: createLaneAction("dashboard-open-rollback", tOptimizer("actions.openRollback"), "rollback"),
      status: data.rollbackAvailability,
      summary: "Restore points, app snapshots, and session recovery stay attached before and after apply.",
      tone: "rollback",
      trustSignal: data.trustState
    }
  ];
}

function createOptimizerCategoryLanes(groups: PlanGroup[]): OptimizerLane[] {
  const allTweaks = groups.flatMap((group) => group.tweaks.map((tweak) => ({ group, tweak })));
  const groupById = new Map(groups.map((group) => [group.id, group]));
  const safeGroup = groupById.get("safe");
  const labGroup = groupById.get("lab");
  const blockedGroup = groupById.get("blocked");

  const selectTweaks = (pattern: RegExp) =>
    allTweaks.filter(({ tweak }) => pattern.test(`${tweak.id} ${tweak.change} ${tweak.why}`));

  const gameTweaks = selectTweaks(/game|capture|pubg|present/i);
  const systemTweaks = selectTweaks(/security|hvci|defender|windows|memory integrity/i);
  const networkTweaks = selectTweaks(/net\.|adapter|rsc|network/i);
  const gpuTweaks = selectTweaks(/gpu|graphics|hags|nvidia|profile|rebar/i);
  const powerTweaks = selectTweaks(/power|plan/i);
  const rollbackTweaks = allTweaks.filter(({ group, tweak }) => group.id !== "blocked" && !/not applied|no mutation/i.test(tweak.rollback));

  return [
    laneFromPlanGroup({
      actionLabel: tOptimizer("actions.applySafeOnly"),
      actionVariant: "secondary",
      detailLabel: tOptimizer("actions.reviewPlan"),
      detailVariant: "ghost",
      eyebrow: "Safe default",
      group: safeGroup,
      id: "safe",
      label: tOptimizer(optimizerGlossaryKeys.safe),
      summary: "Low-risk reversible changes are the only default apply path.",
      trustSignal: "Backup first"
    }),
    laneFromTweaks({
      actionLabel: "Customize game mode",
      actionVariant: "secondary",
      detailLabel: tOptimizer("actions.openBenchmark"),
      detailVariant: "ghost",
      eyebrow: "Game",
      id: "game-mode",
      label: "Game Mode",
      summary: "Capture, Game Mode, present path, and game profile changes are reviewed together.",
      tone: "benchmark",
      trustSignal: "Anti-cheat boundary",
      tweaks: gameTweaks
    }),
    laneFromTweaks({
      actionLabel: "Review system",
      actionVariant: "secondary",
      eyebrow: "Windows",
      id: "system",
      label: "System",
      summary: "OS security tradeoffs and blocked system changes stay separated from safe defaults.",
      tone: "warning",
      trustSignal: "Consent required",
      tweaks: systemTweaks
    }),
    laneFromTweaks({
      actionLabel: "Inspect network",
      actionVariant: "secondary",
      eyebrow: "Connectivity",
      id: "network",
      label: "Network",
      summary: "Adapter experiments stay diagnostic and require review before apply.",
      tone: "lab",
      trustSignal: "Benchmark gated",
      tweaks: networkTweaks
    }),
    laneFromTweaks({
      actionLabel: "Review GPU",
      actionVariant: "secondary",
      detailLabel: tOptimizer("actions.openNvidiaProfile"),
      detailVariant: "ghost",
      eyebrow: "Graphics",
      id: "gpu",
      label: "GPU",
      summary: "HAGS, NVIDIA profile, refresh, and cap decisions stay tied to backup and proof.",
      tone: "active",
      trustSignal: "Profile backup",
      tweaks: gpuTweaks
    }),
    laneFromTweaks({
      actionLabel: tOptimizer("actions.stageBalanced"),
      actionVariant: "secondary",
      eyebrow: "Power",
      id: "power",
      label: "Power",
      summary: "Power plan changes remain scoped, reversible, and separated from global defaults.",
      tone: "success",
      trustSignal: "Previous scheme saved",
      tweaks: powerTweaks
    }),
    {
      details: [
        ["Default action", "None"],
        ["Review source", "Startup scan findings"],
        ["Recovery", "Required before service changes"]
      ],
      eyebrow: "Startup",
      id: "startup-services",
      label: "Startup/Services",
      primaryAction: createLaneAction("startup-services-locked", "Finish scan to unlock", "locked", true),
      status: "Review after scan",
      summary: "Startup and service changes stay out of Safe Boost until scan evidence is complete.",
      tone: "locked",
      trustSignal: "No default apply"
    },
    laneFromPlanGroup({
      actionLabel: tOptimizer("actions.inspectLab"),
      actionVariant: "secondary",
      eyebrow: "Advanced",
      group: labGroup,
      id: "lab",
      label: tOptimizer(optimizerGlossaryKeys.lab),
      summary: "Experiments require explicit opt-in, benchmark framing, and rollback notes.",
      trustSignal: "Advanced opt-in"
    }),
    laneFromPlanGroup({
      actionLabel: "Blocked by policy",
      actionVariant: "locked",
      disabled: true,
      eyebrow: "Denied",
      group: blockedGroup,
      id: "blocked",
      label: tOptimizer(optimizerGlossaryKeys.blocked),
      summary: "Unsafe or anti-cheat-hostile changes stay blocked.",
      trustSignal: tOptimizer("labels.noMutation")
    }),
    laneFromTweaks({
      actionLabel: tOptimizer("actions.openRollback"),
      actionVariant: "rollback",
      detailLabel: tOptimizer("actions.exportAudit"),
      detailVariant: "ghost",
      eyebrow: "Recovery",
      id: "rollback-aware",
      label: "Rollback-aware",
      summary: "Every change that can be applied shows its recovery value first.",
      tone: "rollback",
      trustSignal: tOptimizer("labels.required"),
      tweaks: rollbackTweaks
    })
  ];
}

function laneFromPlanGroup({
  actionLabel,
  actionVariant,
  detailLabel,
  detailVariant,
  disabled,
  eyebrow,
  group,
  id,
  label,
  summary,
  trustSignal
}: {
  actionLabel: string;
  actionVariant: CoreActionVariant;
  detailLabel?: string;
  detailVariant?: CoreActionVariant;
  disabled?: boolean;
  eyebrow: string;
  group: PlanGroup | undefined;
  id: string;
  label: string;
  summary: string;
  trustSignal: string;
}): OptimizerLane {
  const tweakCount = group?.tweaks.length ?? 0;

  return {
    details: [
      ["Changes", String(tweakCount)],
      ["Apply", group ? getPlanGroupApplyState(group) : "Not ready"],
      ["Consent", group ? getPlanGroupConsent(group) : "Review required"]
    ],
    eyebrow,
    id,
    label,
    primaryAction: createLaneAction(`${id}-primary`, actionLabel, actionVariant, disabled),
    ...(detailLabel
      ? { detailAction: createLaneAction(`${id}-detail`, detailLabel, detailVariant ?? "ghost") }
      : {}),
    status: tweakCount > 0 ? `${tweakCount} changes` : "No default apply",
    summary,
    tone: group?.tone ?? (disabled ? "locked" : "neutral"),
    trustSignal
  };
}

function laneFromTweaks({
  actionLabel,
  actionVariant,
  detailLabel,
  detailVariant,
  eyebrow,
  id,
  label,
  summary,
  tone,
  trustSignal,
  tweaks
}: {
  actionLabel: string;
  actionVariant: CoreActionVariant;
  detailLabel?: string;
  detailVariant?: CoreActionVariant;
  eyebrow: string;
  id: string;
  label: string;
  summary: string;
  tone: WorkflowTone;
  trustSignal: string;
  tweaks: Array<{ group: PlanGroup; tweak: PlanTweak }>;
}): OptimizerLane {
  const rebootCount = tweaks.filter(({ tweak }) => !/^(no|n\/a)$/i.test(tweak.reboot)).length;

  return {
    details: [
      ["Changes", String(tweaks.length)],
      ["Restart notes", String(rebootCount)],
      ["Risk", tweaks.length > 0 ? summarizeLaneRisk(tweaks) : "Scan pending"]
    ],
    eyebrow,
    id,
    label,
    primaryAction: createLaneAction(`${id}-primary`, actionLabel, actionVariant, tweaks.length === 0),
    ...(detailLabel
      ? { detailAction: createLaneAction(`${id}-detail`, detailLabel, detailVariant ?? "ghost", tweaks.length === 0) }
      : {}),
    status: tweaks.length > 0 ? `${tweaks.length} related` : "Scan pending",
    summary,
    tone: tweaks.length > 0 ? tone : "locked",
    trustSignal
  };
}

function createApplyProgressCards(groups: PlanGroup[], steps: ApplyStep[]) {
  const stepById = new Map(steps.map((step) => [step.id, step]));
  const backupStep = stepById.get("backup");
  const applyStep = stepById.get("apply");
  const verifyStep = stepById.get("verify");
  const benchmarkStep = stepById.get("benchmark");
  const rollbackStep = stepById.get("rollback");
  const blockedTweaks = groups.find((group) => group.id === "blocked")?.tweaks.length ?? 0;
  const rebootTweaks = groups.flatMap((group) =>
    group.tweaks.filter((tweak) => !/^(no|n\/a)$/i.test(tweak.reboot))
  );

  return [
    {
      detail: backupStep?.detail ?? "Backup must be ready before changes are applied.",
      id: "backup",
      label: "Backup",
      tone: backupStep ? stateToTone(backupStep.state) : "danger",
      value: backupStep?.state ?? "Missing"
    },
    {
      detail: applyStep?.detail ?? "Only safe changes can run without extra consent.",
      id: "apply",
      label: tOptimizer(optimizerGlossaryKeys.apply),
      tone: applyStep ? stateToTone(applyStep.state) : "neutral",
      value: applyStep?.label ?? "Ready"
    },
    {
      detail: verifyStep?.detail ?? "Results are checked before completion.",
      id: "verify",
      label: "Verification",
      tone: verifyStep ? stateToTone(verifyStep.state) : "neutral",
      value: verifyStep?.state ?? "Pending"
    },
    {
      detail: benchmarkStep?.detail ?? "Benchmark prompt follows verify for gaming changes.",
      id: "benchmark",
      label: tOptimizer(optimizerGlossaryKeys.benchmark),
      tone: "benchmark",
      value: benchmarkStep?.state ?? "Pending"
    },
    {
      detail: `${blockedTweaks} blocked changes stay informational; failed changes stop before verification.`,
      id: "failure",
      label: "Blocked changes",
      tone: blockedTweaks > 0 ? "danger" : "success",
      value: blockedTweaks > 0 ? "Isolated" : "Clear"
    },
    {
      detail: rebootTweaks.length > 0 ? rebootTweaks.map((tweak) => tweak.change).join(", ") : "No restart note is queued.",
      id: "reboot",
      label: tOptimizer(optimizerGlossaryKeys.reboot),
      tone: rebootTweaks.length > 0 ? "warning" : "success",
      value: rebootTweaks.length > 0 ? `${rebootTweaks.length} marked` : tOptimizer("labels.none")
    },
    {
      detail: rollbackStep?.detail ?? "Restore actions stay attached to this boost session.",
      id: "rollback",
      label: tOptimizer(optimizerGlossaryKeys.rollback),
      tone: "rollback",
      value: rollbackStep?.state === "pending" ? "Armed after backup" : (rollbackStep?.state ?? "Ready")
    }
  ];
}

function createLaneAction(
  id: string,
  label: string,
  variant: CoreActionVariant = "secondary",
  disabled = false
): CoreAction {
  return {
    disabled,
    icon: disabled ? "ban" : actionIconForLabel(label),
    id,
    label,
    tooltip: label,
    variant
  };
}

function summarizeLaneRisk(tweaks: Array<{ group: PlanGroup; tweak: PlanTweak }>) {
  const riskOrder = ["Critical", "High", "Medium", "Low"];
  const risk = riskOrder.find((label) => tweaks.some(({ tweak }) => new RegExp(label, "i").test(tweak.risk)));

  return risk ?? "Review";
}

function createTweakLedgerRows(tweaks: PlanTweak[], tone: WorkflowTone): TweakLedgerRowData[] {
  return tweaks.map((tweak) => ({
    change: tweak.change,
    confidence: tweak.confidence,
    consent: getTweakConsent(tweak, tone),
    id: tweak.id,
    impact: tweak.expectedImpact,
    reboot: tweak.reboot,
    risk: riskLevelFromLabel(tweak.risk),
    riskLabel: tweak.risk,
    rollback: tweak.rollback,
    source: tweak.why,
    tone
  }));
}

function createPlanDiffPreview(groups: PlanGroup[]): DiffItem[] {
  return groups
    .flatMap((group) => group.tweaks.slice(0, group.id === "safe" ? 2 : 1).map((tweak) => ({ group, tweak })))
    .slice(0, 5)
    .map(({ group, tweak }) => ({
      after: `${tOptimizer("labels.impact")}: ${tweak.expectedImpact}`,
      before: `${tOptimizer(optimizerGlossaryKeys.rollback)}: ${tweak.rollback}`,
      id: `${group.id}-${tweak.id}`,
      label: tweak.change,
      tone: group.tone
    }));
}

function toTimelineSteps(steps: ApplyStep[]): TimelineStep[] {
  return steps.map((step) => ({
    detail: step.detail,
    id: step.id,
    label: step.label,
    state: step.state,
    tone: stateToTone(step.state)
  }));
}

function createRollbackSessionLog(session: RollbackSession): RollbackSessionLogData {
  return {
    actions: createSessionRollbackActions(session, hasGpuProfileRollback(session)),
    id: session.id,
    items: session.items.map((item) => ({
      after: `${item.after}; ${item.rollback}`,
      before: item.before,
      id: item.id,
      label: item.label,
      tone: item.state.toLowerCase().includes("ready") ? "success" : "rollback"
    })),
    label: session.label,
    rebootRequired: session.rebootRequired,
    state: session.state,
    summary: session.summary,
    time: session.time
  };
}

function createSessionRollbackActions(session: RollbackSession, hasGpuSession: boolean): CoreAction[] {
  return [
    {
      icon: "rollback",
      id: `${session.id}-restore-all`,
      label: tOptimizer("actions.restoreAll"),
      tooltip: tOptimizer("tooltips.restoreAllChangesFromSession", { session: session.label }),
      variant: "secondary"
    },
    ...(hasGpuSession
      ? [
          {
            icon: "gpu" as const,
            id: `${session.id}-restore-gpu`,
            label: tOptimizer("actions.restoreGpuProfiles"),
            tooltip: tOptimizer("tooltips.restoreGpuProfilesFromSession", { session: session.label }),
            variant: "ghost" as const
          }
        ]
      : [])
  ];
}

const scanCompletedCheckSequence = [
  "OS inventory",
  "CPU topology",
  "Active power plan",
  "PUBG process check",
  "GPU driver state",
  "Display and VRR policy",
  "Storage cleanup candidates",
  "Network adapter power",
  "Rollback preflight"
];

function useSmartScanRun(data: ScanData) {
  const [selectedScopeIds, setSelectedScopeIds] = useState(
    () => new Set(data.scopes.filter((scope) => scope.checked).map((scope) => scope.id))
  );
  const [runState, setRunState] = useState<ScanRunState>(() => ({
    current: data.progress.current,
    percent: data.progress.percent,
    phase: data.progress.percent >= 100 ? "complete" : "scanning",
    running: false
  }));

  useEffect(() => {
    setSelectedScopeIds(new Set(data.scopes.filter((scope) => scope.checked).map((scope) => scope.id)));
    setRunState({
      current: data.progress.current,
      percent: data.progress.percent,
      phase: data.progress.percent >= 100 ? "complete" : "scanning",
      running: false
    });
  }, [data]);

  useEffect(() => {
    if (!runState.running) {
      return undefined;
    }

    const interval = window.setInterval(() => {
      setRunState((current) => {
        if (!current.running) {
          return current;
        }

        const nextPercent = Math.min(100, current.percent + (current.percent < 70 ? 8 : 5));

        if (nextPercent >= 100) {
          return {
            current: getScanCurrentForPercent(100),
            percent: 100,
            phase: "complete",
            running: false
          };
        }

        return {
          current: getScanCurrentForPercent(nextPercent),
          percent: nextPercent,
          phase: "scanning",
          running: true
        };
      });
    }, 520);

    return () => window.clearInterval(interval);
  }, [runState.running]);

  const activeData = useMemo(
    () => createInteractiveScanData(data, runState, selectedScopeIds),
    [data, runState, selectedScopeIds]
  );

  const start = () => {
    setRunState((current) => {
      const shouldRestart = current.phase === "complete" || current.phase === "failed";
      const percent = shouldRestart ? 0 : Math.max(0, Math.min(99, current.percent));

      return {
        current: percent > 0 ? getScanCurrentForPercent(percent) : "Preparing selected Smart Scan checks",
        percent,
        phase: "scanning",
        running: true
      };
    });

    void runDesktopAction({
      command: "run-read-only-scan",
      errorFeedback: "Smart Scan could not complete; no changes were applied.",
      feedback: "Smart Scan is checking selected areas without applying changes.",
      id: "start-scan",
      label: tOptimizer("actions.startScan"),
      successFeedback: "Smart Scan is running or completed.",
      targetRoute: "scan"
    });
  };

  const cancel = () => {
    setRunState((current) => ({
      current: `Smart Scan paused at ${Math.round(current.percent)}%; no changes were made`,
      percent: current.percent,
      phase: "cancelled",
      running: false
    }));

    void runDesktopAction({
      command: "status",
      feedback: "Smart Scan paused. No changes were applied.",
      id: "cancel-scan",
      label: tOptimizer("actions.cancelScan"),
      targetRoute: "scan"
    });
  };

  const generatePlan = () => {
    if (activeData.progress.percent < 100) {
      return;
    }

    void runDesktopAction({
      command: "review-plan",
      feedback: "Opening Smart Boost with recommendations and recovery checks.",
      id: "generate-plan",
      label: tOptimizer("actions.generatePlan"),
      successFeedback: "Smart Boost is ready for review.",
      targetRoute: "optimize"
    });
  };

  const toggleScope = (scopeId: string) => {
    if (runState.running) {
      return;
    }

    setSelectedScopeIds((current) => {
      const next = new Set(current);

      if (next.has(scopeId)) {
        if (next.size > 1) {
          next.delete(scopeId);
        }
      } else {
        next.add(scopeId);
      }

      return next;
    });
  };

  return {
    cancel,
    data: activeData,
    generatePlan,
    running: runState.running,
    start,
    toggleScope
  };
}

function createInteractiveScanData(
  data: ScanData,
  runState: ScanRunState,
  selectedScopeIds: Set<string>
): ScanData {
  const percent = Math.max(0, Math.min(100, Math.round(runState.percent)));
  const completedCount = Math.floor((percent / 100) * scanCompletedCheckSequence.length);
  const completed =
    percent === 0
      ? []
      : Array.from(new Set([...data.progress.completed, ...scanCompletedCheckSequence.slice(0, completedCount)]));

  return {
    ...data,
    scopes: data.scopes.map((scope) => ({
      ...scope,
      checked: selectedScopeIds.has(scope.id)
    })),
    states: createInteractiveScanStates(data.states, runState.phase, percent),
    progress: {
      completed,
      current: runState.current || getScanCurrentForPercent(percent),
      label: getScanLabelForPercent(percent),
      percent
    }
  };
}

function createInteractiveScanStates(states: ScanState[], phase: ScanRunPhase, percent: number): ScanState[] {
  return states.map((state) => ({
    ...state,
    detail: state.id === "scanning" && phase === "scanning" ? getScanCurrentForPercent(percent) : state.detail,
    state: getInteractiveScanStateValue(state.id, phase, percent)
  }));
}

function getInteractiveScanStateValue(id: string, phase: ScanRunPhase, percent: number): ScanState["state"] {
  if (phase === "failed") {
    return id === "failed" ? "active" : percent > 0 && id !== "complete" && id !== "cancelled" ? "complete" : "pending";
  }

  if (phase === "cancelled") {
    if (id === "cancelled") {
      return "active";
    }

    if (id === "idle" || (id === "partial" && percent >= 55)) {
      return "complete";
    }

    return "pending";
  }

  if (phase === "complete" || percent >= 100) {
    return id === "failed" || id === "cancelled" ? "pending" : "complete";
  }

  if (phase === "idle" || percent <= 0) {
    return id === "idle" ? "active" : "pending";
  }

  if (id === "idle") {
    return "complete";
  }

  if (id === "scanning") {
    return "active";
  }

  if (id === "partial" && percent >= 55) {
    return "complete";
  }

  return "pending";
}

function getScanCurrentForPercent(percent: number) {
  if (percent >= 100) {
    return "Smart Scan complete; Smart Boost is ready";
  }

  if (percent >= 90) {
    return "Finalizing recommendations and safety checks";
  }

  if (percent >= 72) {
    return "Checking network, storage, and recovery context";
  }

  if (percent >= 50) {
    return "Checking driver, VRR, and overlay state";
  }

  if (percent >= 25) {
    return "Checking Game Mode, capture, and startup apps";
  }

  if (percent > 0) {
    return "Checking Windows, CPU, memory, and power state";
  }

  return "Ready to check selected areas";
}

function getScanLabelForPercent(percent: number) {
  if (percent >= 100) {
    return "Complete";
  }

  if (percent >= 90) {
    return "Safety check";
  }

  if (percent >= 72) {
    return "Network and storage";
  }

  if (percent >= 50) {
    return "Graphics path";
  }

  if (percent >= 25) {
    return "Game readiness";
  }

  return "PC baseline";
}

function getTweakConsent(tweak: PlanTweak, tone: WorkflowTone) {
  if (tone === "success") {
    return tOptimizer("workflow.plan.noExtraConsent");
  }

  if (tone === "warning") {
    return tOptimizer("workflow.plan.competitiveConsent");
  }

  if (tone === "lab") {
    return tOptimizer("workflow.plan.labConsent");
  }

  if (/critical|deny|blocked/i.test(`${tweak.risk} ${tweak.change}`)) {
    return tOptimizer("workflow.plan.deniedByPolicy");
  }

  return tOptimizer("workflow.plan.reviewRequired");
}

function getScanNextAction(data: ScanData): {
  badge: string;
  detail: string;
  eyebrow: string;
  label: string;
  tone: WorkflowTone;
  value: string;
} {
  const failedState = data.states.find((state) => state.id === "failed" && state.state === "active");
  const completeState = data.states.find((state) => state.id === "complete");
  const isComplete = data.progress.percent >= 100 || completeState?.state === "complete";

  if (failedState) {
    return {
      badge: "Safe to retry",
      detail: "Retry keeps confirmed recommendations and no changes have occurred.",
      eyebrow: "Scan needs attention",
      label: tOptimizer("actions.retryScan"),
      tone: "danger",
      value: failedState.label
    };
  }

  if (isComplete) {
    return {
      badge: "Boost ready",
      detail: "Open Smart Boost from the completed recommendations.",
      eyebrow: "Scan complete",
      label: tOptimizer("actions.generatePlan"),
      tone: "success",
      value: tOptimizer("labels.ready")
    };
  }

  return {
    badge: "Scan only",
    detail: `${data.progress.current}; Smart Boost remains locked until the scan completes.`,
    eyebrow: "Smart Scan in progress",
    label: tOptimizer("actions.continueScan"),
    tone: "active",
    value: `${data.progress.percent}%`
  };
}

function findWorkflowStep(steps: ApplyStep[], id: string) {
  return steps.find((step) => step.id === id);
}

function stateToTone(state: ApplyStep["state"]): WorkflowTone {
  if (state === "complete") {
    return "success";
  }

  if (state === "active") {
    return "active";
  }

  return "neutral";
}

function getPlanGroupBadge(group: PlanGroup) {
  if (group.applyEnabled) {
    return tOptimizer("workflow.plan.defaultApply");
  }

  if (group.id === "blocked") {
    return tOptimizer("workflow.plan.noApplyControl");
  }

  return tOptimizer("workflow.plan.reviewRequired");
}

function getPlanGroupApplyState(group: PlanGroup) {
  if (group.applyEnabled) {
    return tOptimizer("workflow.plan.applyControlEnabled");
  }

  if (group.id === "blocked") {
    return tOptimizer("workflow.plan.noApplyControlRendered");
  }

  return tOptimizer("workflow.plan.reviewOnlyUntilConsent");
}

function getPlanGroupConsent(group: PlanGroup) {
  if (group.id === "safe") {
    return tOptimizer("workflow.plan.noExtraConsent");
  }

  if (group.id === "competitive") {
    return tOptimizer("workflow.plan.competitiveConsent");
  }

  if (group.id === "lab") {
    return tOptimizer("workflow.plan.labConsent");
  }

  return tOptimizer("workflow.plan.deniedByPolicy");
}

function getPlanGroupRiskLabel(group: PlanGroup) {
  if (group.id === "safe") {
    return tOptimizer("risk.low");
  }

  if (group.id === "competitive") {
    return tOptimizer("risk.mediumShort");
  }

  if (group.id === "lab") {
    return tOptimizer("risk.high");
  }

  return tOptimizer("risk.critical");
}

const checkRowStyle: CSSProperties = {
  alignItems: "start",
  display: "grid",
  gap: "0.65rem",
  gridTemplateColumns: "1rem minmax(0, 1fr)"
};

const statusRowStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem",
  gridTemplateColumns: "0.7rem minmax(0, 1fr)",
  alignItems: "start"
};

const statusRowBodyStyle: CSSProperties = {
  display: "grid",
  gap: "0.18rem",
  minWidth: 0
};

const definitionGridStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem",
  margin: 0
};

const definitionRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "minmax(7.5rem, 0.6fr) minmax(0, 1fr)",
  gap: "0.75rem",
  margin: 0
};

const progressTrackStyle: CSSProperties = {
  height: "0.65rem",
  overflow: "hidden",
  borderRadius: "var(--radius-pill)",
  background: "var(--chart-track)",
  border: "1px solid var(--border)"
};

const progressBarStyle: CSSProperties = {
  display: "block",
  height: "100%",
  background: "var(--active)"
};

const findingGridStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 15rem), 1fr))",
  gap: "0.75rem"
};
