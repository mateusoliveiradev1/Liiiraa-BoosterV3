import { useEffect, useMemo, useState, type FocusEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  BarChart3,
  BatteryCharging,
  Bell,
  CheckCircle2,
  CircuitBoard,
  CircleHelp,
  Crown,
  Crosshair,
  Gamepad2,
  Grid3X3,
  History,
  LayoutDashboard,
  Minus,
  MonitorCog,
  PanelLeftOpen,
  Radar,
  RefreshCw,
  Settings,
  SlidersHorizontal,
  Sparkles,
  Square,
  X,
  type LucideIcon
} from "lucide-react";
import { getActiveOptimizerLocale, tOptimizer } from "../../../packages/ui/src/localization";
import logoFull from "./assets/logo.svg";
import logoMark from "./assets/logo-mark.svg";
import {
  runDesktopAction,
  subscribeDesktopActionFeedback,
  type DesktopActionFeedback
} from "./actionRuntime";
import {
  desktopCommandCenterState,
  isDesktopRouteId,
  type DesktopNavigationIconName,
  type DesktopNavigationItem,
  type DesktopRouteInspector,
  type DesktopRouteId
} from "./adapters/desktopState";
import {
  actionIconForLabel,
  CommandHeader,
  StatusStrip,
  type CommandHeaderTrustItem,
  type CoreAction,
  type CoreActionVariant
} from "./components/CorePrimitives";
import {
  canUseLiveDashboardTelemetry,
  collectLiveDashboardTelemetry,
  getLiveDashboardTelemetryPreference,
  LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS,
  subscribeLiveDashboardTelemetryPreference,
  type LiveResourceSnapshot
} from "./liveDashboardTelemetry";
import { defaultOptimizationRouteId, optimizationRoutes } from "./routes";

const navigationIcons: Record<DesktopNavigationIconName, LucideIcon> = {
  activity: Activity,
  "bar-chart": BarChart3,
  crosshair: Crosshair,
  gamepad: Gamepad2,
  gauge: LayoutDashboard,
  gpu: CircuitBoard,
  power: BatteryCharging,
  rollback: History,
  rocket: Sparkles,
  scan: Radar,
  settings: SlidersHorizontal
};

export function App() {
  const [activeView, setActiveView] = useState<DesktopRouteId>(getInitialActiveView);
  const [actionFeedback, setActionFeedback] = useState<DesktopActionFeedback | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(true);
  const [infoPanelOpen, setInfoPanelOpen] = useState(false);
  const activeLocale = getActiveOptimizerLocale();

  const activeRoute = useMemo(
    () =>
      optimizationRoutes.find((route) => route.id === activeView) ??
      optimizationRoutes.find((route) => route.id === defaultOptimizationRouteId),
    [activeView]
  );
  if (!activeRoute) {
    throw new Error(tOptimizer("shell.routeMissing"));
  }

  const routeInspector = desktopCommandCenterState.inspector[activeRoute.id];
  const headerActions = createCommandHeaderActions(activeRoute.id, routeInspector.actions);
  const trustItems = createCommandHeaderTrustItems();
  const nextAction = {
    detail: routeInspector.summary,
    label: tOptimizer("commandHeader.nextAction"),
    tone: routeInspector.tone,
    value: routeInspector.actions[0] ?? routeInspector.title
  };

  useEffect(() => {
    document.documentElement.lang = activeLocale;
  }, [activeLocale]);

  useEffect(() => subscribeDesktopActionFeedback(setActionFeedback), []);

  useEffect(() => {
    if (!actionFeedback) {
      return undefined;
    }

    const timeout = window.setTimeout(() => setActionFeedback(null), 3600);

    return () => window.clearTimeout(timeout);
  }, [actionFeedback]);

  useEffect(() => {
    if (!infoPanelOpen) {
      return undefined;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setInfoPanelOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [infoPanelOpen]);

  useEffect(() => {
    const handleHashChange = () => {
      const routeId = getRouteIdFromHash();

      if (routeId) {
        setActiveView(routeId);
      }
    };

    window.addEventListener("hashchange", handleHashChange);

    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  const selectView = (id: DesktopRouteId) => {
    setActiveView(id);

    if (window.location.hash !== `#${id}`) {
      window.history.replaceState(null, "", `${window.location.pathname}${window.location.search}#${id}`);
    }
  };

  const expandSidebar = () => setSidebarCollapsed(false);
  const collapseSidebar = () => setSidebarCollapsed(true);
  const handleSidebarBlur = (event: FocusEvent<HTMLElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
      collapseSidebar();
    }
  };

  return (
    <div className="app-shell" data-locale={activeLocale} data-sidebar-collapsed={sidebarCollapsed}>
      <aside
        className="sidebar"
        aria-label={tOptimizer("shell.sidebarAria")}
        data-collapsed={sidebarCollapsed}
        onBlurCapture={handleSidebarBlur}
        onFocusCapture={expandSidebar}
        onMouseEnter={expandSidebar}
        onMouseLeave={collapseSidebar}
      >
        <span
          className="sidebar__menu"
          aria-hidden="true"
        >
          <PanelLeftOpen aria-hidden="true" size={24} strokeWidth={2.1} />
        </span>
        <a
          className="brand"
          href={`#${defaultOptimizationRouteId}`}
          aria-label={tOptimizer("brand.commandCenterAria")}
          onClick={(event) => {
            event.preventDefault();
            selectView(defaultOptimizationRouteId);
          }}
        >
          <img src={logoFull} alt="" className="brand__logo" />
          <img src={logoMark} alt="" className="brand__mark" />
          <span className="brand__text">{tOptimizer("brand.appName")}</span>
        </a>
        <nav className="nav-list" id="desktop-sidebar-navigation" aria-label={tOptimizer("shell.navigationAria")}>
          {desktopCommandCenterState.navigation.map((item) => (
            <NavButton key={item.id} item={item} active={item.id === activeView} onSelect={selectView} />
          ))}
        </nav>

        <section className="subscription-card" aria-label="Your License">
          <span className="subscription-card__icon" aria-hidden="true">
            <Crown size={28} strokeWidth={2.2} />
          </span>
          <span>
            <strong>Your License</strong>
            <b>Pro Edition</b>
            <small>Expires: May 26, 2026</small>
          </span>
          <button
            type="button"
            onClick={() =>
              void runDesktopAction({
                feedback: "Opening license and trust settings.",
                id: "manage-plan",
                label: "Manage Plan",
                targetRoute: "settings"
              })
            }
          >
            Manage Plan
          </button>
        </section>

        <span className="sidebar__version">
          <span aria-hidden="true" />
          Version 2.3.0
        </span>
      </aside>

      <div className="workspace" data-route={activeRoute.id}>
        <header className="workspace-chrome" data-tauri-drag-region>
          <div className="workspace-chrome__identity" data-tauri-drag-region aria-label={tOptimizer("brand.appName")}>
            <Grid3X3 aria-hidden="true" size={22} strokeWidth={2.2} />
            <span>{tOptimizer("brand.appName")}</span>
          </div>
          <div className="workspace-chrome__system" data-tauri-drag-region aria-label="System Status Optimal">
            <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2.4} />
            <span>
              System Status: <strong>Optimal</strong>
            </span>
          </div>
          <StatusStrip items={desktopCommandCenterState.statusStrip} label={tOptimizer("shell.statusStripAria")} />
          <div className="workspace-chrome__actions" aria-label="Window controls">
            <button
              className="chrome-button"
              type="button"
              aria-label="Notifications"
              onClick={() =>
                void runDesktopAction({
                  feedback: "No new optimization notifications. Status and rollback context are current.",
                  id: "notifications",
                  label: "Notifications"
                })
              }
            >
              <Bell aria-hidden="true" size={20} strokeWidth={2.1} />
            </button>
            <button
              className="chrome-button"
              type="button"
              aria-label={tOptimizer("routes.settings.label")}
              onClick={() => selectView("settings")}
            >
              <Settings aria-hidden="true" size={20} strokeWidth={2.1} />
            </button>
            <button
              className="chrome-button"
              type="button"
              aria-label="Help"
              onClick={() => setInfoPanelOpen(true)}
            >
              <CircleHelp aria-hidden="true" size={20} strokeWidth={2.1} />
            </button>
            <span className="workspace-chrome__divider" aria-hidden="true" />
            <button
              className="chrome-button"
              type="button"
              aria-label="Minimize"
              onClick={() => void runWindowCommand("minimize")}
            >
              <Minus aria-hidden="true" size={20} strokeWidth={2.1} />
            </button>
            <button
              className="chrome-button"
              type="button"
              aria-label="Maximize"
              onClick={() => void runWindowCommand("toggleMaximize")}
            >
              <Square aria-hidden="true" size={16} strokeWidth={2.1} />
            </button>
            <button
              className="chrome-button chrome-button--close"
              type="button"
              aria-label="Close"
              onClick={() => void runWindowCommand("close")}
            >
              <X aria-hidden="true" size={21} strokeWidth={2.1} />
            </button>
          </div>
        </header>
        <CommandHeader
          actions={headerActions}
          eyebrow={routeInspector.eyebrow}
          nextAction={nextAction}
          summary={routeInspector.summary}
          title={routeInspector.title}
          trustItems={trustItems}
        />
        <main className="command-center" id={activeRoute.id}>
          {activeRoute.element}
        </main>
        {infoPanelOpen ? (
          <DesktopInfoModal
            activeRouteId={activeRoute.id}
            activeRouteLabel={activeRoute.label}
            inspector={routeInspector}
            onClose={() => setInfoPanelOpen(false)}
            onSelectView={selectView}
          />
        ) : null}
        {actionFeedback ? <ActionFeedbackToast feedback={actionFeedback} /> : null}
      </div>
    </div>
  );
}

function ActionFeedbackToast({ feedback }: { feedback: DesktopActionFeedback }) {
  return (
    <aside className="action-feedback-toast" data-tone={feedback.tone} role="status" aria-live="polite">
      <strong>{feedback.label}</strong>
      <span>{feedback.detail}</span>
    </aside>
  );
}

function DesktopInfoModal({
  activeRouteId,
  activeRouteLabel,
  inspector,
  onClose,
  onSelectView
}: {
  activeRouteId: DesktopRouteId;
  activeRouteLabel: string;
  inspector: DesktopRouteInspector;
  onClose: () => void;
  onSelectView: (id: DesktopRouteId) => void;
}) {
  const liveTelemetryAvailable = useMemo(() => canUseLiveDashboardTelemetry(), []);
  const [liveTelemetryEnabled, setLiveTelemetryEnabled] = useState(getLiveDashboardTelemetryPreference);
  const [liveSnapshot, setLiveSnapshot] = useState<LiveResourceSnapshot | null>(null);
  const [liveStatus, setLiveStatus] = useState<DesktopInfoLiveStatus>(
    liveTelemetryEnabled ? (liveTelemetryAvailable ? "idle" : "unavailable") : "paused"
  );
  const [liveError, setLiveError] = useState<string | null>(null);
  const [refreshRequest, setRefreshRequest] = useState(0);
  const openRoute = (route: DesktopRouteId) => {
    onClose();
    onSelectView(route);
  };
  const liveTone = toneForDesktopInfoStatus(liveStatus);
  const liveMetrics = liveStatus === "ready" && liveSnapshot ? createDesktopInfoLiveMetrics(liveSnapshot) : [];
  const runtimeFacts = createDesktopInfoFacts({
    activeRouteId,
    activeRouteLabel,
    liveSnapshot,
    liveStatus,
    liveTelemetryAvailable,
    liveTelemetryEnabled
  });

  useEffect(() => subscribeLiveDashboardTelemetryPreference(setLiveTelemetryEnabled), []);

  useEffect(() => {
    if (!liveTelemetryEnabled) {
      setLiveStatus("paused");
      setLiveError(null);
      return undefined;
    }

    if (!liveTelemetryAvailable) {
      setLiveStatus("unavailable");
      setLiveError(null);
      return undefined;
    }

    let cancelled = false;
    let inFlight = false;

    const refreshLiveSnapshot = async () => {
      if (inFlight) {
        return;
      }

      inFlight = true;
      setLiveStatus((current) => (current === "ready" ? current : "loading"));

      try {
        const snapshot = await collectLiveDashboardTelemetry();

        if (!cancelled) {
          setLiveSnapshot(snapshot);
          setLiveError(null);
          setLiveStatus("ready");
        }
      } catch (error) {
        if (!cancelled) {
          setLiveError(error instanceof Error ? error.message : "Live desktop telemetry is unavailable.");
          setLiveStatus("error");
        }
      } finally {
        inFlight = false;
      }
    };

    void refreshLiveSnapshot();
    const interval = window.setInterval(() => void refreshLiveSnapshot(), LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [liveTelemetryAvailable, liveTelemetryEnabled, refreshRequest]);

  return (
    <div className="desktop-info-modal" role="presentation" onMouseDown={onClose}>
      <section
        aria-label="Desktop information"
        aria-modal="true"
        className="desktop-info-modal__panel"
        data-tone={inspector.tone}
        onMouseDown={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="desktop-info-modal__header">
          <div>
            <p className="eyebrow">Help</p>
            <h2>Help and live data</h2>
          </div>
          <button className="chrome-button" type="button" aria-label="Close information" onClick={onClose}>
            <X aria-hidden="true" size={20} strokeWidth={2.1} />
          </button>
        </div>

        <div className="desktop-info-modal__summary">
          <span className="desktop-info-modal__summary-icon" data-tone={liveTone} aria-hidden="true">
            <MonitorCog size={21} strokeWidth={2.2} />
          </span>
          <span>
            <strong>{activeRouteLabel}</strong>
            <small>{desktopInfoStatusDetail(liveStatus, liveSnapshot, liveError)}</small>
          </span>
          <b data-tone={liveTone}>{desktopInfoStatusLabel(liveStatus)}</b>
        </div>

        <dl className="desktop-info-modal__facts">
          {runtimeFacts.map(([label, value]) => (
            <div key={label}>
              <dt>{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>

        <section className="desktop-info-modal__live" aria-label="Live desktop resource readings">
          <div className="desktop-info-modal__section-heading">
            <span>
              <MonitorCog aria-hidden="true" size={18} strokeWidth={2.2} />
              <strong>Live resource readings</strong>
            </span>
            <button
              className="button button--ghost"
              disabled={!liveTelemetryAvailable || !liveTelemetryEnabled}
              onClick={() => setRefreshRequest((current) => current + 1)}
              type="button"
            >
              <RefreshCw aria-hidden="true" size={16} strokeWidth={2.2} />
              <span>Refresh</span>
            </button>
          </div>

          {liveMetrics.length > 0 ? (
            <div className="desktop-info-modal__metrics">
              {liveMetrics.map((metric) => (
                <article data-tone={metric.tone} key={metric.id}>
                  <span>{metric.label}</span>
                  <strong>{metric.value}</strong>
                  <small>{metric.detail}</small>
                </article>
              ))}
            </div>
          ) : (
            <div className="desktop-info-modal__empty" data-tone={liveTone}>
              <MonitorCog aria-hidden="true" size={22} strokeWidth={2.2} />
              <span>
                <strong>{desktopInfoStatusLabel(liveStatus)}</strong>
                <small>{desktopInfoStatusDetail(liveStatus, liveSnapshot, liveError)}</small>
              </span>
            </div>
          )}
        </section>

        <div className="desktop-info-modal__actions">
          <button className="button button--secondary" type="button" onClick={() => openRoute("settings")}>
            <Settings aria-hidden="true" size={16} strokeWidth={2.2} />
            <span>{tOptimizer("routes.settings.label")}</span>
          </button>
          <button className="button button--ghost" type="button" onClick={onClose}>
            <X aria-hidden="true" size={16} strokeWidth={2.2} />
            <span>Close</span>
          </button>
        </div>
      </section>
    </div>
  );
}

type DesktopInfoLiveStatus = "error" | "idle" | "loading" | "paused" | "ready" | "unavailable";

type DesktopInfoMetric = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: DesktopRouteInspector["tone"];
};

function createDesktopInfoFacts({
  activeRouteId,
  activeRouteLabel,
  liveSnapshot,
  liveStatus,
  liveTelemetryAvailable,
  liveTelemetryEnabled
}: {
  activeRouteId: DesktopRouteId;
  activeRouteLabel: string;
  liveSnapshot: LiveResourceSnapshot | null;
  liveStatus: DesktopInfoLiveStatus;
  liveTelemetryAvailable: boolean;
  liveTelemetryEnabled: boolean;
}): Array<[string, string]> {
  return [
    ["Current screen", activeRouteLabel],
    ["Route id", activeRouteId],
    ["Runtime", liveTelemetryAvailable ? "Desktop IPC available" : "Browser preview"],
    ["Live monitor", liveTelemetryEnabled ? "On" : "Off"],
    ["Live status", desktopInfoStatusLabel(liveStatus)],
    ["Last reading", liveSnapshot ? formatSnapshotTime(liveSnapshot.collectedAtUtc) : "No reading yet"],
    ["Data source", liveSnapshot ? formatSnapshotSource(liveSnapshot.source) : "No live sample"],
    ["Requester", liveSnapshot?.requester ?? "No live sample"]
  ];
}

function createDesktopInfoLiveMetrics(snapshot: LiveResourceSnapshot): DesktopInfoMetric[] {
  const cpuDetail = [
    formatProcessorName(snapshot.cpu.name),
    snapshot.cpu.logicalProcessors ? `${snapshot.cpu.logicalProcessors} logical processors` : null,
    snapshot.cpu.maxClockMhz ? formatClock(snapshot.cpu.maxClockMhz) : null
  ]
    .filter(Boolean)
    .join(" - ");
  const storageDetail = [
    snapshot.disk.primaryVolume ?? "Storage volume",
    snapshot.disk.health ? `health ${snapshot.disk.health}` : null,
    snapshot.disk.bytesPerSecond !== null ? `${formatBytesPerSecond(snapshot.disk.bytesPerSecond)} activity` : null
  ]
    .filter(Boolean)
    .join(" - ");
  const networkDetail = [
    snapshot.network.adapterName ??
      `${snapshot.network.activeAdapters} active adapter${snapshot.network.activeAdapters === 1 ? "" : "s"}`,
    snapshot.network.linkSpeedBitsPerSecond ? `${formatBitsPerSecond(snapshot.network.linkSpeedBitsPerSecond)} link` : null
  ]
    .filter(Boolean)
    .join(" - ");

  return [
    {
      detail: cpuDetail || "Windows CPU counter",
      id: "cpu",
      label: "CPU",
      tone: toneForPercent(snapshot.cpu.usagePercent, 85, "success"),
      value: formatPercent(snapshot.cpu.usagePercent)
    },
    {
      detail: formatMemoryDetail(snapshot.memory.usedBytes, snapshot.memory.totalBytes),
      id: "memory",
      label: "Memory",
      tone: toneForPercent(snapshot.memory.usedPercent, 82, "benchmark"),
      value: formatPercent(snapshot.memory.usedPercent)
    },
    {
      detail: storageDetail || "Windows storage counter",
      id: "storage",
      label: "Storage",
      tone: toneForPercent(snapshot.disk.usedPercent, 88, "rollback"),
      value:
        snapshot.disk.bytesPerSecond !== null
          ? formatBytesPerSecond(snapshot.disk.bytesPerSecond)
          : formatPercent(snapshot.disk.usedPercent)
    },
    {
      detail: networkDetail || "Windows network counter",
      id: "network",
      label: "Network",
      tone: "trust",
      value: formatBytesPerSecond(snapshot.network.bytesPerSecond)
    }
  ];
}

function desktopInfoStatusLabel(status: DesktopInfoLiveStatus) {
  if (status === "ready") return "Live";
  if (status === "loading" || status === "idle") return "Reading";
  if (status === "paused") return "Paused";
  if (status === "error") return "Unavailable";

  return "Browser preview";
}

function desktopInfoStatusDetail(
  status: DesktopInfoLiveStatus,
  snapshot: LiveResourceSnapshot | null,
  error: string | null
) {
  if (status === "ready" && snapshot) {
    return `Updated ${formatSnapshotTime(snapshot.collectedAtUtc)} from ${formatSnapshotSource(snapshot.source)}.`;
  }

  if (status === "paused") {
    return "Live resource monitor is off in Settings.";
  }

  if (status === "error") {
    return `Windows counters are unavailable: ${formatTelemetryError(error)}.`;
  }

  if (status === "unavailable") {
    return "Desktop runtime is not connected, so the modal is not showing demo hardware.";
  }

  return "Reading read-only Windows counters.";
}

function toneForDesktopInfoStatus(status: DesktopInfoLiveStatus): DesktopRouteInspector["tone"] {
  if (status === "ready") return "success";
  if (status === "error" || status === "unavailable") return "warning";
  if (status === "paused") return "neutral";

  return "active";
}

function toneForPercent(
  value: number | null | undefined,
  warningThreshold: number,
  normalTone: DesktopRouteInspector["tone"]
): DesktopRouteInspector["tone"] {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "neutral";
  }

  return value >= warningThreshold ? "warning" : normalTone;
}

function formatPercent(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "Unavailable";
  }

  if (value > 0 && value < 1) {
    return "<1%";
  }

  return value < 10 ? `${value.toFixed(1)}%` : `${Math.round(value)}%`;
}

function formatClock(valueMhz: number) {
  if (valueMhz >= 1000) {
    return `${(valueMhz / 1000).toFixed(2)} GHz`;
  }

  return `${Math.round(valueMhz)} MHz`;
}

function formatBytes(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "Unavailable";
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
    return "Unavailable";
  }

  return `${formatBytes(value)}/s`;
}

function formatBitsPerSecond(value: number | null | undefined) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "Unavailable";
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
    return "Windows memory counter";
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

function formatSnapshotSource(value: string) {
  return value.replace(/[-_]/g, " ");
}

function formatTelemetryError(value: string | null) {
  if (!value) {
    return "no diagnostic returned";
  }

  return value.length > 140 ? `${value.slice(0, 137)}...` : value;
}

function NavButton({
  item,
  active,
  onSelect
}: {
  item: DesktopNavigationItem;
  active: boolean;
  onSelect: (id: DesktopRouteId) => void;
}) {
  const Icon = navigationIcons[item.icon];

  return (
    <button
      aria-current={active ? "page" : undefined}
      aria-label={item.group ? `${item.group} ${item.label}` : item.label}
      className="nav-button"
      data-active={active}
      data-tone={item.tone}
      onClick={() => onSelect(item.id)}
      title={item.group ? tOptimizer("navigation.titleWithGroup", { group: item.group, label: item.label }) : item.label}
      type="button"
    >
      <span className="nav-button__icon-frame">
        <Icon aria-hidden="true" className="nav-button__icon" size={19} strokeWidth={2.25} />
      </span>
      <span className="nav-button__body">
        {item.group ? <span className="nav-button__group">{item.group}</span> : null}
        <span className="nav-button__label">{item.label}</span>
        <span className="nav-button__summary">{item.summary}</span>
      </span>
    </button>
  );
}

function getInitialActiveView(): DesktopRouteId {
  return getRouteIdFromHash() ?? defaultOptimizationRouteId;
}

function getRouteIdFromHash(): DesktopRouteId | null {
  const hashValue = window.location.hash.replace(/^#/, "");

  return isDesktopRouteId(hashValue) ? hashValue : null;
}

function createCommandHeaderActions(routeId: DesktopRouteId, labels: string[]): CoreAction[] {
  return labels.map((label, index) => ({
    icon: actionIconForLabel(label),
    id: `${routeId}-command-${index}`,
    label,
    tooltip: label,
    variant: getHeaderActionVariant(index)
  }));
}

function getHeaderActionVariant(index: number): CoreActionVariant {
  if (index === 0) {
    return "primary";
  }

  if (index === 1) {
    return "secondary";
  }

  return "ghost";
}

function createCommandHeaderTrustItems(): CommandHeaderTrustItem[] {
  const updater = desktopCommandCenterState.statusStrip.find((item) => item.id === "updater");
  const backups = desktopCommandCenterState.statusStrip.find((item) => item.id === "backups");

  return [
    {
      detail: updater?.detail ?? tOptimizer("brand.signedBy"),
      icon: "shield-check",
      id: "signature",
      label: tOptimizer("labels.trust"),
      tone: "trust",
      value: tOptimizer("brand.signedBy")
    },
    {
      detail: backups?.detail ?? tOptimizer("labels.available"),
      icon: "history",
      id: "rollback",
      label: tOptimizer(optimizerRollbackKey),
      tone: "rollback",
      value: backups?.value ?? tOptimizer("labels.available")
    }
  ];
}

async function runWindowCommand(command: "close" | "minimize" | "toggleMaximize") {
  if (!hasTauriRuntime()) {
    return;
  }

  const appWindow = getCurrentWindow();

  if (command === "close") {
    await appWindow.close();
    return;
  }

  if (command === "minimize") {
    await appWindow.minimize();
    return;
  }

  await appWindow.toggleMaximize();
}

function hasTauriRuntime() {
  return typeof window !== "undefined" && Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__");
}

const optimizerRollbackKey = "glossary.rollback";
