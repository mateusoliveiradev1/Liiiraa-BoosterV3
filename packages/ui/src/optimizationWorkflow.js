export const optimizationModeOptions = [
  {
    id: "safe",
    label: "Safe",
    detail: "Default reversible changes",
    tone: "success"
  },
  {
    id: "competitive",
    label: "Competitive",
    detail: "Consent required",
    tone: "warning"
  },
  {
    id: "lab",
    label: "Lab",
    detail: "Benchmark gated",
    tone: "lab"
  }
];

export const optimizationWorkflow = {
  dashboard: {
    readinessScore: 84,
    activeMode: "Safe",
    activePowerPlan: "Balanced",
    driverState: "NVIDIA 551.86 current",
    pubgReadiness: "Ready, BattlEye clear",
    lastBenchmarkDelta: "+11.8% 1% low",
    rollbackAvailability: "5 snapshots",
    trustState: "Signed stable channel",
    metrics: [
      {
        id: "readiness",
        label: "Readiness",
        value: "84",
        detail: "7 findings queued",
        tone: "active"
      },
      {
        id: "mode",
        label: "Mode",
        value: "Safe",
        detail: "Competitive locked",
        tone: "success"
      },
      {
        id: "power",
        label: "Power",
        value: "Balanced",
        detail: "AC profile active",
        tone: "neutral"
      },
      {
        id: "driver",
        label: "GPU driver",
        value: "Current",
        detail: "NVIDIA 551.86",
        tone: "success"
      },
      {
        id: "pubg",
        label: "PUBG",
        value: "Ready",
        detail: "BattlEye clear",
        tone: "success"
      },
      {
        id: "benchmark",
        label: "Benchmark",
        value: "+11.8%",
        detail: "1% low delta",
        tone: "active"
      },
      {
        id: "rollback",
        label: "Rollback",
        value: "Available",
        detail: "5 snapshots",
        tone: "warning"
      },
      {
        id: "trust",
        label: "Trust",
        value: "Signed",
        detail: "Liiiraa channel",
        tone: "success"
      }
    ],
    readinessSignals: [
      {
        id: "restore-point",
        label: "Restore point",
        value: "Available",
        detail: "System Restore has free space",
        tone: "success"
      },
      {
        id: "pending-reboot",
        label: "Pending reboot",
        value: "Clear",
        detail: "High-risk apply is not blocked",
        tone: "success"
      },
      {
        id: "driver-age",
        label: "Driver age",
        value: "Fresh",
        detail: "Game Ready branch is current",
        tone: "success"
      },
      {
        id: "background-load",
        label: "Background load",
        value: "Moderate",
        detail: "3 startup items worth review",
        tone: "warning"
      }
    ]
  },
  scan: {
    scopes: [
      {
        id: "system",
        label: "System inventory",
        detail: "OS, CPU, RAM, disks, active power plan",
        checked: true
      },
      {
        id: "graphics",
        label: "Graphics path",
        detail: "Driver, display refresh, VRR, HAGS, overlays",
        checked: true
      },
      {
        id: "gaming",
        label: "Gaming surface",
        detail: "Game Mode, captures, PUBG path, BattlEye state",
        checked: true
      },
      {
        id: "network-storage",
        label: "Network and storage",
        detail: "Adapter power, TRIM, DirectStorage readiness",
        checked: false
      }
    ],
    states: [
      {
        id: "idle",
        label: "Idle",
        detail: "Ready for read-only scan",
        state: "complete"
      },
      {
        id: "scanning",
        label: "Scanning",
        detail: "Inventory and guardrails",
        state: "active"
      },
      {
        id: "partial",
        label: "Partial result",
        detail: "Graphics checks complete",
        state: "pending"
      },
      {
        id: "complete",
        label: "Complete",
        detail: "Plan can be generated",
        state: "pending"
      },
      {
        id: "failed",
        label: "Failed",
        detail: "Retry keeps prior safe findings",
        state: "pending"
      },
      {
        id: "cancelled",
        label: "Cancelled",
        detail: "No system changes were made",
        state: "pending"
      }
    ],
    progress: {
      label: "Graphics path",
      percent: 62,
      current: "Reading driver, VRR, and overlay state",
      completed: ["OS inventory", "CPU topology", "Active power plan", "PUBG process check"]
    },
    findings: [
      {
        id: "capture",
        group: "High impact",
        risk: "Low",
        title: "Background capture is enabled",
        detail: "Safe Game DVR change can reduce recording overhead.",
        tone: "success"
      },
      {
        id: "power",
        group: "High impact",
        risk: "Low",
        title: "Balanced plan is active",
        detail: "Liiiraa performance plan can be staged with rollback.",
        tone: "success"
      },
      {
        id: "driver",
        group: "Moderate impact",
        risk: "Low",
        title: "Driver branch is current",
        detail: "No driver replacement recommendation required.",
        tone: "neutral"
      },
      {
        id: "vbs",
        group: "Tradeoff",
        risk: "Medium",
        title: "VBS and HVCI are visible",
        detail: "Competitive plan requires explicit security disclosure.",
        tone: "warning"
      },
      {
        id: "defender",
        group: "Blocked",
        risk: "Critical",
        title: "Global Defender disable denied",
        detail: "Use narrow verified exclusions or scheduling only.",
        tone: "danger"
      }
    ]
  },
  optimize: {
    actions: [
      {
        id: "apply-safe",
        label: "Apply safe only",
        variant: "primary"
      },
      {
        id: "include-competitive",
        label: "Include competitive",
        variant: "secondary"
      },
      {
        id: "inspect-lab",
        label: "Inspect lab",
        variant: "secondary"
      },
      {
        id: "export-plan",
        label: "Export plan",
        variant: "secondary"
      },
      {
        id: "cancel",
        label: "Cancel",
        variant: "ghost"
      }
    ],
    groups: [
      {
        id: "safe",
        label: "Safe",
        summary: "Default reversible changes",
        tone: "success",
        applyEnabled: true,
        tweaks: [
          {
            id: "game.capture.background.off",
            change: "Disable background recording when unused",
            expectedImpact: "Lower capture overhead",
            risk: "Low",
            rollback: "HKCU/HKLM values backed up",
            reboot: "No",
            confidence: "High",
            why: "Capture is enabled and no recording workflow was detected."
          },
          {
            id: "power.plan.liiiraa-balanced",
            change: "Create Liiiraa Boost - Balanced plan",
            expectedImpact: "Stable reversible baseline",
            risk: "Low",
            rollback: "Restore previous active scheme",
            reboot: "No",
            confidence: "High",
            why: "Current plan can be duplicated before performance tuning."
          },
          {
            id: "game.mode.verify",
            change: "Verify Game Mode is enabled",
            expectedImpact: "Windows gaming scheduling",
            risk: "Low",
            rollback: "Back up user setting",
            reboot: "No",
            confidence: "Medium",
            why: "Game Mode is a supported gaming surface."
          }
        ]
      },
      {
        id: "competitive",
        label: "Competitive",
        summary: "Explicit performance tradeoffs",
        tone: "warning",
        applyEnabled: false,
        tweaks: [
          {
            id: "security.hvci.tradeoff",
            change: "Plan Memory Integrity off/on comparison",
            expectedImpact: "Possible latency and FPS uplift",
            risk: "Medium",
            rollback: "Restore previous HVCI state",
            reboot: "Required",
            confidence: "Medium",
            why: "Only valid when the user accepts the security tradeoff."
          },
          {
            id: "game.hags.benchmark",
            change: "Benchmark HAGS before and after",
            expectedImpact: "Hardware-dependent frametime change",
            risk: "Medium",
            rollback: "Restore previous graphics setting",
            reboot: "Maybe",
            confidence: "Medium",
            why: "The matrix rejects a universal HAGS default."
          }
        ]
      },
      {
        id: "lab",
        label: "Lab",
        summary: "Advanced benchmark experiments",
        tone: "lab",
        applyEnabled: false,
        tweaks: [
          {
            id: "net.rsc.profile",
            change: "Adapter-specific RSC experiment",
            expectedImpact: "Latency or throughput diagnosis",
            risk: "High",
            rollback: "Adapter state backup",
            reboot: "Adapter restart",
            confidence: "Low",
            why: "Only useful with evidence from VPN, capture, or driver issues."
          }
        ]
      },
      {
        id: "blocked",
        label: "Blocked",
        summary: "Education only",
        tone: "danger",
        applyEnabled: false,
        tweaks: [
          {
            id: "blocked.defender.disable",
            change: "Deny global Defender disable",
            expectedImpact: "Not applicable",
            risk: "Critical",
            rollback: "No mutation allowed",
            reboot: "N/A",
            confidence: "High",
            why: "The matrix treats this as a security regression."
          },
          {
            id: "blocked.anticheat-tamper",
            change: "Deny BattlEye and PUBG memory tamper",
            expectedImpact: "Not applicable",
            risk: "Critical",
            rollback: "No mutation allowed",
            reboot: "N/A",
            confidence: "High",
            why: "Anti-cheat safety is non-negotiable."
          }
        ]
      }
    ],
    applySteps: [
      {
        id: "backup",
        label: "Backup",
        state: "complete",
        detail: "Snapshot power, capture, NVIDIA, and registry values"
      },
      {
        id: "apply",
        label: "Apply",
        state: "active",
        detail: "Writing safe changes only"
      },
      {
        id: "verify",
        label: "Verify",
        state: "pending",
        detail: "Read back values and reboot markers"
      },
      {
        id: "benchmark",
        label: "Benchmark prompt",
        state: "pending",
        detail: "Capture before/after metadata"
      },
      {
        id: "rollback",
        label: "Rollback if needed",
        state: "pending",
        detail: "Restore all changed values for the session"
      }
    ]
  },
  rollback: {
    sessions: [
      {
        id: "session-2026-04-30-0947",
        time: "09:47",
        label: "Safe gaming baseline",
        state: "Ready",
        rebootRequired: false,
        summary: "5 safe changes, 0 failed",
        items: [
          {
            id: "power.plan.liiiraa-balanced",
            label: "Active power plan",
            before: "Balanced",
            after: "Liiiraa Boost - Balanced",
            rollback: "Restore Balanced",
            state: "Ready"
          },
          {
            id: "game.capture.background.off",
            label: "Background capture",
            before: "Enabled",
            after: "Disabled",
            rollback: "Restore capture setting",
            state: "Ready"
          },
          {
            id: "game.mode.verify",
            label: "Game Mode",
            before: "Off",
            after: "On",
            rollback: "Restore Off",
            state: "Ready"
          }
        ]
      },
      {
        id: "session-2026-04-29-2131",
        time: "Yesterday",
        label: "NVIDIA profile rehearsal",
        state: "Needs review",
        rebootRequired: true,
        summary: "2 competitive changes, 1 reboot marker",
        items: [
          {
            id: "nvidia.global.profile",
            label: "Global profile",
            before: "Driver default",
            after: "Liiiraa Boost - Global Performance",
            rollback: "Import profile backup",
            state: "Ready"
          },
          {
            id: "nvidia.max-frame-rate.vrr",
            label: "Frame cap",
            before: "Off",
            after: "237 FPS cap",
            rollback: "Restore Off",
            state: "Ready"
          }
        ]
      }
    ]
  },
  guardrails: [
    "Safe changes can be default selected.",
    "Competitive changes require explicit consent.",
    "Lab changes require advanced opt-in and benchmark framing.",
    "Blocked changes are never actionable apply rows.",
    "Every apply step must preserve backup, verify, and rollback visibility."
  ]
};

export function assertOptimizationWorkflowSmoke(workflow = optimizationWorkflow) {
  const requiredMetrics = [
    "Readiness",
    "Mode",
    "Power",
    "GPU driver",
    "PUBG",
    "Benchmark",
    "Rollback",
    "Trust"
  ];
  const metricLabels = new Set(workflow.dashboard.metrics.map((metric) => metric.label));
  const missingMetrics = requiredMetrics.filter((metric) => !metricLabels.has(metric));

  if (missingMetrics.length > 0) {
    throw new Error(`Dashboard metrics missing: ${missingMetrics.join(", ")}`);
  }

  const requiredScanStates = ["idle", "scanning", "partial", "complete", "failed", "cancelled"];
  const scanStates = new Set(workflow.scan.states.map((state) => state.id));
  const missingScanStates = requiredScanStates.filter((state) => !scanStates.has(state));

  if (missingScanStates.length > 0) {
    throw new Error(`Scan states missing: ${missingScanStates.join(", ")}`);
  }

  const requiredGroups = ["safe", "competitive", "lab", "blocked"];
  const groups = new Set(workflow.optimize.groups.map((group) => group.id));
  const missingGroups = requiredGroups.filter((group) => !groups.has(group));

  if (missingGroups.length > 0) {
    throw new Error(`Plan groups missing: ${missingGroups.join(", ")}`);
  }

  const blockedGroup = workflow.optimize.groups.find((group) => group.id === "blocked");
  if (!blockedGroup || blockedGroup.applyEnabled) {
    throw new Error("Blocked recommendations must not be apply-enabled.");
  }

  const unsafeDefaultGroups = workflow.optimize.groups.filter(
    (group) => group.id !== "safe" && group.applyEnabled
  );

  if (unsafeDefaultGroups.length > 0) {
    throw new Error(
      `Only Safe may be apply-enabled by default: ${unsafeDefaultGroups.map((group) => group.id).join(", ")}`
    );
  }

  const requiredApplySteps = ["backup", "apply", "verify", "benchmark", "rollback"];
  const applySteps = new Set(workflow.optimize.applySteps.map((step) => step.id));
  const missingApplySteps = requiredApplySteps.filter((step) => !applySteps.has(step));

  if (missingApplySteps.length > 0) {
    throw new Error(`Apply flow steps missing: ${missingApplySteps.join(", ")}`);
  }
}

export function renderOptimizationWorkflowSmokeHtml(workflow = optimizationWorkflow) {
  const renderMetric = (metric) => `
    <article class="tile" data-tone="${metric.tone}">
      <span>${metric.label}</span>
      <strong>${metric.value}</strong>
      <small>${metric.detail}</small>
    </article>`;

  const renderPlan = (group) => `
    <section class="panel plan" data-tone="${group.tone}">
      <header><span>${group.label}</span><strong>${group.summary}</strong></header>
      ${group.tweaks
        .map(
          (tweak) => `
          <div class="row">
            <b>${tweak.change}</b>
            <span>${tweak.expectedImpact}</span>
            <span>${tweak.risk}</span>
            <span>${tweak.rollback}</span>
          </div>`
        )
        .join("")}
    </section>`;

  const renderRollback = (session) => `
    <section class="panel">
      <header><span>${session.time}</span><strong>${session.label}</strong></header>
      <p>${session.summary}${session.rebootRequired ? " - reboot required" : ""}</p>
      ${session.items
        .map(
          (item) => `
          <div class="row">
            <b>${item.label}</b>
            <span>${item.before}</span>
            <span>${item.after}</span>
            <span>${item.rollback}</span>
          </div>`
        )
        .join("")}
    </section>`;

  return `<!doctype html>
  <html lang="en">
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <title>Liiiraa Optimization Workflow Smoke</title>
      <style>
        * { box-sizing: border-box; }
        body {
          margin: 0;
          min-width: 320px;
          background: #0b0f14;
          color: #f5f8fb;
          font-family: Inter, "Segoe UI", system-ui, sans-serif;
        }
        main { display: grid; gap: 16px; max-width: 1180px; margin: 0 auto; padding: 20px; }
        header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
        h1, h2, p { margin: 0; letter-spacing: 0; }
        h1 { font-size: 30px; line-height: 1.1; }
        h2 { font-size: 18px; }
        .grid { display: grid; gap: 12px; grid-template-columns: repeat(4, minmax(0, 1fr)); }
        .views { display: grid; gap: 14px; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); align-items: start; }
        .panel, .tile {
          border: 1px solid #2a3541;
          border-radius: 8px;
          background: #151d26;
          box-shadow: 0 16px 44px rgba(0, 0, 0, 0.25);
        }
        .panel { display: grid; gap: 12px; padding: 14px; }
        .tile { display: grid; gap: 7px; min-height: 105px; padding: 13px; border-top: 3px solid var(--tone, #9aa8b8); }
        [data-tone="success"] { --tone: #3af28f; }
        [data-tone="active"] { --tone: #27d7ff; }
        [data-tone="warning"] { --tone: #ffbd5a; }
        [data-tone="danger"] { --tone: #ff5a67; }
        [data-tone="lab"] { --tone: #9b7cff; }
        [data-tone="neutral"] { --tone: #9aa8b8; }
        .tile span, .panel span, .panel p, small { color: #9aa8b8; }
        strong, b { color: #f5f8fb; }
        .tile strong { font-family: Consolas, monospace; font-size: 22px; }
        .states, .actions { display: flex; flex-wrap: wrap; gap: 8px; }
        button {
          min-height: 36px;
          border: 1px solid #344252;
          border-radius: 8px;
          background: #202b37;
          color: #f5f8fb;
          font: inherit;
          font-weight: 700;
        }
        button.primary { background: #27d7ff; color: #071015; border-color: #27d7ff; }
        .row {
          display: grid;
          grid-template-columns: minmax(160px, 1.2fr) repeat(3, minmax(120px, 1fr));
          gap: 10px;
          padding: 10px 0;
          border-top: 1px solid #2a3541;
        }
        .row:first-of-type { border-top: 0; }
        @media (max-width: 900px) {
          .grid, .views { grid-template-columns: 1fr 1fr; }
          .row { grid-template-columns: 1fr; }
        }
        @media (max-width: 640px) {
          main { padding: 14px; }
          header { align-items: flex-start; flex-direction: column; }
          .grid, .views { grid-template-columns: 1fr; }
        }
      </style>
    </head>
    <body>
      <main>
        <header>
          <div>
            <p>Optimization workflow</p>
            <h1>Dashboard, scan, optimize, rollback</h1>
          </div>
          <div class="actions">
            ${workflow.optimize.actions
              .map((action) => `<button class="${action.variant === "primary" ? "primary" : ""}">${action.label}</button>`)
              .join("")}
          </div>
        </header>
        <section class="grid" aria-label="Dashboard metrics">
          ${workflow.dashboard.metrics.map(renderMetric).join("")}
        </section>
        <section class="views">
          <section class="panel">
            <h2>Scan</h2>
            <div class="states">
              ${workflow.scan.states.map((state) => `<button>${state.label}</button>`).join("")}
            </div>
            <p>${workflow.scan.progress.percent}% - ${workflow.scan.progress.current}</p>
            ${workflow.scan.findings
              .map((finding) => `<div class="row"><b>${finding.title}</b><span>${finding.group}</span><span>${finding.risk}</span><span>${finding.detail}</span></div>`)
              .join("")}
          </section>
          <section class="panel">
            <h2>Apply safety</h2>
            ${workflow.optimize.applySteps
              .map((step) => `<div class="row"><b>${step.label}</b><span>${step.state}</span><span>${step.detail}</span><span>Visible</span></div>`)
              .join("")}
          </section>
          ${workflow.optimize.groups.map(renderPlan).join("")}
          ${workflow.rollback.sessions.map(renderRollback).join("")}
        </section>
      </main>
    </body>
  </html>`;
}
