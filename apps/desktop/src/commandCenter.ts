export type NavigationItem = {
  id: string;
  label: string;
  group?: string;
};

export type MetricTone = "active" | "success" | "warning" | "danger" | "neutral" | "violet";

export type CommandMetric = {
  label: string;
  value: string;
  detail: string;
  tone: MetricTone;
};

export type ModeOption = {
  id: "safe" | "competitive" | "lab";
  label: string;
  summary: string;
  tone: MetricTone;
};

export type FlowStepState = "complete" | "active" | "pending";

export type FlowStep = {
  label: string;
  state: FlowStepState;
  detail: string;
};

export type PlanBucket = {
  label: string;
  count: number;
  risk: string;
  rollback: string;
  reboot: string;
  tone: MetricTone;
};

export type StatusItem = {
  label: string;
  value: string;
  tone: MetricTone;
};

export type SessionEvent = {
  time: string;
  label: string;
  detail: string;
  tone: MetricTone;
};

export const navigationItems: NavigationItem[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "scan", label: "Scan" },
  { id: "optimize", label: "Optimize" },
  { id: "power", label: "Power" },
  { id: "nvidia", label: "NVIDIA" },
  { id: "pubg", label: "PUBG", group: "Games" },
  { id: "benchmarks", label: "Benchmarks" },
  { id: "rollback", label: "Rollback" },
  { id: "settings", label: "Settings" }
];

export const commandMetrics: CommandMetric[] = [
  {
    label: "Readiness",
    value: "84",
    detail: "7 findings queued",
    tone: "active"
  },
  {
    label: "Mode",
    value: "Safe",
    detail: "Competitive locked",
    tone: "success"
  },
  {
    label: "Power",
    value: "Balanced",
    detail: "AC profile active",
    tone: "neutral"
  },
  {
    label: "GPU driver",
    value: "Current",
    detail: "NVIDIA 551.86",
    tone: "success"
  },
  {
    label: "PUBG",
    value: "Ready",
    detail: "BattlEye clear",
    tone: "success"
  },
  {
    label: "Benchmark",
    value: "+11.8%",
    detail: "1% low delta",
    tone: "violet"
  },
  {
    label: "Rollback",
    value: "Available",
    detail: "5 snapshots",
    tone: "warning"
  },
  {
    label: "Trust",
    value: "Signed",
    detail: "Liiiraa channel",
    tone: "active"
  }
];

export const modeOptions: ModeOption[] = [
  {
    id: "safe",
    label: "Safe",
    summary: "Rollback-backed system changes only",
    tone: "success"
  },
  {
    id: "competitive",
    label: "Competitive",
    summary: "Requires disclosure for tradeoffs",
    tone: "warning"
  },
  {
    id: "lab",
    label: "Lab",
    summary: "Benchmark-gated experiments",
    tone: "violet"
  }
];

export const flowSteps: FlowStep[] = [
  {
    label: "Scan",
    state: "complete",
    detail: "Read-only hardware and Windows state"
  },
  {
    label: "Plan",
    state: "active",
    detail: "Safe changes selected, Competitive disclosed"
  },
  {
    label: "Apply",
    state: "pending",
    detail: "Backup before every write"
  },
  {
    label: "Verify",
    state: "pending",
    detail: "Confirm values and reboot markers"
  },
  {
    label: "Benchmark",
    state: "pending",
    detail: "Compare FPS lows and frametime"
  },
  {
    label: "Rollback",
    state: "pending",
    detail: "Session restore stays available"
  }
];

export const planBuckets: PlanBucket[] = [
  {
    label: "Safe",
    count: 9,
    risk: "Low",
    rollback: "Full",
    reboot: "1 maybe",
    tone: "success"
  },
  {
    label: "Competitive",
    count: 4,
    risk: "Medium",
    rollback: "Full",
    reboot: "2 required",
    tone: "warning"
  },
  {
    label: "Lab",
    count: 2,
    risk: "High",
    rollback: "Manual review",
    reboot: "Blocked",
    tone: "violet"
  },
  {
    label: "Blocked",
    count: 3,
    risk: "Denied",
    rollback: "N/A",
    reboot: "N/A",
    tone: "danger"
  }
];

export const statusStripItems: StatusItem[] = [
  { label: "Auth", value: "Local", tone: "neutral" },
  { label: "Agent", value: "Idle", tone: "active" },
  { label: "Backups", value: "Ready", tone: "success" },
  { label: "Updater", value: "Signed", tone: "active" }
];

export const sessionEvents: SessionEvent[] = [
  {
    time: "09:42",
    label: "Snapshot captured",
    detail: "Power, Game Bar, NVIDIA profile",
    tone: "success"
  },
  {
    time: "09:44",
    label: "Plan generated",
    detail: "13 apply candidates, 3 denied",
    tone: "active"
  },
  {
    time: "09:47",
    label: "PUBG check",
    detail: "BattlEye not running, config readable",
    tone: "warning"
  }
];
