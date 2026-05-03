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
        label: "PC baseline",
        detail: "Windows, CPU, memory, storage, and active power plan",
        checked: true
      },
      {
        id: "graphics",
        label: "Graphics and display",
        detail: "Driver, refresh rate, VRR, HAGS, and overlays",
        checked: true
      },
      {
        id: "gaming",
        label: "Game readiness",
        detail: "Game Mode, captures, PUBG location, and BattlEye state",
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
        label: "Ready",
        detail: "Smart Scan is ready to check this PC",
        state: "complete"
      },
      {
        id: "scanning",
        label: "Checking PC",
        detail: "Collecting performance and safety signals",
        state: "active"
      },
      {
        id: "partial",
        label: "Graphics checked",
        detail: "Display and driver checks are complete",
        state: "pending"
      },
      {
        id: "complete",
        label: "Boost ready",
        detail: "Smart Boost can now open",
        state: "pending"
      },
      {
        id: "failed",
        label: "Needs retry",
        detail: "Retry keeps confirmed recommendations",
        state: "pending"
      },
      {
        id: "cancelled",
        label: "Paused",
        detail: "No changes were made",
        state: "pending"
      }
    ],
    progress: {
      label: "Graphics check",
      percent: 62,
      current: "Checking driver, VRR, and overlay state",
      completed: ["OS inventory", "CPU topology", "Active power plan", "PUBG process check"]
    },
    findings: [
      {
        id: "capture",
        group: "High impact",
        risk: "Low",
        title: "Background capture can be paused",
        detail: "Safe Boost can reduce recording overhead when capture is unused.",
        tone: "success"
      },
      {
        id: "power",
        group: "High impact",
        risk: "Low",
        title: "Balanced power plan found",
        detail: "Safe Boost can prepare a reversible Liiiraa power plan.",
        tone: "success"
      },
      {
        id: "driver",
        group: "Moderate impact",
        risk: "Low",
        title: "Driver is current",
        detail: "No driver replacement is recommended.",
        tone: "neutral"
      },
      {
        id: "vbs",
        group: "Tradeoff",
        risk: "Medium",
        title: "Security tradeoff needs review",
        detail: "Competitive changes explain the security impact before anything is applied.",
        tone: "warning"
      },
      {
        id: "defender",
        group: "Blocked",
        risk: "Critical",
        title: "Unsafe Defender change blocked",
        detail: "Only narrow verified exclusions or scheduling are allowed.",
        tone: "danger"
      }
    ]
  },
  optimize: {
    actions: [
      {
        id: "apply-safe",
        label: "Apply Safe Boost",
        variant: "primary"
      },
      {
        id: "include-competitive",
        label: "Review Competitive",
        variant: "secondary"
      },
      {
        id: "inspect-lab",
        label: "Inspect Lab",
        variant: "secondary"
      },
      {
        id: "export-plan",
        label: "Export Boost Plan",
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
        summary: "Low-risk reversible boost",
        tone: "success",
        applyEnabled: true,
        tweaks: [
          {
            id: "game.capture.background.off",
            change: "Pause unused background recording",
            expectedImpact: "Lower capture overhead",
            risk: "Low",
            rollback: "Capture setting saved first",
            reboot: "No",
            confidence: "High",
            why: "Capture is enabled and no active recording workflow was detected."
          },
          {
            id: "power.plan.liiiraa-balanced",
            change: "Prepare Liiiraa Boost - Balanced",
            expectedImpact: "Stable reversible baseline",
            risk: "Low",
            rollback: "Restore previous power plan",
            reboot: "No",
            confidence: "High",
            why: "The current power plan can be saved before performance tuning."
          },
          {
            id: "game.mode.verify",
            change: "Turn on Game Mode when needed",
            expectedImpact: "Windows gaming scheduling",
            risk: "Low",
            rollback: "Save current Game Mode choice",
            reboot: "No",
            confidence: "Medium",
            why: "Game Mode is the supported Windows path for gaming scheduling."
          }
        ]
      },
      {
        id: "competitive",
        label: "Competitive",
        summary: "Review-only performance tradeoffs",
        tone: "warning",
        applyEnabled: false,
        tweaks: [
          {
            id: "security.hvci.tradeoff",
            change: "Review Memory Integrity comparison",
            expectedImpact: "Possible latency and FPS uplift",
            risk: "Medium",
            rollback: "Restore previous HVCI state",
            reboot: "Required",
            confidence: "Medium",
            why: "Only valid after the security tradeoff is accepted."
          },
          {
            id: "game.hags.benchmark",
            change: "Benchmark HAGS before changing it",
            expectedImpact: "Hardware-dependent frametime change",
            risk: "Medium",
            rollback: "Restore previous graphics setting",
            reboot: "Maybe",
            confidence: "Medium",
            why: "HAGS results vary by hardware, driver, and game."
          }
        ]
      },
      {
        id: "lab",
        label: "Lab",
        summary: "Advanced tests with benchmarks",
        tone: "lab",
        applyEnabled: false,
        tweaks: [
          {
            id: "net.rsc.profile",
            change: "Test adapter-specific RSC",
            expectedImpact: "Latency or throughput diagnosis",
            risk: "High",
            rollback: "Adapter state backup",
            reboot: "Adapter restart",
            confidence: "Low",
            why: "Useful only with evidence from VPN, capture, or driver issues."
          }
        ]
      },
      {
        id: "blocked",
        label: "Blocked",
        summary: "Never applied",
        tone: "danger",
        applyEnabled: false,
        tweaks: [
          {
            id: "blocked.defender.disable",
            change: "Keep Defender protection on",
            expectedImpact: "Blocked for safety",
            risk: "Critical",
            rollback: "Not applied",
            reboot: "N/A",
            confidence: "High",
            why: "Disabling Defender globally is treated as a security regression."
          },
          {
            id: "blocked.anticheat-tamper",
            change: "Block BattlEye and PUBG memory tamper",
            expectedImpact: "Blocked for safety",
            risk: "Critical",
            rollback: "Not applied",
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
        detail: "Saving power, capture, GPU profile, and Windows settings"
      },
      {
        id: "apply",
        label: "Apply",
        state: "active",
        detail: "Applying Safe Boost changes only"
      },
      {
        id: "verify",
        label: "Verify",
        state: "pending",
        detail: "Confirming results and restart notes"
      },
      {
        id: "benchmark",
        label: "Benchmark prompt",
        state: "pending",
        detail: "Capture before and after results"
      },
      {
        id: "rollback",
        label: "Rollback if needed",
        state: "pending",
        detail: "Restore this boost session if needed"
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
  gaming: {
    power: {
      metrics: [
        {
          id: "active-plan",
          label: "Active plan",
          value: "Balanced",
          detail: "Baseline captured",
          tone: "neutral"
        },
        {
          id: "desktop-policy",
          label: "Desktop",
          value: "Safe",
          detail: "Performance plan allowed",
          tone: "success"
        },
        {
          id: "laptop-policy",
          label: "Laptop",
          value: "Consent",
          detail: "Heat and battery warning",
          tone: "warning"
        },
        {
          id: "rollback",
          label: "Rollback",
          value: "Full",
          detail: "Previous scheme stored",
          tone: "success"
        }
      ],
      actions: [
        {
          id: "stage-balanced",
          label: "Stage balanced",
          variant: "primary"
        },
        {
          id: "review-competitive",
          label: "Review competitive",
          variant: "secondary"
        },
        {
          id: "export-power-plan",
          label: "Export plan",
          variant: "ghost"
        }
      ],
      plans: [
        {
          id: "power.plan.liiiraa-balanced",
          label: "Liiiraa Boost - Balanced",
          mode: "Safe",
          state: "Ready",
          detail: "Duplicate current or Balanced plan before tuning.",
          rollback: "Restore previous active scheme",
          defaults: "On",
          tone: "success"
        },
        {
          id: "power.plan.liiiraa-performance",
          label: "Liiiraa Boost - Performance",
          mode: "Safe desktop, Competitive laptop",
          state: "Staged",
          detail: "Scoped AC performance values without global system defaults.",
          rollback: "Remove created GUID and restore prior plan",
          defaults: "Desktop on",
          tone: "warning"
        },
        {
          id: "power.plan.liiiraa-competitive",
          label: "Liiiraa Boost - Competitive",
          mode: "Competitive",
          state: "Locked",
          detail: "More aggressive AC-only values with explicit tradeoff review.",
          rollback: "Restore all backed-up plan values",
          defaults: "Off",
          tone: "warning"
        }
      ],
      rules: [
        {
          id: "usb",
          label: "USB selective suspend",
          value: "Scoped off",
          detail: "Applies only inside Liiiraa plans to reduce device wake latency.",
          tone: "success"
        },
        {
          id: "pcie",
          label: "PCIe link state",
          value: "Desktop safe",
          detail: "Laptop path requires heat and battery disclosure before apply.",
          tone: "warning"
        },
        {
          id: "processor",
          label: "Processor bias",
          value: "Competitive",
          detail: "EPP/min-max changes stay inside Liiiraa plans and require evidence.",
          tone: "warning"
        },
        {
          id: "core-parking",
          label: "Core parking",
          value: "Lab only",
          detail: "Benchmark experiment, never part of safe defaults.",
          tone: "lab"
        }
      ]
    },
    nvidia: {
      metrics: [
        {
          id: "gpu",
          label: "GPU",
          value: "RTX 4070",
          detail: "NVIDIA detected",
          tone: "success"
        },
        {
          id: "driver",
          label: "Driver",
          value: "551.86",
          detail: "Game Ready branch",
          tone: "success"
        },
        {
          id: "nvapi",
          label: "Profile API",
          value: "Ready",
          detail: "Backup required",
          tone: "active"
        },
        {
          id: "display",
          label: "Refresh",
          value: "240 Hz",
          detail: "VRR detected",
          tone: "active"
        },
        {
          id: "pubg-profile",
          label: "PUBG profile",
          value: "Staged",
          detail: "TslGame.exe scoped",
          tone: "warning"
        },
        {
          id: "battleye",
          label: "BattlEye",
          value: "Clear",
          detail: "No process lock",
          tone: "success"
        }
      ],
      actions: [
        {
          id: "backup-profiles",
          label: "Back up profiles",
          variant: "primary"
        },
        {
          id: "stage-pubg-profile",
          label: "Stage PUBG profile",
          variant: "secondary"
        },
        {
          id: "open-benchmark",
          label: "Open benchmark",
          variant: "ghost"
        }
      ],
      profiles: [
        {
          id: "nvidia.global.profile",
          label: "Liiiraa Boost - Global Performance",
          scope: "Global profile",
          state: "Backup required",
          recommendation: "Conservative profile-level power and shader cache settings.",
          rollback: "Import global backup",
          tone: "success"
        },
        {
          id: "nvidia.pubg.profile",
          label: "Liiiraa Boost - PUBG Competitive",
          scope: "TslGame.exe",
          state: "Ready when PUBG is closed",
          recommendation: "Low Latency On for non-Reflex paths, highest refresh, VRR cap logic.",
          rollback: "Import PUBG profile backup",
          tone: "warning"
        },
        {
          id: "nvidia.rebar.hidden-override",
          label: "Hidden ReBAR override",
          scope: "NPI compatibility",
          state: "Lab locked",
          recommendation: "Benchmark-gated only; no global force or firmware flashing.",
          rollback: "Restore profile backup",
          tone: "lab"
        }
      ],
      policies: [
        {
          id: "reflex",
          label: "Reflex vs Low Latency",
          value: "No blind stacking",
          detail: "Prefer in-game Reflex when supported; driver LLM stays profile-specific.",
          tone: "success"
        },
        {
          id: "vrr-cap",
          label: "VRR frame cap",
          value: "237 FPS",
          detail: "Recommended cap is below 240 Hz and tied to the PUBG profile.",
          tone: "active"
        },
        {
          id: "battleye",
          label: "Mutation guard",
          value: "Defer while running",
          detail: "Profile changes are blocked when PUBG or BattlEye is active.",
          tone: "warning"
        },
        {
          id: "hidden-flags",
          label: "Hidden bulk flags",
          value: "Denied",
          detail: "Undocumented dumps are not default optimization content.",
          tone: "danger"
        }
      ],
      capLogic: [
        ["Display refresh", "240 Hz"],
        ["VRR state", "Detected and enabled"],
        ["Recommended cap", "237 FPS"],
        ["Cap source", "Profile-specific, verified after write"],
        ["V-SYNC policy", "Explained with VRR, not forced blindly"]
      ]
    },
    pubg: {
      metrics: [
        {
          id: "install",
          label: "Install",
          value: "Steam",
          detail: "TslGame.exe found",
          tone: "success"
        },
        {
          id: "config",
          label: "Config",
          value: "Readable",
          detail: "Snapshot required",
          tone: "success"
        },
        {
          id: "battleye",
          label: "BattlEye",
          value: "Protected",
          detail: "No file or memory touch",
          tone: "success"
        },
        {
          id: "dx-mode",
          label: "DX mode",
          value: "Benchmark",
          detail: "DX11 vs DX11 Enhanced",
          tone: "warning"
        },
        {
          id: "launch-options",
          label: "Launch flags",
          value: "Review",
          detail: "Legacy flags detected",
          tone: "warning"
        },
        {
          id: "nvidia-link",
          label: "NVIDIA",
          value: "Linked",
          detail: "PUBG profile staged",
          tone: "active"
        }
      ],
      actions: [
        {
          id: "snapshot-config",
          label: "Snapshot config",
          variant: "primary"
        },
        {
          id: "start-dx-benchmark",
          label: "Start DX benchmark",
          variant: "secondary"
        },
        {
          id: "open-nvidia-profile",
          label: "Open NVIDIA profile",
          variant: "ghost"
        }
      ],
      detections: [
        {
          id: "exe",
          label: "Executable",
          value: "TslGame.exe",
          detail: "Steam install path detected without modifying game folders.",
          tone: "success"
        },
        {
          id: "config",
          label: "Config snapshot",
          value: "Required",
          detail: "Supported user config keys are read only until backup exists.",
          tone: "active"
        },
        {
          id: "launch",
          label: "Launch options",
          value: "Cleanup recommended",
          detail: "Remove legacy viral flags instead of adding unsupported flags.",
          tone: "warning"
        },
        {
          id: "anticheat",
          label: "Anti-cheat boundary",
          value: "Strict",
          detail: "No BattlEye files, game memory, binaries, kernel state, or integrity bypasses.",
          tone: "danger"
        }
      ],
      dxChoices: [
        {
          id: "dx11e",
          label: "DX11 Enhanced",
          evidence: "Candidate only after measured 1% lows and p95 frametime clear variance.",
          rollback: "Restore prior render mode",
          state: "Benchmark required",
          tone: "warning"
        },
        {
          id: "dx11",
          label: "DX11",
          evidence: "Compatibility baseline for the same map, route, and duration.",
          rollback: "Restore prior render mode",
          state: "Benchmark required",
          tone: "active"
        },
        {
          id: "dx12",
          label: "DX12",
          evidence: "Lab only if exposed by game and user opts in.",
          rollback: "Restore prior render mode",
          state: "Lab locked",
          tone: "lab"
        }
      ],
      dxBenchmark: {
        currentMode: "DX11",
        selectedMode: "Pending evidence",
        rationale:
          "No universal forced default: keep the current render mode unless DX11 or DX11 Enhanced wins outside the variance band with no stability blocker.",
        varianceBand: "3%",
        steps: [
          {
            id: "snapshot-config",
            label: "Snapshot config",
            detail: "Back up the current render mode before any comparison.",
            state: "complete",
            tone: "success"
          },
          {
            id: "capture-dx11",
            label: "Capture DX11",
            detail: "Run the same route and duration as the compatibility baseline.",
            state: "active",
            tone: "active"
          },
          {
            id: "capture-dx11e",
            label: "Capture DX11 Enhanced",
            detail: "Repeat the capture after user-controlled mode selection.",
            state: "pending",
            tone: "warning"
          },
          {
            id: "compare",
            label: "Compare",
            detail: "Use native FPS, 1% lows, 0.1% lows, p95 frametime, dropped frames, and stability notes.",
            state: "pending",
            tone: "neutral"
          },
          {
            id: "recommend",
            label: "Recommend",
            detail: "Recommend only the mode that wins outside variance; otherwise keep current.",
            state: "pending",
            tone: "neutral"
          }
        ],
        results: [
          {
            id: "dx11",
            label: "DX11 baseline",
            averageFps: 176,
            onePercentLow: 127,
            pointOnePercentLow: 92,
            p95FrameMs: 10.2,
            droppedFrames: 4,
            verdict: "Baseline",
            tone: "active"
          },
          {
            id: "dx11e",
            label: "DX11 Enhanced",
            averageFps: 181,
            onePercentLow: 139,
            pointOnePercentLow: 99,
            p95FrameMs: 9.5,
            droppedFrames: 2,
            verdict: "Candidate wins after stability pass",
            tone: "success"
          }
        ],
        metadata: [
          ["Route", "Same map/replay route"],
          ["Duration", "Same capture length"],
          ["Driver", "Same GPU driver"],
          ["Power plan", "Same Windows power plan"],
          ["Frames", "Native frames only"]
        ]
      },
      checklist: [
        {
          id: "pubg.settings.visibility",
          label: "Visibility settings",
          value: "Backed-up apply",
          detail:
            "Snapshot config first; recommend supported keys for shadows, effects, foliage, render scale, and clarity only after user review.",
          tone: "success"
        },
        {
          id: "game.graphics.preference.pubg",
          label: "Windows GPU preference",
          value: "High performance",
          detail: "Set TslGame.exe to the high-performance GPU when Windows exposes the app preference; back up the prior value.",
          tone: "active"
        },
        {
          id: "game.capture.background.off",
          label: "Capture and Game Mode",
          value: "Safe Windows plan",
          detail:
            "Keep Game Mode visible, disable unused background capture, and warn before GameDVR/FSO changes when HDR or ICC workflows are present.",
          tone: "warning"
        },
        {
          id: "game.present-path.benchmark",
          label: "Present path",
          value: "Benchmark",
          detail: "Compare fullscreen, borderless, VRR, HAGS, and overlay combinations instead of assuming one path wins.",
          tone: "warning"
        },
        {
          id: "nvidia.pubg.profile",
          label: "NVIDIA profile",
          value: "TslGame.exe scoped",
          detail:
            "Use the PUBG profile link for max performance, highest refresh, and texture-filtering choices; block changes while PUBG or BattlEye is running.",
          tone: "active"
        },
        {
          id: "nvidia.reflex-vs-llm.policy",
          label: "Reflex and frame cap",
          value: "No blind stacking",
          detail:
            "Prefer in-game Reflex when available, avoid driver Ultra as a default, and tie the cap to detected refresh and VRR state.",
          tone: "success"
        },
        {
          id: "pubg.files.verify",
          label: "Verify files",
          value: "Store flow",
          detail: "Crashes or corruption route to Steam/Epic repair, not file deletion.",
          tone: "warning"
        },
        {
          id: "blocked.anticheat-tamper",
          label: "Unsafe tweaks",
          value: "Denied",
          detail: "Realtime priority, memory edits, and BattlEye tamper remain blocked.",
          tone: "danger"
        }
      ]
    },
    benchmarks: {
      metrics: [
        {
          id: "avg-fps",
          label: "Average",
          value: "188",
          detail: "FPS, native frames",
          tone: "active"
        },
        {
          id: "one-percent",
          label: "1% low",
          value: "142",
          detail: "+11.8% vs baseline",
          tone: "success"
        },
        {
          id: "point-one",
          label: "0.1% low",
          value: "96",
          detail: "+7.4% vs baseline",
          tone: "success"
        },
        {
          id: "p95",
          label: "p95 frame",
          value: "8.8 ms",
          detail: "Lower is better",
          tone: "active"
        },
        {
          id: "variance",
          label: "Variance",
          value: "Medium",
          detail: "Same map required",
          tone: "warning"
        },
        {
          id: "metadata",
          label: "Metadata",
          value: "Complete",
          detail: "Driver, build, plan, catalog",
          tone: "success"
        }
      ],
      actions: [
        {
          id: "capture-before",
          label: "Capture before",
          variant: "primary"
        },
        {
          id: "compare-after",
          label: "Compare after",
          variant: "secondary"
        },
        {
          id: "export-benchmark",
          label: "Export report",
          variant: "ghost"
        }
      ],
      summary: {
        score: "72/100",
        confidence: "Medium confidence",
        decision: "Prefer after run",
        varianceBand: "+/-3%",
        detail: "Weighted from 1% lows, 0.1% lows, p95 frametime, average FPS, and dropped frames.",
        warnings: [
          {
            id: "score-variance",
            label: "Variance",
            value: "Warn",
            detail: "Average FPS is useful context, but the score depends on lows clearing the +/-3% band.",
            tone: "warning"
          },
          {
            id: "score-confidence",
            label: "Confidence",
            value: "Medium",
            detail: "Repeat the same map, route, duration, driver, and power plan before calling it final.",
            tone: "warning"
          }
        ]
      },
      chart: [
        {
          id: "baseline",
          label: "Baseline",
          averageFps: 176,
          onePercentLow: 127,
          pointOnePercentLow: 91,
          p95FrameMs: 10.2,
          tone: "neutral"
        },
        {
          id: "safe-plan",
          label: "Safe plan",
          averageFps: 184,
          onePercentLow: 136,
          pointOnePercentLow: 99,
          p95FrameMs: 9.3,
          tone: "active"
        },
        {
          id: "pubg-profile",
          label: "PUBG profile",
          averageFps: 188,
          onePercentLow: 142,
          pointOnePercentLow: 106,
          p95FrameMs: 8.8,
          tone: "success"
        }
      ],
      metadata: [
        ["Map/session", "Training range, 5 minute route"],
        ["Driver", "NVIDIA 551.86"],
        ["Windows build", "23H2, pending reboot clear"],
        ["Power plan", "Liiiraa Boost - Performance"],
        ["Catalog", "v1 local matrix"],
        ["Generated frames", "Excluded from native FPS summary"]
      ],
      sessions: [
        {
          id: "bench-before",
          label: "Before",
          value: "Captured",
          detail: "Baseline with Balanced plan and driver defaults.",
          tone: "neutral"
        },
        {
          id: "bench-after",
          label: "After",
          value: "Ready",
          detail: "Safe plan plus PUBG profile, same metadata required.",
          tone: "success"
        },
        {
          id: "bench-variance",
          label: "Confidence",
          value: "Warn",
          detail: "Result inside variance stays advisory instead of success.",
          tone: "warning"
        }
      ]
    }
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

  const gaming = workflow.gaming;
  const requiredGamingSurfaces = ["power", "nvidia", "pubg", "benchmarks"];
  const missingGamingSurfaces = requiredGamingSurfaces.filter((surface) => !gaming?.[surface]);

  if (missingGamingSurfaces.length > 0) {
    throw new Error(`Gaming surfaces missing: ${missingGamingSurfaces.join(", ")}`);
  }

  const requiredPowerPlans = [
    "power.plan.liiiraa-balanced",
    "power.plan.liiiraa-performance",
    "power.plan.liiiraa-competitive"
  ];
  const powerPlans = new Set(gaming.power.plans.map((plan) => plan.id));
  const missingPowerPlans = requiredPowerPlans.filter((plan) => !powerPlans.has(plan));

  if (missingPowerPlans.length > 0) {
    throw new Error(`Power plans missing: ${missingPowerPlans.join(", ")}`);
  }

  const requiredNvidiaProfiles = ["nvidia.global.profile", "nvidia.pubg.profile"];
  const nvidiaProfiles = new Set(gaming.nvidia.profiles.map((profile) => profile.id));
  const missingNvidiaProfiles = requiredNvidiaProfiles.filter((profile) => !nvidiaProfiles.has(profile));

  if (missingNvidiaProfiles.length > 0) {
    throw new Error(`NVIDIA profiles missing: ${missingNvidiaProfiles.join(", ")}`);
  }

  const hasBattleyePolicy = gaming.nvidia.policies.some((policy) => /BattlEye|running/i.test(policy.detail));
  if (!hasBattleyePolicy) {
    throw new Error("NVIDIA surface must disclose BattlEye/PUBG running deferral.");
  }

  const hasPubgBoundary = gaming.pubg.detections.some((item) => /BattlEye|memory|binaries/i.test(item.detail));
  if (!hasPubgBoundary) {
    throw new Error("PUBG surface must show anti-cheat boundaries.");
  }

  const dxResults = new Set(gaming.pubg.dxBenchmark.results.map((result) => result.id));
  if (!dxResults.has("dx11") || !dxResults.has("dx11e")) {
    throw new Error("PUBG DX benchmark flow must compare DX11 and DX11 Enhanced.");
  }

  if (!/No universal forced default/i.test(gaming.pubg.dxBenchmark.rationale)) {
    throw new Error("PUBG DX benchmark flow must reject a universal forced default.");
  }

  const defaultedDxChoice = gaming.pubg.dxChoices.find((choice) => /default selected/i.test(choice.state));
  if (defaultedDxChoice) {
    throw new Error(`PUBG DX choice must not be default-selected: ${defaultedDxChoice.label}`);
  }

  const requiredPubgChecklist = [
    "pubg.settings.visibility",
    "game.graphics.preference.pubg",
    "game.capture.background.off",
    "game.present-path.benchmark",
    "nvidia.pubg.profile",
    "nvidia.reflex-vs-llm.policy",
    "blocked.anticheat-tamper"
  ];
  const pubgChecklistIds = new Set(gaming.pubg.checklist.map((item) => item.id));
  const missingPubgChecklist = requiredPubgChecklist.filter((item) => !pubgChecklistIds.has(item));

  if (missingPubgChecklist.length > 0) {
    throw new Error(`PUBG competitive checklist missing: ${missingPubgChecklist.join(", ")}`);
  }

  const hasWindowsCrossPlan = gaming.pubg.checklist.some((item) =>
    /Windows|GameDVR|HAGS|VRR|fullscreen|borderless|overlay/i.test(`${item.label} ${item.detail}`)
  );
  if (!hasWindowsCrossPlan) {
    throw new Error("PUBG checklist must include Windows graphics and present-path planning.");
  }

  const hasNvidiaCrossPlan = gaming.pubg.checklist.some((item) =>
    /NVIDIA|Reflex|frame cap|refresh|BattlEye/i.test(`${item.label} ${item.detail}`)
  );
  if (!hasNvidiaCrossPlan) {
    throw new Error("PUBG checklist must include NVIDIA profile and latency/cap planning.");
  }

  if (gaming.benchmarks.chart.length < 2 || gaming.benchmarks.metadata.length === 0) {
    throw new Error("Benchmark surface must include comparison data and metadata.");
  }

  const benchmarkSummary = gaming.benchmarks.summary;
  if (!benchmarkSummary?.score || !benchmarkSummary?.confidence || !benchmarkSummary?.decision) {
    throw new Error("Benchmark surface must include comparison score, confidence, and decision.");
  }

  const hasVarianceWarning = benchmarkSummary.warnings?.some((warning) => /variance/i.test(warning.label)) === true;
  if (!hasVarianceWarning) {
    throw new Error("Benchmark comparison summary must disclose variance warnings.");
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

  const renderStatusRows = (items) =>
    items
      .map(
        (item) => `
        <div class="row" data-tone="${item.tone}">
          <b>${item.label}</b>
          <span>${item.value}</span>
          <span>${item.detail}</span>
          <span>${item.id}</span>
        </div>`
      )
      .join("");

  const renderPowerPlan = (plan) => `
    <div class="row" data-tone="${plan.tone}">
      <b>${plan.label}</b>
      <span>${plan.mode}</span>
      <span>${plan.state}</span>
      <span>${plan.rollback}</span>
    </div>`;

  const renderProfile = (profile) => `
    <div class="row" data-tone="${profile.tone}">
      <b>${profile.label}</b>
      <span>${profile.scope}</span>
      <span>${profile.state}</span>
      <span>${profile.rollback}</span>
    </div>`;

  const renderBenchmarkBar = (point) => {
    const width = Math.max(14, Math.min(100, Math.round((point.onePercentLow / 160) * 100)));

    return `
      <div class="bar-row" data-tone="${point.tone}">
        <b>${point.label}</b>
        <span class="bar-track"><span class="bar-fill" style="width: ${width}%"></span></span>
        <span>${point.onePercentLow} FPS 1% low</span>
        <span>p95 ${point.p95FrameMs} ms</span>
      </div>`;
  };

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
        .bar-row {
          display: grid;
          grid-template-columns: minmax(140px, 0.8fr) minmax(180px, 1.4fr) repeat(2, minmax(110px, 0.8fr));
          gap: 10px;
          align-items: center;
          padding: 10px 0;
          border-top: 1px solid #2a3541;
        }
        .bar-track {
          display: block;
          height: 12px;
          overflow: hidden;
          border: 1px solid #344252;
          border-radius: 999px;
          background: #202b37;
        }
        .bar-fill {
          display: block;
          height: 100%;
          background: var(--tone, #27d7ff);
        }
        .row:first-of-type { border-top: 0; }
        @media (max-width: 900px) {
          .grid, .views { grid-template-columns: 1fr 1fr; }
          .row, .bar-row { grid-template-columns: 1fr; }
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
            <h1>Dashboard, scan, optimize, rollback, gaming surfaces</h1>
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
          <section class="panel">
            <h2>Power</h2>
            ${workflow.gaming.power.plans.map(renderPowerPlan).join("")}
            ${renderStatusRows(workflow.gaming.power.rules)}
          </section>
          <section class="panel">
            <h2>NVIDIA</h2>
            ${workflow.gaming.nvidia.profiles.map(renderProfile).join("")}
            ${renderStatusRows(workflow.gaming.nvidia.policies)}
          </section>
          <section class="panel">
            <h2>PUBG</h2>
            ${renderStatusRows(workflow.gaming.pubg.detections)}
            ${workflow.gaming.pubg.dxBenchmark.results
              .map(
                (result) => `
                <div class="row" data-tone="${result.tone}">
                  <b>${result.label}</b>
                  <span>${result.averageFps} avg / ${result.onePercentLow} FPS 1% low</span>
                  <span>p95 ${result.p95FrameMs} ms / ${result.droppedFrames} dropped</span>
                  <span>${result.verdict}</span>
                </div>`
              )
              .join("")}
            ${renderStatusRows(workflow.gaming.pubg.checklist)}
          </section>
          <section class="panel">
            <h2>Benchmarks</h2>
            <div class="row" data-tone="warning">
              <b>${workflow.gaming.benchmarks.summary.score}</b>
              <span>${workflow.gaming.benchmarks.summary.decision}</span>
              <span>${workflow.gaming.benchmarks.summary.confidence}</span>
              <span>${workflow.gaming.benchmarks.summary.varianceBand}</span>
            </div>
            ${renderStatusRows(workflow.gaming.benchmarks.summary.warnings)}
            ${workflow.gaming.benchmarks.chart.map(renderBenchmarkBar).join("")}
            ${workflow.gaming.benchmarks.sessions
              .map((session) => `<div class="row" data-tone="${session.tone}"><b>${session.label}</b><span>${session.value}</span><span>${session.detail}</span><span>${session.id}</span></div>`)
              .join("")}
          </section>
        </section>
      </main>
    </body>
  </html>`;
}
