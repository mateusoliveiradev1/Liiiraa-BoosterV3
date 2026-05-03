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
  PanelLeftOpen,
  Radar,
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
              onClick={() =>
                void runDesktopAction({
                  feedback: "Help is routed through visible tooltips, route summaries, and trust settings for now.",
                  id: "help",
                  label: "Help",
                  targetRoute: "settings"
                })
              }
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
