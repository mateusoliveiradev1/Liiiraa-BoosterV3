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
  guardrails: string[];
}

export const optimizationModeOptions: OptimizationModeOption[];
export const optimizationWorkflow: OptimizationWorkflow;

export function assertOptimizationWorkflowSmoke(workflow?: OptimizationWorkflow): void;
export function renderOptimizationWorkflowSmokeHtml(workflow?: OptimizationWorkflow): string;
