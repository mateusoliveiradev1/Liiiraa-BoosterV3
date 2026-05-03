import {
  Activity,
  Ban,
  BarChart3,
  Check,
  CheckCircle2,
  ChevronRight,
  Circle,
  Clock3,
  DatabaseBackup,
  Download,
  FileText,
  FlaskConical,
  Gauge,
  History,
  Info,
  MonitorCog,
  OctagonAlert,
  Play,
  RefreshCw,
  RotateCcw,
  ScanSearch,
  Settings,
  Shield,
  ShieldCheck,
  SlidersHorizontal,
  TriangleAlert,
  Zap,
  type LucideIcon
} from "lucide-react";
import { useState } from "react";
import { optimizerGlossaryKeys, tOptimizer } from "../../../../packages/ui/src/localization";
import {
  runDesktopAction,
  type DesktopActionCommand,
  type DesktopActionDescriptor
} from "../actionRuntime";
import type { DesktopRouteId } from "../adapters/desktopState";

export type CoreTone =
  | "active"
  | "benchmark"
  | "danger"
  | "lab"
  | "locked"
  | "neutral"
  | "rollback"
  | "success"
  | "trust"
  | "warning";

export type CoreIconName =
  | "activity"
  | "backup"
  | "ban"
  | "benchmark"
  | "check"
  | "chevron-right"
  | "clock"
  | "file"
  | "gauge"
  | "gpu"
  | "history"
  | "info"
  | "lab"
  | "play"
  | "refresh"
  | "rollback"
  | "scan"
  | "settings"
  | "shield"
  | "shield-check"
  | "sliders"
  | "triangle-alert"
  | "zap";

export type CoreActionVariant =
  | "primary"
  | "secondary"
  | "ghost"
  | "danger"
  | "destructive"
  | "rollback"
  | "locked"
  | "success";

export type CoreAction = {
  id: string;
  label: string;
  tooltip: string;
  icon: CoreIconName;
  variant?: CoreActionVariant;
  disabled?: boolean;
} & Partial<Pick<DesktopActionDescriptor, "feedback" | "successFeedback" | "errorFeedback">> & {
    command?: DesktopActionCommand;
    targetRoute?: DesktopRouteId;
  };

export type CommandHeaderNextAction = {
  label: string;
  value: string;
  detail: string;
  tone: CoreTone;
};

export type CommandHeaderTrustItem = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: CoreTone;
  icon?: CoreIconName;
};

export type StatusTelemetryItem = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: CoreTone;
  icon?: CoreIconName;
};

export type MetricReadoutItem = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: CoreTone;
  delta?: string;
};

export type RiskLevel = "low" | "medium" | "high" | "critical" | "lab" | "blocked";

export type ModeSegment = {
  id: string;
  label: string;
  detail: string;
  tone: CoreTone;
  icon: CoreIconName;
  disabled?: boolean;
};

export type TweakLedgerRowData = {
  id: string;
  change: string;
  impact: string;
  confidence: string;
  source: string;
  risk: RiskLevel;
  riskLabel: string;
  reboot: string;
  rollback: string;
  consent: string;
  tone: CoreTone;
};

export type DiffItem = {
  id: string;
  label: string;
  before: string;
  after: string;
  tone?: CoreTone;
};

export type TimelineStep = {
  id: string;
  label: string;
  detail: string;
  state: "active" | "complete" | "pending" | "failed";
  tone?: CoreTone;
};

export type RollbackSessionLogData = {
  id: string;
  time: string;
  label: string;
  state: string;
  summary: string;
  rebootRequired: boolean;
  items: DiffItem[];
  actions?: CoreAction[];
};

export type BenchmarkProofPoint = {
  id: string;
  label: string;
  averageFps: number;
  onePercentLow: number;
  pointOnePercentLow?: number;
  p95FrameMs: number;
  tone: CoreTone;
};

const iconComponents: Record<CoreIconName, LucideIcon> = {
  activity: Activity,
  backup: DatabaseBackup,
  ban: Ban,
  benchmark: BarChart3,
  check: Check,
  "chevron-right": ChevronRight,
  clock: Clock3,
  file: FileText,
  gauge: Gauge,
  gpu: MonitorCog,
  history: History,
  info: Info,
  lab: FlaskConical,
  play: Play,
  refresh: RefreshCw,
  rollback: RotateCcw,
  scan: ScanSearch,
  settings: Settings,
  shield: Shield,
  "shield-check": ShieldCheck,
  sliders: SlidersHorizontal,
  "triangle-alert": TriangleAlert,
  zap: Zap
};

const riskMeta: Record<RiskLevel, { icon: CoreIconName; tone: CoreTone; shape: string }> = {
  blocked: { icon: "ban", tone: "danger", shape: "risk-badge--blocked" },
  critical: { icon: "triangle-alert", tone: "danger", shape: "risk-badge--critical" },
  high: { icon: "triangle-alert", tone: "warning", shape: "risk-badge--high" },
  lab: { icon: "lab", tone: "lab", shape: "risk-badge--lab" },
  low: { icon: "shield-check", tone: "success", shape: "risk-badge--low" },
  medium: { icon: "info", tone: "active", shape: "risk-badge--medium" }
};

export function CommandHeader({
  actions,
  eyebrow,
  nextAction,
  summary,
  title,
  trustItems
}: {
  actions: CoreAction[];
  eyebrow: string;
  nextAction: CommandHeaderNextAction;
  summary: string;
  title: string;
  trustItems: CommandHeaderTrustItem[];
}) {
  const [primaryAction, ...secondaryActions] = actions;

  return (
    <header className="command-header" data-tone={nextAction.tone} aria-label={tOptimizer("commandHeader.aria")}>
      <div className="command-header__identity">
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="command-header__summary">{summary}</p>
      </div>

      <div className="command-header__next" data-tone={nextAction.tone}>
        <span>{tOptimizer("commandHeader.nextAction")}</span>
        <strong>{nextAction.value}</strong>
        <small>{nextAction.detail}</small>
      </div>

      <div className="command-header__controls" aria-label={tOptimizer("commandHeader.controlsAria")}>
        {primaryAction ? <ActionButton action={primaryAction} className="command-header__primary" /> : null}
        {secondaryActions.length > 0 ? (
          <IconToolbar
            actions={secondaryActions}
            label={tOptimizer("commandHeader.secondaryControlsAria")}
          />
        ) : null}
      </div>

      <div className="command-header__trust" aria-label={tOptimizer("commandHeader.trustAria")}>
        {trustItems.map((item) => (
          <TrustBadge
            key={item.id}
            detail={item.detail}
            label={item.label}
            tone={item.tone}
            value={item.value}
            {...(item.icon ? { icon: item.icon } : {})}
          />
        ))}
      </div>
    </header>
  );
}

export function StatusStrip({ items, label }: { items: StatusTelemetryItem[]; label: string }) {
  return (
    <section className="status-strip" aria-label={label} aria-live="polite">
      {items.map((item) => {
        const tooltipId = `status-${item.id}-tooltip`;

        return (
          <span
            className="status-item"
            data-tone={item.tone}
            key={item.id}
            tabIndex={0}
            title={item.detail}
            aria-describedby={tooltipId}
          >
            <IconGlyph className="status-item__icon" name={item.icon ?? inferStatusIcon(item.id)} />
            <span className="status-item__label">{item.label}</span>
            <strong>{item.value}</strong>
            <small>{item.detail}</small>
            <span className="primitive-tooltip" id={tooltipId} role="tooltip">
              {item.detail}
            </span>
          </span>
        );
      })}
    </section>
  );
}

export function MetricReadout({ metric }: { metric: MetricReadoutItem }) {
  return (
    <article className="metric-readout" data-tone={metric.tone} aria-label={`${metric.label}: ${metric.value}`}>
      <span className="metric-readout__label">{metric.label}</span>
      <strong>{metric.value}</strong>
      {metric.delta ? <span className="metric-readout__delta">{metric.delta}</span> : null}
      <small>{metric.detail}</small>
    </article>
  );
}

export function RiskBadge({
  detail,
  label,
  level
}: {
  detail?: string;
  label: string;
  level: RiskLevel;
}) {
  const meta = riskMeta[level];
  const ariaLabel = detail
    ? tOptimizer("risk.ariaWithDetail", { detail, label, risk: tOptimizer(optimizerGlossaryKeys.risk) })
    : tOptimizer("risk.aria", { label, risk: tOptimizer(optimizerGlossaryKeys.risk) });

  return (
    <span
      className={`risk-badge ${meta.shape}`}
      data-tone={meta.tone}
      data-risk={level}
      aria-label={ariaLabel}
      title={detail}
    >
      <IconGlyph name={meta.icon} />
      <span>{label}</span>
    </span>
  );
}

export function ModeSegmentedControl({
  label,
  options,
  value
}: {
  label: string;
  options: ModeSegment[];
  value: string;
}) {
  const [selectedValue, setSelectedValue] = useState(value);

  return (
    <div className="mode-segmented" role="radiogroup" aria-label={label}>
      {options.map((option) => {
        const selected = option.id === selectedValue;

        return (
          <button
            className="mode-segmented__option"
            data-selected={selected}
            data-tone={option.tone}
            disabled={option.disabled}
            key={option.id}
            onClick={() => {
              setSelectedValue(option.id);
              void runDesktopAction({
                feedback: `${option.label} mode selected for review. Apply remains gated by consent and rollback.`,
                id: `mode-${option.id}`,
                label: option.label,
                targetRoute: "optimize"
              });
            }}
            role="radio"
            type="button"
            aria-checked={selected}
            aria-label={`${option.label}: ${option.detail}`}
            title={option.detail}
          >
            <IconGlyph name={option.icon} />
            <span>{option.label}</span>
          </button>
        );
      })}
    </div>
  );
}

export function IconToolbar({ actions, label }: { actions: CoreAction[]; label: string }) {
  return (
    <div className="icon-toolbar" role="toolbar" aria-label={label}>
      {actions.map((action) => (
        <IconButton action={action} key={action.id} />
      ))}
    </div>
  );
}

export function TrustBadge({
  detail,
  icon = "shield-check",
  label,
  tone,
  value
}: {
  detail: string;
  icon?: CoreIconName;
  label: string;
  tone: CoreTone;
  value: string;
}) {
  return (
    <span className="trust-badge" data-tone={tone} title={detail}>
      <IconGlyph name={icon} />
      <span>
        <small>{label}</small>
        <strong>{value}</strong>
      </span>
    </span>
  );
}

export function TweakLedger({ rows }: { rows: TweakLedgerRowData[] }) {
  return (
    <div className="tweak-ledger" role="table" aria-label={tOptimizer("primitives.tweakLedgerAria")}>
      <div className="tweak-ledger__head" role="row">
        <span role="columnheader">{tOptimizer("labels.change")}</span>
        <span role="columnheader">{tOptimizer("labels.impact")}</span>
        <span role="columnheader">{tOptimizer(optimizerGlossaryKeys.confidence)}</span>
        <span role="columnheader">{tOptimizer(optimizerGlossaryKeys.risk)}</span>
        <span role="columnheader">{tOptimizer(optimizerGlossaryKeys.rollback)}</span>
      </div>
      {rows.map((row) => (
        <TweakLedgerRow key={row.id} row={row} />
      ))}
    </div>
  );
}

export function TweakLedgerRow({ row }: { row: TweakLedgerRowData }) {
  return (
    <div className="tweak-ledger__row" data-tone={row.tone} role="row">
      <span className="tweak-ledger__change" role="cell">
        <strong>{row.change}</strong>
        <small>{row.source}</small>
      </span>
      <span role="cell">
        {row.impact}
        <small>{row.consent}</small>
      </span>
      <span role="cell">
        {row.confidence}
        <small>
          {tOptimizer(optimizerGlossaryKeys.source)}: {row.source}
        </small>
      </span>
      <span role="cell">
        <RiskBadge detail={row.reboot} label={row.riskLabel} level={row.risk} />
        <small>
          {tOptimizer(optimizerGlossaryKeys.reboot)}: {row.reboot}
        </small>
      </span>
      <span role="cell">
        {row.rollback}
        <small>{row.consent}</small>
      </span>
    </div>
  );
}

export function DiffPanel({ items, label }: { items: DiffItem[]; label: string }) {
  return (
    <div className="diff-panel" aria-label={label}>
      {items.map((item) => (
        <div className="diff-panel__row" data-tone={item.tone ?? "neutral"} key={item.id}>
          <strong>{item.label}</strong>
          <span>
            <small>{tOptimizer("labels.before")}</small>
            {item.before}
          </span>
          <ChevronRight aria-hidden="true" size={16} strokeWidth={2.2} />
          <span>
            <small>{tOptimizer("labels.after")}</small>
            {item.after}
          </span>
        </div>
      ))}
    </div>
  );
}

export function ApplyTimeline({ label, steps }: { label: string; steps: TimelineStep[] }) {
  return (
    <ol className="apply-timeline" aria-label={label}>
      {steps.map((step) => {
        const tone = step.tone ?? timelineTone(step.state);

        return (
          <li className="apply-timeline__item" data-state={step.state} data-tone={tone} key={step.id}>
            <span className="apply-timeline__marker" aria-hidden="true">
              <TimelineIcon state={step.state} />
            </span>
            <span>
              <strong>{step.label}</strong>
              <small>{step.detail}</small>
            </span>
          </li>
        );
      })}
    </ol>
  );
}

export function RollbackSessionLog({ session }: { session: RollbackSessionLogData }) {
  return (
    <article className="rollback-session-log" data-tone={session.rebootRequired ? "warning" : "rollback"}>
      <div className="rollback-session-log__header">
        <span>
          <small>{session.time}</small>
          <strong>{session.label}</strong>
        </span>
        <RiskBadge
          detail={session.state}
          label={session.rebootRequired ? tOptimizer("workflow.rollback.rebootRequired") : tOptimizer("workflow.rollback.noReboot")}
          level={session.rebootRequired ? "high" : "low"}
        />
      </div>
      <p>{session.summary}</p>
      {session.actions && session.actions.length > 0 ? (
        <IconToolbar
          actions={session.actions}
          label={tOptimizer("workflow.actions.sessionRollbackAria", { session: session.label })}
        />
      ) : null}
      <DiffPanel items={session.items} label={tOptimizer("workflow.rollback.valuesAria")} />
    </article>
  );
}

export function BenchmarkProofChart({ label, points }: { label: string; points: BenchmarkProofPoint[] }) {
  const maxFps = Math.max(...points.flatMap((point) => [point.averageFps, point.onePercentLow, point.pointOnePercentLow ?? 0]), 1);
  const maxFrameMs = Math.max(...points.map((point) => point.p95FrameMs), 1);

  return (
    <div className="benchmark-proof-chart" role="img" aria-label={label}>
      {points.map((point) => (
        <div className="benchmark-proof-chart__row" data-tone={point.tone} key={point.id}>
          <strong>{point.label}</strong>
          <MetricBar label="Avg" tone={point.tone} value={`${point.averageFps} FPS`} width={point.averageFps / maxFps} />
          <MetricBar
            label="1% low"
            tone={point.tone}
            value={`${point.onePercentLow} FPS`}
            width={point.onePercentLow / maxFps}
          />
          <MetricBar
            label="0.1% low"
            tone={point.tone}
            value={`${point.pointOnePercentLow ?? point.onePercentLow} FPS`}
            width={(point.pointOnePercentLow ?? point.onePercentLow) / maxFps}
          />
          <MetricBar
            label="p95"
            tone={point.tone}
            value={`${point.p95FrameMs} ms`}
            width={point.p95FrameMs / maxFrameMs}
          />
        </div>
      ))}
    </div>
  );
}

export function ActionButton({ action, className = "" }: { action: CoreAction; className?: string }) {
  const tooltipId = `${action.id}-tooltip`;

  return (
    <button
      className={`button ${className} button--${action.variant ?? "secondary"}`.trim()}
      disabled={action.disabled}
      onClick={() => void runDesktopAction(action)}
      type="button"
      aria-describedby={tooltipId}
    >
      <IconGlyph name={action.icon} />
      <span>{action.label}</span>
      <span className="primitive-tooltip" id={tooltipId} role="tooltip">
        {action.tooltip}
      </span>
    </button>
  );
}

function IconButton({ action }: { action: CoreAction }) {
  const tooltipId = `${action.id}-tooltip`;

  return (
    <button
      className={`icon-button icon-button--${action.variant ?? "ghost"}`}
      disabled={action.disabled}
      onClick={() => void runDesktopAction(action)}
      title={action.tooltip}
      type="button"
      aria-describedby={tooltipId}
      aria-label={action.label}
    >
      <IconGlyph name={action.icon} />
      <span className="sr-only">{action.label}</span>
      <span className="primitive-tooltip" id={tooltipId} role="tooltip">
        {action.tooltip}
      </span>
    </button>
  );
}

function IconGlyph({
  className,
  name,
  size = 16
}: {
  className?: string;
  name: CoreIconName;
  size?: number;
}) {
  const Icon = iconComponents[name];

  return <Icon aria-hidden="true" className={className} size={size} strokeWidth={2.2} />;
}

function TimelineIcon({ state }: { state: TimelineStep["state"] }) {
  if (state === "complete") {
    return <CheckCircle2 aria-hidden="true" size={15} strokeWidth={2.4} />;
  }

  if (state === "active") {
    return <Activity aria-hidden="true" size={15} strokeWidth={2.4} />;
  }

  if (state === "failed") {
    return <OctagonAlert aria-hidden="true" size={15} strokeWidth={2.4} />;
  }

  return <Circle aria-hidden="true" size={13} strokeWidth={2.4} />;
}

function MetricBar({
  label,
  tone,
  value,
  width
}: {
  label: string;
  tone: CoreTone;
  value: string;
  width: number;
}) {
  return (
    <span className="benchmark-proof-chart__metric" data-tone={tone}>
      <small>{label}</small>
      <span className="benchmark-proof-chart__track" aria-hidden="true">
        <span style={{ width: `${Math.max(8, Math.round(width * 100))}%` }} />
      </span>
      <b>{value}</b>
    </span>
  );
}

function timelineTone(state: TimelineStep["state"]): CoreTone {
  if (state === "complete") {
    return "success";
  }

  if (state === "active") {
    return "active";
  }

  if (state === "failed") {
    return "danger";
  }

  return "neutral";
}

function inferStatusIcon(id: string): CoreIconName {
  if (/scan/i.test(id)) return "scan";
  if (/backup|rollback/i.test(id)) return "history";
  if (/update|trust|sign/i.test(id)) return "shield-check";
  if (/benchmark/i.test(id)) return "benchmark";

  return "activity";
}

export function actionIconForLabel(label: string): CoreIconName {
  if (/backup|back up|copia|copias|respald/i.test(label)) return "backup";
  if (/benchmark|capture|captur|compare|compar/i.test(label)) return "benchmark";
  if (/cancel/i.test(label)) return "ban";
  if (/check|update|verificar|buscar|actualiz/i.test(label)) return "refresh";
  if (/export|folder|data|dados|datos|pasta|carpeta/i.test(label)) return "file";
  if (/gpu|profile|perfil|nvidia/i.test(label)) return "gpu";
  if (/lab|laborat|inspect|inspec/i.test(label)) return "lab";
  if (/apply|boost|aplicar/i.test(label)) return "zap";
  if (/review|stage|plan|revis|prepar/i.test(label)) return "sliders";
  if (/rollback|restore|restaur|revers/i.test(label)) return "rollback";
  if (/scan|anal/i.test(label)) return "scan";

  return "play";
}

export function riskLevelFromLabel(label: string): RiskLevel {
  const normalized = label.toLowerCase();

  if (/blocked|denied/.test(normalized)) return "blocked";
  if (/critical|danger/.test(normalized)) return "critical";
  if (/lab/.test(normalized)) return "lab";
  if (/high/.test(normalized)) return "high";
  if (/medium|moderate|competitive/.test(normalized)) return "medium";

  return "low";
}

export function modeSegmentsFromGroups(
  groups: Array<{ id: string; label: string; summary: string; tone: CoreTone }>
): ModeSegment[] {
  return groups.map((group) => ({
    detail: group.summary,
    disabled: group.id === "blocked",
    icon: group.id === "safe" ? "shield-check" : group.id === "competitive" ? "zap" : group.id === "lab" ? "lab" : "ban",
    id: group.id,
    label: group.label,
    tone: group.tone
  }));
}
