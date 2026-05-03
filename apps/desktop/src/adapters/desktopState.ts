import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import type {
  OptimizationWorkflow,
  OptimizationWorkflowTone
} from "../../../../packages/ui/src/optimizationWorkflow.js";
import { tOptimizer, type OptimizerLocaleKey } from "../../../../packages/ui/src/localization";
import { settingsTrust } from "../../../../packages/ui/src/settingsTrust.js";
import type { SettingsTrustData } from "../../../../packages/ui/src/settingsTrust.js";
import {
  applyPrivacyConsentToSettings,
  buildPrivacyConsentGateSummary,
  createDefaultPrivacyConsentState,
  evaluateDesktopPrivacyGate,
  type PrivacyConsentGateSummary,
  type PrivacyGateResult
} from "../privacyConsent";

export type DesktopRouteId =
  | "benchmarks"
  | "dashboard"
  | "nvidia"
  | "optimize"
  | "power"
  | "pubg"
  | "rollback"
  | "scan"
  | "settings";

export type DesktopNavigationIconName =
  | "activity"
  | "bar-chart"
  | "crosshair"
  | "gamepad"
  | "gauge"
  | "gpu"
  | "power"
  | "rollback"
  | "rocket"
  | "scan"
  | "settings";

export type DesktopAdapterSource = "tauri-ipc" | "typed-mock";
export type DesktopSemanticTone = OptimizationWorkflowTone | "benchmark" | "rollback" | "trust";

export type DesktopNavigationItem = {
  id: DesktopRouteId;
  labelKey: OptimizerLocaleKey;
  label: string;
  icon: DesktopNavigationIconName;
  summaryKey: OptimizerLocaleKey;
  summary: string;
  tone: DesktopSemanticTone;
  groupKey?: OptimizerLocaleKey;
  group?: string;
};

export type DesktopStatusStripItem = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: DesktopSemanticTone;
};

export type DesktopAdapterDescriptor = {
  id: string;
  label: string;
  source: DesktopAdapterSource;
  state: string;
  detail: string;
  tone: DesktopSemanticTone;
};

export type DesktopRouteInspector = {
  title: string;
  eyebrow: string;
  summary: string;
  tone: DesktopSemanticTone;
  facts: Array<[string, string]>;
  actions: string[];
};

export type PubgLaunchOption = {
  id: string;
  token: string;
  reason: string;
  recommendation: string;
  backup: string;
  tone: OptimizationWorkflowTone;
};

export type DesktopPubgData = OptimizationWorkflow["gaming"]["pubg"] & {
  launchOptions: PubgLaunchOption[];
};

export type DesktopTrustPayload = {
  benchmarkSyncGate: PrivacyGateResult;
  consentGates: PrivacyConsentGateSummary[];
  settings: SettingsTrustData;
};

export type DesktopUpdateStatus = {
  channel: string;
  signature: string;
  state: string;
  transport: string;
};

export type DesktopHardwareStatus = {
  benchmarkDelta: string;
  gpuDriver: string;
  powerPlan: string;
  pubgReadiness: string;
};

export type DesktopCommandCenterRoutes = {
  benchmarks: OptimizationWorkflow["gaming"]["benchmarks"];
  dashboard: OptimizationWorkflow["dashboard"];
  nvidia: OptimizationWorkflow["gaming"]["nvidia"];
  optimize: OptimizationWorkflow["optimize"];
  power: OptimizationWorkflow["gaming"]["power"];
  pubg: DesktopPubgData;
  rollback: OptimizationWorkflow["rollback"];
  scan: OptimizationWorkflow["scan"];
  settings: SettingsTrustData;
  settingsConsentGates: PrivacyConsentGateSummary[];
};

export type DesktopCommandCenterState = {
  adapters: DesktopAdapterDescriptor[];
  inspector: Record<DesktopRouteId, DesktopRouteInspector>;
  navigation: DesktopNavigationItem[];
  privacy: {
    benchmarkSyncGate: PrivacyGateResult;
  };
  routes: DesktopCommandCenterRoutes;
  statusStrip: DesktopStatusStripItem[];
};

export const desktopNavigationItems: DesktopNavigationItem[] = [
  {
    id: "dashboard",
    labelKey: "routes.dashboard.label",
    label: tOptimizer("routes.dashboard.label"),
    icon: "gauge",
    summaryKey: "navigation.dashboard.summary",
    summary: tOptimizer("navigation.dashboard.summary"),
    tone: "active"
  },
  {
    id: "scan",
    labelKey: "routes.scan.label",
    label: tOptimizer("routes.scan.label"),
    icon: "scan",
    summaryKey: "navigation.scan.summary",
    summary: tOptimizer("navigation.scan.summary"),
    tone: "active"
  },
  {
    id: "optimize",
    labelKey: "routes.optimize.label",
    label: tOptimizer("routes.optimize.label"),
    icon: "rocket",
    summaryKey: "navigation.optimize.summary",
    summary: tOptimizer("navigation.optimize.summary"),
    tone: "success"
  },
  {
    id: "power",
    labelKey: "routes.power.label",
    label: tOptimizer("routes.power.label"),
    icon: "power",
    summaryKey: "navigation.power.summary",
    summary: tOptimizer("navigation.power.summary"),
    tone: "warning"
  },
  {
    id: "nvidia",
    labelKey: "routes.nvidia.label",
    label: tOptimizer("routes.nvidia.label"),
    icon: "gpu",
    summaryKey: "navigation.nvidia.summary",
    summary: tOptimizer("navigation.nvidia.summary"),
    tone: "active"
  },
  {
    id: "pubg",
    labelKey: "routes.pubg.label",
    label: tOptimizer("routes.pubg.label"),
    icon: "gamepad",
    summaryKey: "navigation.pubg.summary",
    summary: tOptimizer("navigation.pubg.summary"),
    tone: "success",
    groupKey: "navigation.pubg.group",
    group: tOptimizer("navigation.pubg.group")
  },
  {
    id: "benchmarks",
    labelKey: "routes.benchmarks.label",
    label: tOptimizer("routes.benchmarks.label"),
    icon: "bar-chart",
    summaryKey: "navigation.benchmarks.summary",
    summary: tOptimizer("navigation.benchmarks.summary"),
    tone: "benchmark"
  },
  {
    id: "rollback",
    labelKey: "routes.rollback.label",
    label: tOptimizer("routes.rollback.label"),
    icon: "rollback",
    summaryKey: "navigation.rollback.summary",
    summary: tOptimizer("navigation.rollback.summary"),
    tone: "rollback"
  },
  {
    id: "settings",
    labelKey: "routes.settings.label",
    label: tOptimizer("routes.settings.label"),
    icon: "settings",
    summaryKey: "navigation.settings.summary",
    summary: tOptimizer("navigation.settings.summary"),
    tone: "trust"
  }
];

export const pubgLaunchOptionCleanup: PubgLaunchOption[] = [
  {
    id: "use-all-cores",
    token: "-USEALLAVAILABLECORES",
    reason: "Windows already schedules PUBG across available cores.",
    recommendation: "Remove without adding a replacement flag.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  },
  {
    id: "malloc-system",
    token: "-malloc=system",
    reason: "Allocator forcing is legacy and unsupported for current PUBG.",
    recommendation: "Remove without adding a replacement flag.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  },
  {
    id: "priority-high",
    token: "-high",
    reason: "Priority forcing can starve system work and stays blocked by policy.",
    recommendation: "Remove and keep priority changes out of default plans.",
    backup: "Steam value captured before cleanup.",
    tone: "danger"
  },
  {
    id: "dx11-force",
    token: "-dx11",
    reason: "Renderer forcing should be benchmarked per machine.",
    recommendation: "Remove and use the DirectX benchmark flow instead.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  }
];

export const desktopStateAdapters = {
  scan: {
    id: "scan",
    label: "Scan adapter",
    source: "typed-mock",
    state: tOptimizer("statusStrip.agent.valueReady"),
    detail: "Read-only scan IPC can replace this typed fixture without changing route views.",
    tone: "active",
    read: () => optimizationWorkflow.scan
  },
  plan: {
    id: "plan",
    label: "Plan adapter",
    source: "typed-mock",
    state: "Safety gated",
    detail: "Safe, Competitive, Lab, and Blocked groups share one typed contract.",
    tone: "success",
    read: () => optimizationWorkflow.optimize
  },
  rollback: {
    id: "rollback",
    label: "Rollback adapter",
    source: "typed-mock",
    state: "Snapshots visible",
    detail: "Session data keeps restore actions and reboot markers in view.",
    tone: "rollback",
    read: () => optimizationWorkflow.rollback
  },
  benchmark: {
    id: "benchmark",
    label: "Benchmark adapter",
    source: "typed-mock",
    state: "Comparison ready",
    detail: "FPS lows, p95 frametime, metadata, and variance warnings stay together.",
    tone: "benchmark",
    read: () => optimizationWorkflow.gaming.benchmarks
  },
  trust: {
    id: "trust",
    label: "Trust adapter",
    source: "typed-mock",
    state: "Local first",
    detail: "Privacy consent gates and signed update state are exposed from one boundary.",
    tone: "trust",
    read: (): DesktopTrustPayload => {
      const consent = createDefaultPrivacyConsentState();

      return {
        benchmarkSyncGate: evaluateDesktopPrivacyGate({ consent, kind: "benchmark-sync" }),
        consentGates: buildPrivacyConsentGateSummary(consent),
        settings: applyPrivacyConsentToSettings(settingsTrust, consent)
      };
    }
  },
  update: {
    id: "update",
    label: "Update adapter",
    source: "typed-mock",
    state: "Signed",
    detail: "Tauri updater status can hydrate channel, transport, and signature fields.",
    tone: "trust",
    read: (data: SettingsTrustData = settingsTrust): DesktopUpdateStatus => {
      const selectedChannel = data.updateChannels.find((channel) => channel.selected);
      const metadata = new Map(data.updateMetadata);

      return {
        channel: selectedChannel?.label ?? "Stable",
        signature: metadata.get("Signature") ?? "Required",
        state: selectedChannel?.state ?? "Selected",
        transport: metadata.get("Transport") ?? "HTTPS only"
      };
    }
  },
  hardware: {
    id: "hardware",
    label: "Hardware adapter",
    source: "typed-mock",
    state: "Detected",
    detail: "Power, GPU, PUBG, and benchmark readiness are normalized for the shell.",
    tone: "success",
    read: (): DesktopHardwareStatus => ({
      benchmarkDelta: optimizationWorkflow.dashboard.lastBenchmarkDelta,
      gpuDriver: optimizationWorkflow.dashboard.driverState,
      powerPlan: optimizationWorkflow.dashboard.activePowerPlan,
      pubgReadiness: optimizationWorkflow.dashboard.pubgReadiness
    })
  }
} as const;

export const desktopCommandCenterState = createDesktopCommandCenterState();

export function isDesktopRouteId(value: string): value is DesktopRouteId {
  return desktopNavigationItems.some((item) => item.id === value);
}

export function createDesktopCommandCenterState(): DesktopCommandCenterState {
  const scan = desktopStateAdapters.scan.read();
  const plan = desktopStateAdapters.plan.read();
  const rollback = desktopStateAdapters.rollback.read();
  const benchmark = desktopStateAdapters.benchmark.read();
  const trust = desktopStateAdapters.trust.read();
  const update = desktopStateAdapters.update.read(trust.settings);
  const hardware = desktopStateAdapters.hardware.read();
  const pubg: DesktopPubgData = {
    ...optimizationWorkflow.gaming.pubg,
    launchOptions: pubgLaunchOptionCleanup
  };

  const routes: DesktopCommandCenterRoutes = {
    benchmarks: benchmark,
    dashboard: optimizationWorkflow.dashboard,
    nvidia: optimizationWorkflow.gaming.nvidia,
    optimize: plan,
    power: optimizationWorkflow.gaming.power,
    pubg,
    rollback,
    scan,
    settings: trust.settings,
    settingsConsentGates: trust.consentGates
  };

  return {
    adapters: [
      describeAdapter(desktopStateAdapters.scan),
      describeAdapter(desktopStateAdapters.plan),
      describeAdapter(desktopStateAdapters.rollback),
      describeAdapter(desktopStateAdapters.benchmark),
      describeAdapter(desktopStateAdapters.trust),
      describeAdapter(desktopStateAdapters.update),
      describeAdapter(desktopStateAdapters.hardware)
    ],
    inspector: createRouteInspectors(routes, update, hardware),
    navigation: desktopNavigationItems,
    privacy: {
      benchmarkSyncGate: trust.benchmarkSyncGate
    },
    routes,
    statusStrip: [
      {
        id: "scan",
        label: tOptimizer("statusStrip.scan.label"),
        value: tOptimizer("statusStrip.scan.value", { percent: scan.progress.percent }),
        detail: scan.progress.current,
        tone: "active"
      },
      {
        id: "agent",
        label: tOptimizer("statusStrip.agent.label"),
        value: desktopStateAdapters.scan.state,
        detail: tOptimizer("statusStrip.agent.detail"),
        tone: "success"
      },
      {
        id: "backups",
        label: tOptimizer("statusStrip.backups.label"),
        value: tOptimizer("statusStrip.backups.value", { count: rollback.sessions.length }),
        detail: optimizationWorkflow.dashboard.rollbackAvailability,
        tone: "rollback"
      },
      {
        id: "updater",
        label: tOptimizer("statusStrip.updater.label"),
        value: update.signature,
        detail: tOptimizer("statusStrip.updater.detail", {
          channel: update.channel,
          transport: update.transport
        }),
        tone: "trust"
      }
    ]
  };
}

function describeAdapter(adapter: DesktopAdapterDescriptor): DesktopAdapterDescriptor {
  return {
    id: adapter.id,
    label: adapter.label,
    source: adapter.source,
    state: adapter.state,
    detail: adapter.detail,
    tone: adapter.tone
  };
}

function createRouteInspectors(
  routes: DesktopCommandCenterRoutes,
  update: DesktopUpdateStatus,
  hardware: DesktopHardwareStatus
): Record<DesktopRouteId, DesktopRouteInspector> {
  return {
    benchmarks: {
      title: "Performance",
      eyebrow: routes.benchmarks.summary.confidence,
      summary: routes.benchmarks.summary.detail,
      tone: "benchmark",
      facts: [
        ["Decision", routes.benchmarks.summary.decision],
        ["Variance", routes.benchmarks.summary.varianceBand],
        ["Metadata", `${routes.benchmarks.metadata.length} fields`]
      ],
      actions: [
        tOptimizer("actions.captureBefore"),
        tOptimizer("actions.compareAfter"),
        tOptimizer("actions.exportReport")
      ]
    },
    dashboard: {
      title: "Dashboard",
      eyebrow: `${routes.dashboard.readinessScore}/100 readiness`,
      summary: "System score, active mode, rollback, and trust in one premium control deck.",
      tone: "active",
      facts: [
        ["Mode", routes.dashboard.activeMode],
        ["Power", hardware.powerPlan],
        ["GPU", hardware.gpuDriver],
        ["PUBG", hardware.pubgReadiness],
        ["Benchmark", hardware.benchmarkDelta],
        ["Trust", routes.dashboard.trustState]
      ],
      actions: [
        tOptimizer("actions.startScan"),
        tOptimizer("actions.reviewPlan"),
        tOptimizer("actions.openRollback")
      ]
    },
    nvidia: {
      title: "GPU Control",
      eyebrow: `${routes.nvidia.profiles.length} profiles`,
      summary: "Driver state, PUBG profile staging, backup requirements, and frame cap policy.",
      tone: "active",
      facts: [
        ["Driver", routes.nvidia.metrics.find((metric) => metric.id === "driver")?.value ?? "Unknown"],
        ["Refresh", routes.nvidia.metrics.find((metric) => metric.id === "display")?.value ?? "Unknown"],
        ["Profile API", routes.nvidia.metrics.find((metric) => metric.id === "nvapi")?.value ?? "Unknown"]
      ],
      actions: [
        tOptimizer("actions.backupProfiles"),
        tOptimizer("actions.stagePubgProfile"),
        tOptimizer("actions.openBenchmark")
      ]
    },
    optimize: {
      title: "Smart Boost",
      eyebrow: `${routes.optimize.groups.length} buckets`,
      summary: "Apply safe tweaks first; competitive and lab changes stay review-gated.",
      tone: "success",
      facts: routes.optimize.groups.map((group) => [
        group.label,
        `${group.tweaks.length} changes, ${group.applyEnabled ? "apply enabled" : "review required"}`
      ]),
      actions: routes.optimize.actions.map((action) => action.label)
    },
    power: {
      title: "Power policy",
      eyebrow: `${routes.power.plans.length} plans`,
      summary: "Scoped plan changes keep defaults visible and rollback attached.",
      tone: "warning",
      facts: routes.power.plans.map((plan) => [plan.label, `${plan.state}, rollback: ${plan.rollback}`]),
      actions: routes.power.actions.map((action) => action.label)
    },
    pubg: {
      title: "Game Mode",
      eyebrow: "Anti-cheat boundary",
      summary: "Detection, launch option cleanup, DX benchmark choice, and NVIDIA link live here.",
      tone: "success",
      facts: [
        ["Detections", `${routes.pubg.detections.length} checks`],
        ["Launch cleanup", `${routes.pubg.launchOptions.length} legacy flags`],
        ["DX results", `${routes.pubg.dxBenchmark.results.length} measured modes`],
        ["Checklist", `${routes.pubg.checklist.length} items`]
      ],
      actions: routes.pubg.actions.map((action) => action.label)
    },
    rollback: {
      title: "Recovery",
      eyebrow: `${routes.rollback.sessions.length} sessions ready`,
      summary: "Every visible session keeps changed values, reboot markers, and restore actions together.",
      tone: "rollback",
      facts: routes.rollback.sessions.map((session) => [
        session.label,
        `${session.state}, ${session.rebootRequired ? "reboot required" : "no reboot"}`
      ]),
      actions: [
        tOptimizer("actions.restoreAll"),
        tOptimizer("actions.restoreGpuProfiles"),
        tOptimizer("actions.exportAudit")
      ]
    },
    scan: {
      title: "Smart Scan",
      eyebrow: routes.scan.progress.label,
      summary: routes.scan.progress.current,
      tone: "active",
      facts: [
        ["Progress", `${routes.scan.progress.percent}%`],
        ["Scopes", `${routes.scan.scopes.filter((scope) => scope.checked).length}/${routes.scan.scopes.length}`],
        ["Findings", `${routes.scan.findings.length} visible`]
      ],
      actions: [
        tOptimizer("actions.startScan"),
        tOptimizer("actions.cancelScan"),
        tOptimizer("actions.generatePlan")
      ]
    },
    settings: {
      title: "Settings",
      eyebrow: routes.settings.signature,
      summary: "Privacy, telemetry, update channel, signed metadata, and local data controls.",
      tone: "trust",
      facts: [
        ["Telemetry", routes.settings.privacyControls.find((control) => control.id === "telemetry")?.value ?? "Off"],
        ["Channel", update.channel],
        ["Updater", update.signature],
        ["Transport", update.transport]
      ],
      actions: [
        tOptimizer("actions.checkUpdates"),
        tOptimizer("actions.exportLocalData"),
        tOptimizer("actions.openDataFolder")
      ]
    }
  };
}
