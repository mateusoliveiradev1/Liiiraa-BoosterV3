export type OptimizationWorkflowTone = "active" | "danger" | "lab" | "neutral" | "success" | "warning";

export type OptimizationWorkflowButtonVariant = "ghost" | "primary" | "secondary";

export interface OptimizationModeOption {
  id: "safe" | "competitive" | "lab";
  label: string;
  detail: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowSignal {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowScanScope {
  id: string;
  label: string;
  detail: string;
  checked: boolean;
}

export interface OptimizationWorkflowStep {
  id: string;
  label: string;
  detail: string;
  state: "active" | "complete" | "pending";
}

export interface OptimizationWorkflowFinding {
  id: string;
  group: string;
  risk: string;
  title: string;
  detail: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowAction {
  id: string;
  label: string;
  variant: OptimizationWorkflowButtonVariant;
}

export interface OptimizationWorkflowTweak {
  id: string;
  change: string;
  expectedImpact: string;
  risk: string;
  rollback: string;
  reboot: string;
  confidence: string;
  why: string;
}

export interface OptimizationWorkflowPlanGroup {
  id: "blocked" | "competitive" | "lab" | "safe";
  label: string;
  summary: string;
  tone: OptimizationWorkflowTone;
  applyEnabled: boolean;
  tweaks: OptimizationWorkflowTweak[];
}

export interface OptimizationWorkflowRollbackItem {
  id: string;
  label: string;
  before: string;
  after: string;
  rollback: string;
  state: string;
}

export interface OptimizationWorkflowRollbackSession {
  id: string;
  time: string;
  label: string;
  state: string;
  rebootRequired: boolean;
  summary: string;
  items: OptimizationWorkflowRollbackItem[];
}

export interface OptimizationWorkflowPowerPlan {
  id: string;
  label: string;
  mode: string;
  state: string;
  detail: string;
  rollback: string;
  defaults: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowNvidiaProfile {
  id: string;
  label: string;
  scope: string;
  state: string;
  recommendation: string;
  rollback: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowPubgDxChoice {
  id: string;
  label: string;
  evidence: string;
  rollback: string;
  state: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowPubgDxBenchmarkStep {
  id: string;
  label: string;
  detail: string;
  state: "active" | "complete" | "pending";
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowPubgDxBenchmarkResult {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  pointOnePercentLow: number;
  p95FrameMs: number;
  droppedFrames: number;
  verdict: string;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowPubgDxBenchmark {
  currentMode: string;
  selectedMode: string;
  rationale: string;
  varianceBand: string;
  steps: OptimizationWorkflowPubgDxBenchmarkStep[];
  results: OptimizationWorkflowPubgDxBenchmarkResult[];
  metadata: Array<[string, string]>;
}

export interface OptimizationWorkflowBenchmarkPoint {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  p95FrameMs: number;
  tone: OptimizationWorkflowTone;
}

export interface OptimizationWorkflowGaming {
  power: {
    metrics: OptimizationWorkflowMetric[];
    actions: OptimizationWorkflowAction[];
    plans: OptimizationWorkflowPowerPlan[];
    rules: OptimizationWorkflowSignal[];
  };
  nvidia: {
    metrics: OptimizationWorkflowMetric[];
    actions: OptimizationWorkflowAction[];
    profiles: OptimizationWorkflowNvidiaProfile[];
    policies: OptimizationWorkflowSignal[];
    capLogic: Array<[string, string]>;
  };
  pubg: {
    metrics: OptimizationWorkflowMetric[];
    actions: OptimizationWorkflowAction[];
    detections: OptimizationWorkflowSignal[];
    dxChoices: OptimizationWorkflowPubgDxChoice[];
    dxBenchmark: OptimizationWorkflowPubgDxBenchmark;
    checklist: OptimizationWorkflowSignal[];
  };
  benchmarks: {
    metrics: OptimizationWorkflowMetric[];
    actions: OptimizationWorkflowAction[];
    chart: OptimizationWorkflowBenchmarkPoint[];
    metadata: Array<[string, string]>;
    sessions: OptimizationWorkflowSignal[];
  };
}

export interface OptimizationWorkflow {
  dashboard: {
    readinessScore: number;
    activeMode: string;
    activePowerPlan: string;
    driverState: string;
    pubgReadiness: string;
    lastBenchmarkDelta: string;
    rollbackAvailability: string;
    trustState: string;
    metrics: OptimizationWorkflowMetric[];
    readinessSignals: OptimizationWorkflowSignal[];
  };
  scan: {
    scopes: OptimizationWorkflowScanScope[];
    states: OptimizationWorkflowStep[];
    progress: {
      label: string;
      percent: number;
      current: string;
      completed: string[];
    };
    findings: OptimizationWorkflowFinding[];
  };
  optimize: {
    actions: OptimizationWorkflowAction[];
    groups: OptimizationWorkflowPlanGroup[];
    applySteps: OptimizationWorkflowStep[];
  };
  rollback: {
    sessions: OptimizationWorkflowRollbackSession[];
  };
  gaming: OptimizationWorkflowGaming;
  guardrails: string[];
}

export const optimizationModeOptions: OptimizationModeOption[];
export const optimizationWorkflow: OptimizationWorkflow;

export function assertOptimizationWorkflowSmoke(workflow?: OptimizationWorkflow): void;
export function renderOptimizationWorkflowSmokeHtml(workflow?: OptimizationWorkflow): string;
