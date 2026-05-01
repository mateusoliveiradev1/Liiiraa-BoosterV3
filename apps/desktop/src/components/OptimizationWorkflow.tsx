import type { CSSProperties, ReactNode } from "react";

type WorkflowTone = "active" | "danger" | "lab" | "neutral" | "success" | "warning";
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
  p95FrameMs: number;
  tone: WorkflowTone;
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
  chart: BenchmarkPoint[];
  metadata: Array<[string, string]>;
  sessions: ReadinessSignal[];
};

type DashboardViewProps = {
  data: DashboardData;
  actions?: ReactNode;
};

type ScanViewProps = {
  data: ScanData;
  actions?: ReactNode;
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

const toneAccent: Record<WorkflowTone, string> = {
  active: "#27d7ff",
  danger: "#ff5a67",
  lab: "#9b7cff",
  neutral: "#9aa8b8",
  success: "#3af28f",
  warning: "#ffbd5a"
};

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

export function DashboardWorkflowView({ data, actions }: DashboardViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Dashboard optimization overview">
      <WorkflowHeader eyebrow="Dashboard" title="System readiness" actions={actions} />
      <MetricGrid metrics={data.metrics} />
      <div style={twoColumnStyle}>
        <Surface title="Readiness signals" eyebrow={`${data.readinessScore}/100 readiness`}>
          <div style={compactRowStyle}>
            {data.readinessSignals.map((signal) => (
              <StatusRow
                key={signal.id}
                label={signal.label}
                value={signal.value}
                detail={signal.detail}
                tone={signal.tone}
              />
            ))}
          </div>
        </Surface>
        <Surface title="Current state" eyebrow="Active optimizer context">
          <DefinitionGrid
            items={[
              ["Mode", data.activeMode],
              ["Power plan", data.activePowerPlan],
              ["GPU driver", data.driverState],
              ["PUBG", data.pubgReadiness],
              ["Last benchmark", data.lastBenchmarkDelta],
              ["Rollback", data.rollbackAvailability],
              ["Trust", data.trustState]
            ]}
          />
        </Surface>
      </div>
    </div>
  );
}

export function ScanWorkflowView({ data, actions }: ScanViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Scan workflow">
      <WorkflowHeader eyebrow="Scan" title="Read-only system scan" actions={actions} />
      <div style={twoColumnStyle}>
        <Surface title="Scan scope" eyebrow="Selected checks">
          <div style={compactRowStyle}>
            {data.scopes.map((scope) => (
              <label className="workflow-check" key={scope.id} style={checkRowStyle}>
                <input checked={scope.checked} readOnly type="checkbox" />
                <span>
                  <strong>{scope.label}</strong>
                  <small>{scope.detail}</small>
                </span>
              </label>
            ))}
          </div>
        </Surface>
        <Surface title={data.progress.label} eyebrow={`${data.progress.percent}% complete`}>
          <ProgressBar percent={data.progress.percent} label={data.progress.current} />
          <FlowList items={data.states} />
        </Surface>
      </div>
      <Surface title="Findings" eyebrow="Grouped by impact and risk">
        <FindingGrid findings={data.findings} />
      </Surface>
    </div>
  );
}

export function OptimizeWorkflowView({ data }: OptimizeViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Optimization plan workflow">
      <WorkflowHeader
        eyebrow="Optimize"
        title="Safety-gated plan"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <div style={twoColumnStyle}>
        <Surface title="Apply flow" eyebrow="Backup before every write">
          <FlowList items={data.applySteps} />
        </Surface>
        <Surface title="Default policy" eyebrow="Safe only">
          <DefinitionGrid
            items={[
              ["Safe", "Apply-enabled by default"],
              ["Competitive", "Requires explicit consent"],
              ["Lab", "Advanced opt-in and benchmark framing"],
              ["Blocked", "Education only, no mutation"]
            ]}
          />
        </Surface>
      </div>
      <PlanGroupGrid groups={data.groups} />
    </div>
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
          <Surface
            key={session.id}
            title={session.label}
            eyebrow={`${session.time} - ${session.state}`}
            badge={session.rebootRequired ? "Reboot required" : "No reboot"}
          >
            <p className="workflow-muted">{session.summary}</p>
            <SessionRollbackActions session={session} />
            <RollbackTable items={session.items} />
          </Surface>
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
        <Surface title="Profile states" eyebrow="Backup before mutation">
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

export function BenchmarkWorkflowView({ data }: BenchmarkViewProps) {
  return (
    <div style={viewGridStyle} aria-label="Benchmark comparison workflow">
      <WorkflowHeader
        eyebrow="Benchmarks"
        title="Before and after proof"
        actions={<PlanActionBar actions={data.actions} />}
      />
      <MetricGrid metrics={data.metrics} />
      <div style={twoColumnStyle}>
        <Surface title="1% low comparison" eyebrow="Native frames only">
          <BenchmarkChart points={data.chart} />
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

export function PlanActionBar({ actions }: { actions: PlanAction[] }) {
  return (
    <div className="action-bar" aria-label="Optimization plan actions">
      {actions.map((action) => (
        <button
          className={`button button--${action.variant === "secondary" ? "secondary" : action.variant}`}
          key={action.id}
          title={action.label}
          type="button"
        >
          {action.label}
        </button>
      ))}
    </div>
  );
}

function RollbackActionBar({ sessions }: { sessions: RollbackSession[] }) {
  const hasGpuSession = sessions.some(hasGpuProfileRollback);

  return (
    <div className="action-bar" aria-label="Rollback actions">
      <button className="button button--primary" title="Restore selected session" type="button">
        Restore all
      </button>
      {hasGpuSession ? (
        <button className="button button--secondary" title="Restore NVIDIA profile backup" type="button">
          Restore GPU profiles
        </button>
      ) : null}
      <button className="button button--ghost" title="Export rollback audit" type="button">
        Export audit
      </button>
    </div>
  );
}

function SessionRollbackActions({ session }: { session: RollbackSession }) {
  const hasGpuSession = hasGpuProfileRollback(session);

  return (
    <div className="rollback-actions" aria-label={`${session.label} rollback actions`}>
      <button
        className="button button--secondary"
        title={`Restore all changes from ${session.label}`}
        type="button"
      >
        Restore all
      </button>
      {hasGpuSession ? (
        <button
          className="button button--ghost"
          title={`Restore GPU profiles from ${session.label}`}
          type="button"
        >
          Restore GPU profiles
        </button>
      ) : null}
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
        <article className="metric-tile" data-tone={metric.tone} key={metric.id}>
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
          <small>{metric.detail}</small>
        </article>
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
      <div>
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
            Risk: {finding.risk}. {finding.detail}
          </small>
        </article>
      ))}
    </div>
  );
}

function PlanGroupGrid({ groups }: { groups: PlanGroup[] }) {
  return (
    <div style={viewGridStyle}>
      {groups.map((group) => (
        <Surface
          key={group.id}
          title={group.label}
          eyebrow={group.summary}
          badge={group.applyEnabled ? "Default apply" : "Review required"}
        >
          <PlanTable tweaks={group.tweaks} tone={group.tone} />
        </Surface>
      ))}
    </div>
  );
}

function PlanTable({ tweaks, tone }: { tweaks: PlanTweak[]; tone: WorkflowTone }) {
  return (
    <div className="bucket-table" role="table" aria-label="Plan tweaks">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Change</span>
        <span role="columnheader">Impact</span>
        <span role="columnheader">Risk</span>
        <span role="columnheader">Rollback</span>
      </div>
      {tweaks.map((tweak) => (
        <div className="bucket-row" data-tone={tone} role="row" key={tweak.id}>
          <span role="cell">
            <strong>{tweak.change}</strong>
            <small>{tweak.why}</small>
          </span>
          <span role="cell">
            {tweak.expectedImpact}
            <small>Confidence: {tweak.confidence}</small>
          </span>
          <span role="cell">
            {tweak.risk}
            <small>Reboot: {tweak.reboot}</small>
          </span>
          <span role="cell">{tweak.rollback}</span>
        </div>
      ))}
    </div>
  );
}

function RollbackTable({ items }: { items: RollbackItem[] }) {
  return (
    <div className="bucket-table" role="table" aria-label="Rollback values">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Item</span>
        <span role="columnheader">Before</span>
        <span role="columnheader">After</span>
        <span role="columnheader">Rollback</span>
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
  borderRadius: "999px",
  background: "#202b37",
  border: "1px solid #344252"
};

const progressBarStyle: CSSProperties = {
  display: "block",
  height: "100%",
  background: "#27d7ff"
};

const findingGridStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 15rem), 1fr))",
  gap: "0.75rem"
};
