import { optimizerGlossaryKeys, tOptimizer, type OptimizerLocaleKey } from "./localization";

export type PrimitiveAttributeValue = string | number | boolean | undefined;

export type PrimitiveAttributes = Record<string, PrimitiveAttributeValue>;

export type PrimitiveElement =
  | "article"
  | "button"
  | "div"
  | "fieldset"
  | "legend"
  | "output"
  | "section"
  | "span";

export type PrimitiveKind =
  | "benchmark-delta"
  | "button"
  | "card"
  | "category-lane"
  | "drawer"
  | "icon-button"
  | "metric-tile"
  | "mode-segmented-control"
  | "proof-tile"
  | "risk-badge"
  | "state-badge"
  | "status-strip"
  | "tab-list"
  | "toolbar"
  | "toggle"
  | "tooltip"
  | "trust-badge";

export type PrimitiveSize = "sm" | "md" | "lg";
export type PrimitiveDensity = "compact" | "comfortable";
export type PrimitiveTone =
  | "neutral"
  | "success"
  | "active"
  | "warning"
  | "danger"
  | "lab"
  | "trust"
  | "rollback"
  | "benchmark"
  | "locked";
export type ButtonVariant =
  | "primary"
  | "secondary"
  | "ghost"
  | "danger"
  | "destructive"
  | "rollback"
  | "locked"
  | "success";
export type RiskLevel = "low" | "medium" | "high" | "critical" | "lab";
export type OptimizationMode = "safe" | "competitive" | "lab" | "blocked";

export type PrimitiveIconName =
  | "activity"
  | "ban"
  | "check"
  | "chevron-down"
  | "circle"
  | "flask-conical"
  | "gauge"
  | "history"
  | "info"
  | "lock"
  | "octagon-alert"
  | "play"
  | "rotate-ccw"
  | "shield"
  | "shield-check"
  | "sliders-horizontal"
  | "triangle-alert"
  | "zap";

export interface PrimitivePart {
  element: PrimitiveElement;
  className: string;
  attributes?: PrimitiveAttributes;
  content?: string;
  parts?: Record<string, PrimitivePart>;
}

export interface PrimitiveDefinition extends PrimitivePart {
  id: string;
  primitive: PrimitiveKind;
}

export interface InteractivePrimitiveState {
  disabled?: boolean;
  busy?: boolean;
  locked?: boolean;
  successful?: boolean;
  selected?: boolean;
  pressed?: boolean;
  expanded?: boolean;
  invalid?: boolean;
}

export interface TooltipPrimitive extends PrimitiveDefinition {
  primitive: "tooltip";
}

export interface PrimitiveA11yIssue {
  id: string;
  message: string;
  severity: "error" | "warning";
}

export interface ButtonPrimitiveOptions {
  id?: string;
  label: string;
  variant?: ButtonVariant;
  size?: PrimitiveSize;
  fullWidth?: boolean;
  leadingIcon?: PrimitiveIconName;
  trailingIcon?: PrimitiveIconName;
  state?: InteractivePrimitiveState;
  describedBy?: string;
}

export interface IconButtonPrimitiveOptions {
  id?: string;
  label: string;
  icon: PrimitiveIconName;
  tooltip: string;
  size?: PrimitiveSize;
  variant?: Exclude<ButtonVariant, "primary">;
  state?: InteractivePrimitiveState;
}

export interface RiskBadgePrimitiveOptions {
  id?: string;
  level: RiskLevel;
  label?: string;
  detail?: string;
}

export interface ModeSegmentedControlOption {
  value: OptimizationMode;
  label: string;
  description: string;
  icon: PrimitiveIconName;
  disabled?: boolean;
}

export interface ModeSegmentedControlPrimitiveOptions {
  id?: string;
  label: string;
  value: OptimizationMode;
  options?: ModeSegmentedControlOption[];
  density?: PrimitiveDensity;
}

export interface StatusStripItem {
  id: string;
  label: string;
  value: string;
  tone?: PrimitiveTone;
  icon?: PrimitiveIconName;
}

export interface StatusStripPrimitiveOptions {
  id?: string;
  label: string;
  items: StatusStripItem[];
  busy?: boolean;
}

export interface MetricTilePrimitiveOptions {
  id?: string;
  label: string;
  value: string;
  unit?: string;
  description?: string;
  delta?: string;
  tone?: PrimitiveTone;
  loading?: boolean;
}

export interface ToolbarPrimitiveOptions {
  id?: string;
  label: string;
  actions: IconButtonPrimitiveOptions[];
}

export interface TabPrimitiveOption {
  id: string;
  label: string;
  panelId: string;
  icon?: PrimitiveIconName;
  disabled?: boolean;
}

export interface TabListPrimitiveOptions {
  id?: string;
  label: string;
  value: string;
  tabs: TabPrimitiveOption[];
}

export interface TogglePrimitiveOptions {
  id?: string;
  label: string;
  pressed: boolean;
  description?: string;
  disabled?: boolean;
}

export interface CardPrimitiveOptions {
  id?: string;
  label: string;
  eyebrow?: string;
  title: string;
  description?: string;
  tone?: PrimitiveTone;
  actions?: ButtonPrimitiveOptions[];
}

export interface StateBadgePrimitiveOptions {
  id?: string;
  label: string;
  tone?: PrimitiveTone;
  icon?: PrimitiveIconName;
  detail?: string;
}

export interface TrustBadgePrimitiveOptions {
  id?: string;
  label: string;
  value: string;
  detail: string;
  icon?: PrimitiveIconName;
  tone?: "trust" | "rollback" | "benchmark" | "success" | "active";
}

export interface BenchmarkDeltaPrimitiveOptions {
  id?: string;
  label: string;
  before: string;
  after: string;
  delta: string;
  tone?: "success" | "active" | "benchmark" | "warning";
  width?: number;
}

export interface ProofTilePrimitiveOptions {
  id?: string;
  label: string;
  metric: string;
  detail: string;
  tone?: "success" | "active" | "benchmark" | "warning" | "trust";
  sourceLabel?: string;
}

export interface CategoryLanePrimitiveOptions {
  id?: string;
  title: string;
  summary: string;
  status: string;
  trustSignal: string;
  primaryAction: ButtonPrimitiveOptions;
  detailAction?: ButtonPrimitiveOptions;
  tone?: PrimitiveTone;
  icon?: PrimitiveIconName;
}

export interface DrawerPrimitiveOptions {
  id?: string;
  label: string;
  title: string;
  description?: string;
  open?: boolean;
  tone?: PrimitiveTone;
  actions?: ButtonPrimitiveOptions[];
}

const focusClass =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--liiiraa-color-focus)] focus-visible:shadow-[var(--liiiraa-shadow-focus)]";

const disabledClass =
  "disabled:cursor-not-allowed disabled:border-[var(--liiiraa-color-border-subtle)] disabled:bg-[var(--liiiraa-color-surface-disabled)] disabled:text-[var(--liiiraa-color-text-disabled)] disabled:opacity-70";

const transitionClass =
  "transition-[background,border-color,color,box-shadow,transform] duration-[var(--liiiraa-motion-fast)] ease-[var(--liiiraa-ease-standard)] motion-reduce:transition-none";

const srOnlyClass =
  "sr-only absolute h-px w-px overflow-hidden whitespace-nowrap border-0 p-0 [clip:rect(0,0,0,0)]";

const toneClassNames: Record<PrimitiveTone, string> = {
  neutral:
    "border-[var(--liiiraa-color-border-default)] bg-[var(--liiiraa-color-surface-raised)] text-[var(--liiiraa-color-text-secondary)]",
  success:
    "border-[var(--color-liiiraa-success)] bg-[var(--color-liiiraa-success-surface)] text-[var(--color-liiiraa-success)]",
  active:
    "border-[var(--color-liiiraa-active)] bg-[var(--color-liiiraa-active-surface)] text-[var(--color-liiiraa-active)]",
  warning:
    "border-[var(--color-liiiraa-warning)] bg-[var(--color-liiiraa-warning-surface)] text-[var(--color-liiiraa-warning)]",
  danger:
    "border-[var(--color-liiiraa-danger)] bg-[var(--color-liiiraa-danger-surface)] text-[var(--color-liiiraa-danger)]",
  lab:
    "border-[var(--color-liiiraa-violet)] bg-[var(--color-liiiraa-violet-soft)] text-[var(--color-liiiraa-violet)]",
  trust:
    "border-[var(--color-liiiraa-trust)] bg-[var(--color-liiiraa-trust-surface)] text-[var(--color-liiiraa-trust)]",
  rollback:
    "border-[var(--color-liiiraa-rollback)] bg-[var(--color-liiiraa-rollback-surface)] text-[var(--color-liiiraa-rollback)]",
  benchmark:
    "border-[var(--color-liiiraa-benchmark)] bg-[var(--color-liiiraa-benchmark-surface)] text-[var(--color-liiiraa-benchmark)]",
  locked:
    "border-[var(--liiiraa-color-border-subtle)] bg-[var(--color-liiiraa-locked-surface)] text-[var(--color-liiiraa-locked)]"
};

const buttonVariantClassNames: Record<ButtonVariant, string> = {
  primary:
    "border-[var(--color-liiiraa-telemetry)] bg-[var(--color-liiiraa-telemetry)] text-[var(--liiiraa-color-background-app)] hover:bg-[var(--color-liiiraa-performance)] hover:border-[var(--color-liiiraa-performance)]",
  secondary:
    "border-[var(--liiiraa-color-border-default)] bg-[var(--liiiraa-color-surface-raised)] text-[var(--liiiraa-color-text-primary)] hover:border-[var(--liiiraa-color-border-strong)] hover:bg-[var(--liiiraa-color-surface-panel-alt)]",
  ghost:
    "border-transparent bg-transparent text-[var(--liiiraa-color-text-secondary)] hover:border-[var(--liiiraa-color-border-subtle)] hover:bg-[var(--liiiraa-color-surface-panel)] hover:text-[var(--liiiraa-color-text-primary)]",
  danger:
    "border-[var(--color-liiiraa-danger)] bg-[var(--color-liiiraa-danger-surface)] text-[var(--color-liiiraa-danger)] hover:bg-[var(--color-liiiraa-danger)] hover:text-[var(--liiiraa-color-background-app)]",
  destructive:
    "border-[var(--color-liiiraa-danger)] bg-[var(--color-liiiraa-danger-surface)] text-[var(--color-liiiraa-danger)] hover:bg-[var(--color-liiiraa-danger)] hover:text-[var(--liiiraa-color-background-app)]",
  rollback:
    "border-[var(--color-liiiraa-rollback)] bg-[var(--color-liiiraa-rollback-surface)] text-[var(--color-liiiraa-rollback)] hover:bg-[var(--color-liiiraa-rollback)] hover:text-[var(--liiiraa-color-background-app)]",
  locked:
    "border-[var(--liiiraa-color-border-subtle)] bg-[var(--color-liiiraa-locked-surface)] text-[var(--liiiraa-color-text-disabled)]",
  success:
    "border-[var(--color-liiiraa-success)] bg-[var(--color-liiiraa-success-surface)] text-[var(--color-liiiraa-success)]"
};

const buttonSizeClassNames: Record<PrimitiveSize, string> = {
  sm: "min-h-8 px-2.5 text-[length:var(--text-liiiraa-caption)]",
  md: "min-h-9 px-3 text-[length:var(--text-liiiraa-control)]",
  lg: "min-h-10 px-4 text-[length:var(--text-liiiraa-body)]"
};

const iconButtonSizeClassNames: Record<PrimitiveSize, string> = {
  sm: "size-8",
  md: "size-9",
  lg: "size-10"
};

const riskMeta: Record<
  RiskLevel,
  { labelKey: OptimizerLocaleKey; tone: PrimitiveTone; icon: PrimitiveIconName; shape: string }
> = {
  low: { labelKey: "risk.low", tone: "success", icon: "shield-check", shape: "rounded-[var(--radius-liiiraa-sm)]" },
  medium: { labelKey: "risk.medium", tone: "active", icon: "info", shape: "rounded-[var(--radius-liiiraa-sm)]" },
  high: { labelKey: "risk.high", tone: "warning", icon: "triangle-alert", shape: "rounded-[var(--radius-liiiraa-md)]" },
  critical: { labelKey: "risk.critical", tone: "danger", icon: "octagon-alert", shape: "rounded-none" },
  lab: { labelKey: "risk.lab", tone: "lab", icon: "flask-conical", shape: "rounded-full" }
};

export const createDefaultModeOptions = (
  translate: (key: OptimizerLocaleKey) => string = tOptimizer
): ModeSegmentedControlOption[] => [
  {
    value: "safe",
    label: translate(optimizerGlossaryKeys.safe),
    description: translate("modes.safeDescription"),
    icon: "shield-check"
  },
  {
    value: "competitive",
    label: translate(optimizerGlossaryKeys.competitive),
    description: translate("modes.competitiveDescription"),
    icon: "zap"
  },
  {
    value: "lab",
    label: translate(optimizerGlossaryKeys.lab),
    description: translate("modes.labDescription"),
    icon: "flask-conical"
  },
  {
    value: "blocked",
    label: translate(optimizerGlossaryKeys.blocked),
    description: translate("modes.blockedDescription"),
    icon: "ban",
    disabled: true
  }
];

export const defaultModeOptions: ModeSegmentedControlOption[] = createDefaultModeOptions();

const compactClasses = (...classes: Array<string | false | null | undefined>) =>
  classes.filter(Boolean).join(" ").replace(/\s+/g, " ").trim();

const toId = (value: string) => {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return slug || "primitive";
};

const interactiveAttributes = (
  state: InteractivePrimitiveState | undefined,
  extras: PrimitiveAttributes = {}
): PrimitiveAttributes => ({
  ...extras,
  "aria-busy": state?.busy ? "true" : undefined,
  "aria-disabled": state?.disabled || state?.locked ? "true" : undefined,
  "aria-expanded": state?.expanded == null ? undefined : String(state.expanded),
  "aria-invalid": state?.invalid ? "true" : undefined,
  "aria-pressed": state?.pressed == null ? undefined : String(state.pressed),
  "aria-selected": state?.selected == null ? undefined : String(state.selected),
  "data-busy": state?.busy ? "true" : undefined,
  "data-disabled": state?.disabled || state?.locked ? "true" : undefined,
  "data-locked": state?.locked ? "true" : undefined,
  "data-selected": state?.selected ? "true" : undefined,
  "data-state": state?.successful ? "success" : state?.busy ? "loading" : undefined
});

const iconPart = (icon: PrimitiveIconName): PrimitivePart => ({
  element: "span",
  className: "inline-flex size-4 shrink-0 items-center justify-center",
  attributes: {
    "aria-hidden": "true",
    "data-liiiraa-icon": icon
  }
});

export function createTooltipPrimitive(id: string, content: string): TooltipPrimitive {
  return {
    id,
    primitive: "tooltip",
    element: "div",
    className:
      "pointer-events-none rounded-[var(--radius-liiiraa-sm)] border border-[var(--liiiraa-color-border-default)] bg-[var(--liiiraa-color-surface-raised)] px-2 py-1 text-[length:var(--text-liiiraa-caption)] text-[var(--liiiraa-color-text-primary)] shadow-[var(--liiiraa-shadow-panel)]",
    attributes: {
      id,
      role: "tooltip",
      "data-liiiraa-primitive": "tooltip"
    },
    content
  };
}

export function createButtonPrimitive(options: ButtonPrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`button-${options.label}`);
  const variant = options.state?.locked ? "locked" : options.state?.successful ? "success" : options.variant ?? "secondary";
  const size = options.size ?? "md";

  return {
    id,
    primitive: "button",
    element: "button",
    className: compactClasses(
      "inline-flex items-center justify-center gap-2 rounded-[var(--radius-liiiraa-md)] border font-[var(--liiiraa-font-ui)] font-semibold tracking-normal",
      buttonSizeClassNames[size],
      buttonVariantClassNames[variant],
      focusClass,
      disabledClass,
      transitionClass,
      options.fullWidth && "w-full"
    ),
    attributes: interactiveAttributes(options.state, {
      id,
      type: "button",
      disabled: options.state?.disabled || options.state?.locked ? true : undefined,
      "aria-describedby": options.describedBy,
      "data-liiiraa-primitive": "button",
      "data-variant": variant
    }),
    parts: {
      ...(options.leadingIcon ? { leadingIcon: iconPart(options.leadingIcon) } : {}),
      label: {
        element: "span",
        className: "min-w-0 truncate",
        content: options.label
      },
      ...(options.trailingIcon ? { trailingIcon: iconPart(options.trailingIcon) } : {})
    }
  };
}

export function createIconButtonPrimitive(options: IconButtonPrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`icon-button-${options.label}`);
  const size = options.size ?? "md";
  const variant = options.state?.locked ? "locked" : options.state?.successful ? "success" : options.variant ?? "ghost";
  const tooltipId = `${id}-tooltip`;

  return {
    id,
    primitive: "icon-button",
    element: "button",
    className: compactClasses(
      "inline-flex shrink-0 items-center justify-center rounded-[var(--radius-liiiraa-md)] border font-[var(--liiiraa-font-ui)]",
      iconButtonSizeClassNames[size],
      buttonVariantClassNames[variant],
      focusClass,
      disabledClass,
      transitionClass
    ),
    attributes: interactiveAttributes(options.state, {
      id,
      type: "button",
      disabled: options.state?.disabled || options.state?.locked ? true : undefined,
      title: options.tooltip,
      "aria-label": options.label,
      "aria-describedby": tooltipId,
      "data-liiiraa-icon-button": "true",
      "data-liiiraa-primitive": "icon-button",
      "data-variant": variant
    }),
    parts: {
      icon: iconPart(options.icon),
      label: {
        element: "span",
        className: srOnlyClass,
        content: options.label
      },
      tooltip: createTooltipPrimitive(tooltipId, options.tooltip)
    }
  };
}

export function createRiskBadgePrimitive(options: RiskBadgePrimitiveOptions): PrimitiveDefinition {
  const meta = riskMeta[options.level];
  const label = options.label ?? tOptimizer(meta.labelKey);
  const id = options.id ?? toId(`risk-${options.level}-${label}`);
  const riskLabel = tOptimizer(optimizerGlossaryKeys.risk);
  const ariaLabel = options.detail
    ? tOptimizer("risk.ariaWithDetail", { detail: options.detail, label, risk: riskLabel })
    : tOptimizer("risk.aria", { label, risk: riskLabel });

  return {
    id,
    primitive: "risk-badge",
    element: "span",
    className: compactClasses(
      "inline-flex min-h-7 items-center gap-1.5 border px-2 py-1 text-[length:var(--text-liiiraa-caption)] font-semibold leading-none",
      toneClassNames[meta.tone],
      meta.shape
    ),
    attributes: {
      id,
      "aria-label": ariaLabel,
      "data-liiiraa-primitive": "risk-badge",
      "data-liiiraa-risk": options.level
    },
    parts: {
      icon: iconPart(meta.icon),
      label: {
        element: "span",
        className: "truncate",
        content: label
      }
    }
  };
}

export function createModeSegmentedControlPrimitive(
  options: ModeSegmentedControlPrimitiveOptions
): PrimitiveDefinition {
  const id = options.id ?? toId(`mode-${options.label}`);
  const modeOptions = options.options ?? defaultModeOptions;
  const density = options.density ?? "compact";
  const checkedCount = modeOptions.filter((option) => option.value === options.value).length;

  if (modeOptions.length === 0) {
    throw new Error("Mode segmented control requires at least one option.");
  }

  if (checkedCount !== 1) {
    throw new Error(`Mode segmented control value must match exactly one option: ${options.value}.`);
  }

  return {
    id,
    primitive: "mode-segmented-control",
    element: "div",
    className: compactClasses(
      "inline-grid rounded-[var(--radius-liiiraa-md)] border border-[var(--liiiraa-color-border-default)] bg-[var(--liiiraa-color-surface-sunken)] p-1",
      density === "compact" ? "grid-flow-col auto-cols-fr gap-1" : "grid-cols-1 gap-1"
    ),
    attributes: {
      id,
      role: "radiogroup",
      "aria-label": options.label,
      "data-liiiraa-primitive": "mode-segmented-control"
    },
    parts: Object.fromEntries(
      modeOptions.map((option) => {
        const selected = option.value === options.value;
        const disabled = Boolean(option.disabled);

        return [
          option.value,
          {
            element: "button",
            className: compactClasses(
              "inline-flex min-h-9 items-center justify-center gap-2 rounded-[var(--radius-liiiraa-sm)] border px-2.5 text-[length:var(--text-liiiraa-control)] font-semibold tracking-normal",
              selected
                ? "border-[var(--liiiraa-color-border-focus)] bg-[var(--liiiraa-color-surface-selected)] text-[var(--liiiraa-color-text-primary)]"
                : "border-transparent bg-transparent text-[var(--liiiraa-color-text-secondary)] hover:border-[var(--liiiraa-color-border-subtle)] hover:bg-[var(--liiiraa-color-surface-panel)]",
              disabled && "cursor-not-allowed text-[var(--liiiraa-color-text-disabled)] opacity-70",
              focusClass,
              transitionClass
            ),
            attributes: {
              id: `${id}-${option.value}`,
              type: "button",
              role: "radio",
              disabled: disabled ? true : undefined,
              "aria-checked": selected ? "true" : "false",
              "aria-disabled": disabled ? "true" : undefined,
              "aria-label": `${option.label}: ${option.description}`,
              "data-liiiraa-mode": option.value,
              "data-selected": selected ? "true" : undefined,
              tabIndex: selected && !disabled ? 0 : -1
            },
            parts: {
              icon: iconPart(option.icon),
              label: {
                element: "span",
                className: "truncate",
                content: option.label
              }
            }
          }
        ];
      })
    )
  };
}

export function createStatusStripPrimitive(options: StatusStripPrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`status-strip-${options.label}`);

  return {
    id,
    primitive: "status-strip",
    element: "section",
    className:
      "grid min-h-[var(--liiiraa-status-strip-height)] grid-cols-[repeat(auto-fit,minmax(8rem,1fr))] items-center gap-2 border-b border-[var(--liiiraa-color-border-subtle)] bg-[var(--liiiraa-color-background-subtle)] px-3 py-2 text-[length:var(--text-liiiraa-control)]",
    attributes: {
      id,
      role: "status",
      "aria-busy": options.busy ? "true" : undefined,
      "aria-label": options.label,
      "aria-live": "polite",
      "data-liiiraa-primitive": "status-strip"
    },
    parts: Object.fromEntries(
      options.items.map((item) => [
        item.id,
        {
          element: "div",
          className: compactClasses(
            "inline-flex min-w-0 items-center gap-2 border-l-2 pl-2",
            toneClassNames[item.tone ?? "neutral"]
          ),
          attributes: {
            "aria-label": `${item.label}: ${item.value}`,
            "data-liiiraa-status-tone": item.tone ?? "neutral"
          },
          parts: {
            ...(item.icon ? { icon: iconPart(item.icon) } : {}),
            label: {
              element: "span",
              className: "truncate text-[var(--liiiraa-color-text-muted)]",
              content: item.label
            },
            value: {
              element: "span",
              className: "truncate font-semibold text-[var(--liiiraa-color-text-primary)]",
              content: item.value
            }
          }
        }
      ])
    )
  };
}

export function createMetricTilePrimitive(options: MetricTilePrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`metric-${options.label}`);
  const tone = options.tone ?? "active";
  const metricValue = options.loading ? tOptimizer("primitives.metric.measuring") : options.value;
  const unitLabel = options.unit ? ` ${options.unit}` : "";
  const ariaLabel = `${options.label}: ${metricValue}${unitLabel}${
    options.delta ? `. ${tOptimizer("primitives.metric.delta")} ${options.delta}` : ""
  }`;

  return {
    id,
    primitive: "metric-tile",
    element: "article",
    className:
      "grid min-h-24 gap-2 rounded-[var(--radius-liiiraa-card)] border border-[var(--liiiraa-color-border-subtle)] bg-[var(--liiiraa-color-surface-panel)] p-3 shadow-[var(--liiiraa-shadow-panel)]",
    attributes: {
      id,
      "aria-busy": options.loading ? "true" : undefined,
      "aria-label": ariaLabel,
      "data-liiiraa-primitive": "metric-tile"
    },
    parts: {
      label: {
        element: "span",
        className: "truncate text-[length:var(--text-liiiraa-caption)] font-semibold text-[var(--liiiraa-color-text-secondary)]",
        content: options.label
      },
      value: {
        element: "output",
        className: compactClasses(
          "text-[length:var(--text-liiiraa-section)] font-bold leading-tight",
          toneClassNames[tone]
        ),
        attributes: {
          "data-liiiraa-metric": "true"
        },
        content: `${metricValue}${unitLabel}`
      },
      ...(options.delta
        ? {
            delta: {
              element: "span",
              className: "text-[length:var(--text-liiiraa-caption)] font-semibold text-[var(--color-liiiraa-success)]",
              content: options.delta
            }
          }
        : {}),
      ...(options.description
        ? {
            description: {
              element: "span",
              className: "text-[length:var(--text-liiiraa-caption)] leading-snug text-[var(--liiiraa-color-text-muted)]",
              content: options.description
            }
          }
        : {})
    }
  };
}

export function createToolbarPrimitive(options: ToolbarPrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`toolbar-${options.label}`);

  return {
    id,
    primitive: "toolbar",
    element: "div",
    className:
      "inline-flex min-h-[var(--liiiraa-toolbar-height)] items-center gap-1 rounded-[var(--radius-liiiraa-md)] border border-[var(--liiiraa-color-border-subtle)] bg-[var(--liiiraa-color-surface-sunken)] p-1",
    attributes: {
      id,
      role: "toolbar",
      "aria-label": options.label,
      "data-liiiraa-primitive": "toolbar"
    },
    parts: Object.fromEntries(
      options.actions.map((action) => {
        const button = createIconButtonPrimitive(action);
        return [button.id, button];
      })
    )
  };
}

export function createTabListPrimitive(options: TabListPrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`tabs-${options.label}`);
  const selectedCount = options.tabs.filter((tab) => tab.id === options.value).length;

  if (options.tabs.length === 0) {
    throw new Error("Tab list requires at least one tab.");
  }

  if (selectedCount !== 1) {
    throw new Error(`Tab list value must match exactly one tab: ${options.value}.`);
  }

  return {
    id,
    primitive: "tab-list",
    element: "div",
    className: "liiiraa-tab-list",
    attributes: {
      id,
      role: "tablist",
      "aria-label": options.label,
      "data-liiiraa-primitive": "tab-list"
    },
    parts: Object.fromEntries(
      options.tabs.map((tab) => {
        const selected = tab.id === options.value;

        return [
          tab.id,
          {
            element: "button",
            className: compactClasses(
              "liiiraa-action min-w-0",
              selected && "border-[var(--liiiraa-color-focus)] bg-[var(--liiiraa-color-surface-selected)]"
            ),
            attributes: {
              id: `${id}-${tab.id}`,
              type: "button",
              role: "tab",
              disabled: tab.disabled ? true : undefined,
              "aria-controls": tab.panelId,
              "aria-selected": selected ? "true" : "false",
              "data-selected": selected ? "true" : undefined,
              "data-variant": selected ? "secondary" : "ghost",
              tabIndex: selected && !tab.disabled ? 0 : -1
            },
            parts: {
              ...(tab.icon ? { icon: iconPart(tab.icon) } : {}),
              label: {
                element: "span",
                className: "min-w-0 truncate",
                content: tab.label
              }
            }
          }
        ];
      })
    )
  };
}

export function createTogglePrimitive(options: TogglePrimitiveOptions): PrimitiveDefinition {
  const id = options.id ?? toId(`toggle-${options.label}`);

  return {
    id,
    primitive: "toggle",
    element: "button",
    className: "liiiraa-toggle",
    attributes: {
      id,
      type: "button",
      role: "switch",
      disabled: options.disabled ? true : undefined,
      title: options.description,
      "aria-checked": options.pressed ? "true" : "false",
      "aria-label": options.label,
      "aria-describedby": options.description ? `${id}-description` : undefined,
      "aria-pressed": options.pressed ? "true" : "false",
      "data-liiiraa-primitive": "toggle"
    },
    parts: {
      label: {
        element: "span",
        className: srOnlyClass,
        content: options.label
      },
      ...(options.description
        ? {
            description: {
              element: "span",
              className: srOnlyClass,
              attributes: {
                id: `${id}-description`
              },
              content: options.description
            }
          }
        : {})
    }
  };
}

export function createStateBadgePrimitive(options: StateBadgePrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "neutral";
  const id = options.id ?? toId(`state-${tone}-${options.label}`);

  return {
    id,
    primitive: "state-badge",
    element: "span",
    className: compactClasses("liiiraa-state-badge", toneClassNames[tone]),
    attributes: {
      id,
      title: options.detail,
      "aria-label": options.detail ? `${options.label}: ${options.detail}` : options.label,
      "data-liiiraa-primitive": "state-badge",
      "data-tone": tone
    },
    parts: {
      ...(options.icon ? { icon: iconPart(options.icon) } : {}),
      label: {
        element: "span",
        className: "min-w-0 truncate",
        content: options.label
      }
    }
  };
}

export function createTrustBadgePrimitive(options: TrustBadgePrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "trust";
  const id = options.id ?? toId(`trust-${options.label}-${options.value}`);

  return {
    id,
    primitive: "trust-badge",
    element: "span",
    className: compactClasses("liiiraa-trust-badge", toneClassNames[tone]),
    attributes: {
      id,
      title: options.detail,
      "aria-label": `${options.label}: ${options.value}. ${options.detail}`,
      "data-liiiraa-primitive": "trust-badge",
      "data-tone": tone
    },
    parts: {
      icon: iconPart(options.icon ?? "shield-check"),
      body: {
        element: "span",
        className: "grid min-w-0 gap-0.5",
        parts: {
          label: {
            element: "span",
            className: "truncate text-[length:var(--text-liiiraa-caption)]",
            content: options.label
          },
          value: {
            element: "span",
            className: "truncate font-semibold text-[var(--liiiraa-color-text-primary)]",
            content: options.value
          }
        }
      }
    }
  };
}

export function createBenchmarkDeltaPrimitive(options: BenchmarkDeltaPrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "benchmark";
  const id = options.id ?? toId(`delta-${options.label}`);
  const width = `${Math.min(100, Math.max(6, Math.round((options.width ?? 0.72) * 100)))}%`;

  return {
    id,
    primitive: "benchmark-delta",
    element: "article",
    className: "liiiraa-benchmark-delta",
    attributes: {
      id,
      "aria-label": `${options.label}: ${options.before} to ${options.after}. ${options.delta}`,
      "data-liiiraa-primitive": "benchmark-delta",
      "data-tone": tone
    },
    parts: {
      body: {
        element: "span",
        className: "grid min-w-0 gap-1",
        parts: {
          label: {
            element: "span",
            className: "truncate text-[length:var(--text-liiiraa-caption)] text-[var(--liiiraa-color-text-muted)]",
            content: options.label
          },
          track: {
            element: "span",
            className: "liiiraa-benchmark-delta__track",
            attributes: {
              "aria-hidden": "true"
            },
            parts: {
              fill: {
                element: "span",
                className: "block h-full bg-[var(--liiiraa-color-benchmark)]",
                attributes: {
                  style: `width: ${width}`
                }
              }
            }
          }
        }
      },
      value: {
        element: "span",
        className: "grid min-w-0 justify-items-end gap-1 font-[var(--liiiraa-font-metric)]",
        parts: {
          after: {
            element: "span",
            className: "text-[var(--liiiraa-color-text-primary)]",
            content: options.after
          },
          delta: {
            element: "span",
            className: compactClasses("text-[length:var(--text-liiiraa-caption)]", toneClassNames[tone]),
            content: options.delta
          }
        }
      }
    }
  };
}

export function createProofTilePrimitive(options: ProofTilePrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "benchmark";
  const id = options.id ?? toId(`proof-${options.label}`);

  return {
    id,
    primitive: "proof-tile",
    element: "article",
    className: "liiiraa-proof-tile",
    attributes: {
      id,
      "aria-label": `${options.label}: ${options.metric}. ${options.detail}`,
      "data-liiiraa-primitive": "proof-tile",
      "data-tone": tone
    },
    parts: {
      label: {
        element: "span",
        className: "text-[length:var(--text-liiiraa-caption)] font-semibold text-[var(--liiiraa-color-text-muted)]",
        content: options.label
      },
      metric: {
        element: "output",
        className: compactClasses("font-[var(--liiiraa-font-metric)] text-[length:var(--text-liiiraa-section)] font-bold", toneClassNames[tone]),
        content: options.metric
      },
      detail: {
        element: "span",
        className: "text-[length:var(--text-liiiraa-caption)] leading-snug text-[var(--liiiraa-color-text-secondary)]",
        content: options.detail
      },
      ...(options.sourceLabel
        ? {
            source: createStateBadgePrimitive({
              label: options.sourceLabel,
              tone: "neutral",
              icon: "info"
            })
          }
        : {})
    }
  };
}

export function createCardPrimitive(options: CardPrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "neutral";
  const id = options.id ?? toId(`card-${options.title}`);

  return {
    id,
    primitive: "card",
    element: "article",
    className: "liiiraa-card",
    attributes: {
      id,
      "aria-label": options.label,
      "data-liiiraa-primitive": "card",
      "data-tone": tone
    },
    parts: {
      ...(options.eyebrow
        ? {
            eyebrow: {
              element: "span",
              className: compactClasses("text-[length:var(--text-liiiraa-caption)] font-semibold uppercase", toneClassNames[tone]),
              content: options.eyebrow
            }
          }
        : {}),
      title: {
        element: "span",
        className: "font-semibold text-[var(--liiiraa-color-text-primary)]",
        content: options.title
      },
      ...(options.description
        ? {
            description: {
              element: "span",
              className: "text-[length:var(--text-liiiraa-caption)] leading-snug text-[var(--liiiraa-color-text-secondary)]",
              content: options.description
            }
          }
        : {}),
      ...(options.actions && options.actions.length > 0
        ? {
            actions: {
              element: "div",
              className: "flex flex-wrap gap-2",
              parts: Object.fromEntries(options.actions.map((action) => [action.id ?? toId(action.label), createButtonPrimitive(action)]))
            }
          }
        : {})
    }
  };
}

export function createCategoryLanePrimitive(options: CategoryLanePrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "active";
  const id = options.id ?? toId(`lane-${options.title}`);

  return {
    id,
    primitive: "category-lane",
    element: "article",
    className: "liiiraa-category-lane",
    attributes: {
      id,
      "aria-label": `${options.title}: ${options.summary}`,
      "data-liiiraa-primitive": "category-lane",
      "data-tone": tone
    },
    parts: {
      body: {
        element: "div",
        className: "grid min-w-0 gap-2",
        parts: {
          heading: {
            element: "span",
            className: "inline-flex min-w-0 items-center gap-2 font-semibold text-[var(--liiiraa-color-text-primary)]",
            parts: {
              ...(options.icon ? { icon: iconPart(options.icon) } : {}),
              label: {
                element: "span",
                className: "truncate",
                content: options.title
              }
            }
          },
          summary: {
            element: "span",
            className: "text-[length:var(--text-liiiraa-caption)] leading-snug text-[var(--liiiraa-color-text-secondary)]",
            content: options.summary
          },
          state: {
            element: "div",
            className: "flex flex-wrap gap-2",
            parts: {
              status: createStateBadgePrimitive({ label: options.status, tone, icon: "activity" }),
              trust: createStateBadgePrimitive({ label: options.trustSignal, tone: "trust", icon: "shield-check" })
            }
          }
        }
      },
      actions: {
        element: "div",
        className: "flex flex-wrap items-start justify-end gap-2",
        parts: {
          primary: createButtonPrimitive(options.primaryAction),
          ...(options.detailAction ? { detail: createButtonPrimitive(options.detailAction) } : {})
        }
      }
    }
  };
}

export function createDrawerPrimitive(options: DrawerPrimitiveOptions): PrimitiveDefinition {
  const tone = options.tone ?? "neutral";
  const id = options.id ?? toId(`drawer-${options.title}`);

  return {
    id,
    primitive: "drawer",
    element: "section",
    className: "liiiraa-drawer",
    attributes: {
      id,
      role: "region",
      "aria-label": options.label,
      "data-liiiraa-primitive": "drawer",
      "data-open": options.open ? "true" : "false",
      "data-tone": tone
    },
    parts: {
      header: {
        element: "div",
        className: "grid min-w-0 gap-1",
        parts: {
          title: {
            element: "span",
            className: "font-semibold text-[var(--liiiraa-color-text-primary)]",
            content: options.title
          },
          ...(options.description
            ? {
                description: {
                  element: "span",
                  className: "text-[length:var(--text-liiiraa-caption)] leading-snug text-[var(--liiiraa-color-text-secondary)]",
                  content: options.description
                }
              }
            : {})
        }
      },
      ...(options.actions && options.actions.length > 0
        ? {
            actions: {
              element: "div",
              className: "flex flex-wrap gap-2",
              parts: Object.fromEntries(options.actions.map((action) => [action.id ?? toId(action.label), createButtonPrimitive(action)]))
            }
          }
        : {})
    }
  };
}

export function createPrimitiveStoryFixtures(): PrimitiveDefinition[] {
  return [
    createStatusStripPrimitive({
      label: tOptimizer("primitives.systemStatus"),
      items: [
        {
          id: "signed",
          label: tOptimizer("labels.trust"),
          value: tOptimizer("brand.signedBy"),
          tone: "success",
          icon: "shield-check"
        },
        {
          id: "scan",
          label: tOptimizer(optimizerGlossaryKeys.scan),
          value: tOptimizer("labels.ready"),
          tone: "active",
          icon: "activity"
        },
        {
          id: "rollback",
          label: tOptimizer(optimizerGlossaryKeys.rollback),
          value: tOptimizer("labels.available"),
          tone: "neutral",
          icon: "history"
        }
      ]
    }),
    createMetricTilePrimitive({
      label: "1% low",
      value: "142",
      unit: "FPS",
      delta: "+18%",
      tone: "success",
      description: "Last benchmark comparison with metadata attached."
    }),
    createModeSegmentedControlPrimitive({
      label: tOptimizer("modes.optimizationMode"),
      value: "safe"
    }),
    createRiskBadgePrimitive({
      level: "high",
      detail: "Requires explicit review, backup, and rollback."
    }),
    createToolbarPrimitive({
      label: tOptimizer("primitives.planActions"),
      actions: [
        {
          label: tOptimizer("actions.applySafePlan"),
          icon: "play",
          tooltip: tOptimizer("tooltips.applySafePlan"),
          variant: "secondary"
        },
        {
          label: tOptimizer("actions.restoreAll"),
          icon: "rotate-ccw",
          tooltip: tOptimizer("tooltips.rollbackSession"),
          variant: "ghost"
        },
        {
          label: tOptimizer("actions.inspectLabItems"),
          icon: "flask-conical",
          tooltip: tOptimizer("tooltips.inspectLab"),
          variant: "ghost",
          state: { disabled: true }
        }
      ]
    }),
    createTabListPrimitive({
      label: "Optimizer detail views",
      value: "summary",
      tabs: [
        { id: "summary", label: "Summary", panelId: "panel-summary", icon: "gauge" },
        { id: "advanced", label: "Advanced", panelId: "panel-advanced", icon: "sliders-horizontal" },
        { id: "rollback", label: "Rollback", panelId: "panel-rollback", icon: "history" }
      ]
    }),
    createTogglePrimitive({
      label: "Show lab controls",
      pressed: false,
      description: "Keeps experimental changes out of the default apply path."
    }),
    createCategoryLanePrimitive({
      title: "Game Mode",
      summary: "Detected profile, GPU policy, benchmark prompt, and safe launch-state checks.",
      status: "Ready",
      trustSignal: "Rollback capable",
      tone: "active",
      icon: "gauge",
      primaryAction: {
        label: "Optimize game",
        variant: "primary",
        leadingIcon: "play"
      },
      detailAction: {
        label: "Review details",
        variant: "secondary",
        leadingIcon: "info"
      }
    }),
    createProofTilePrimitive({
      label: "Benchmark preview",
      metric: "+11.8%",
      detail: "Example 1% low delta with hardware context attached before publication.",
      sourceLabel: "Example data"
    }),
    createBenchmarkDeltaPrimitive({
      label: "1% low",
      before: "82 FPS",
      after: "92 FPS",
      delta: "+12.2%",
      width: 0.76
    }),
    createTrustBadgePrimitive({
      label: tOptimizer("labels.trust"),
      value: tOptimizer("brand.signedBy"),
      detail: "Release, catalog, and rollback state share the same trust grammar."
    }),
    createDrawerPrimitive({
      label: "Advanced optimization inspector",
      title: "Before and after values",
      description: "Dense tweak details stay behind a deliberate secondary entry point.",
      actions: [
        {
          label: "Close",
          variant: "ghost"
        },
        {
          label: "Apply reviewed",
          variant: "primary",
          leadingIcon: "check"
        }
      ]
    }),
    createButtonPrimitive({
      label: tOptimizer("actions.exportPlan"),
      variant: "secondary",
      leadingIcon: "sliders-horizontal"
    }),
    createButtonPrimitive({
      label: "Restore previous plan",
      variant: "rollback",
      leadingIcon: "rotate-ccw"
    }),
    createButtonPrimitive({
      label: "Locked until scan completes",
      variant: "locked",
      leadingIcon: "lock",
      state: { locked: true }
    })
  ];
}

const escapeHtml = (value: string) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const renderAttributes = (attributes: PrimitiveAttributes | undefined) => {
  if (!attributes) return "";

  return Object.entries(attributes)
    .filter(([, value]) => value !== undefined && value !== false)
    .map(([name, value]) => {
      if (value === true) return name;
      return `${name}="${escapeHtml(String(value))}"`;
    })
    .join(" ");
};

const renderPart = (part: PrimitivePart): string => {
  const attributes = renderAttributes({
    ...part.attributes,
    class: part.className
  });
  const children: string = Object.values(part.parts ?? {}).map(renderPart).join("");
  const content = part.content ? escapeHtml(part.content) : "";

  return `<${part.element}${attributes ? ` ${attributes}` : ""}>${content}${children}</${part.element}>`;
};

export function renderPrimitiveStoryHtml(definitions = createPrimitiveStoryFixtures()): string {
  const rendered = definitions.map(renderPart).join("");

  return `<section data-liiiraa-story="primitives" aria-label="${escapeHtml(tOptimizer("primitives.storyAria"))}">${rendered}</section>`;
}

const getPartText = (part: PrimitivePart): string => {
  const children = Object.values(part.parts ?? {}).map(getPartText).join(" ");
  return [part.content, children].filter(Boolean).join(" ").trim();
};

const hasAccessibleName = (part: PrimitivePart) =>
  Boolean(part.attributes?.["aria-label"] || part.attributes?.["aria-labelledby"] || getPartText(part));

const visitParts = (
  part: PrimitivePart,
  id: string,
  visitor: (part: PrimitivePart, id: string) => void
) => {
  visitor(part, id);

  for (const [partId, child] of Object.entries(part.parts ?? {})) {
    visitParts(child, `${id}.${partId}`, visitor);
  }
};

export function runPrimitiveA11ySmoke(definitions: PrimitiveDefinition[]): PrimitiveA11yIssue[] {
  const issues: PrimitiveA11yIssue[] = [];

  for (const definition of definitions) {
    visitParts(definition, definition.id, (part, id) => {
      const role = part.attributes?.role;
      const isButton = part.element === "button" || role === "button" || role === "radio";

      if (isButton && !hasAccessibleName(part)) {
        issues.push({
          id,
          message: "Interactive primitives require an accessible name.",
          severity: "error"
        });
      }

      if (part.element === "button" && part.attributes?.type !== "button") {
        issues.push({
          id,
          message: "Button primitives must set type=\"button\" so forms cannot submit accidentally.",
          severity: "error"
        });
      }

      if (part.attributes?.["data-liiiraa-icon-button"] === "true") {
        if (!part.attributes["aria-label"]) {
          issues.push({ id, message: "Icon buttons require aria-label.", severity: "error" });
        }

        if (!part.attributes["aria-describedby"] || !part.attributes.title) {
          issues.push({ id, message: "Icon buttons require tooltip wiring.", severity: "error" });
        }
      }

      if (part.attributes?.["data-liiiraa-risk"] && (!part.parts?.icon || !part.parts?.label)) {
        issues.push({
          id,
          message: "Risk state must include icon and text, not color alone.",
          severity: "error"
        });
      }

      if (role === "radiogroup") {
        const radios = Object.values(part.parts ?? {}).filter((child) => child.attributes?.role === "radio");
        const checked = radios.filter((child) => child.attributes?.["aria-checked"] === "true");

        if (!part.attributes?.["aria-label"] && !part.attributes?.["aria-labelledby"]) {
          issues.push({ id, message: "Radiogroups require a label.", severity: "error" });
        }

        if (checked.length !== 1) {
          issues.push({ id, message: "Radiogroups require exactly one selected option.", severity: "error" });
        }
      }

      if (role === "tablist") {
        const tabs = Object.values(part.parts ?? {}).filter((child) => child.attributes?.role === "tab");
        const selected = tabs.filter((child) => child.attributes?.["aria-selected"] === "true");

        if (!part.attributes?.["aria-label"] && !part.attributes?.["aria-labelledby"]) {
          issues.push({ id, message: "Tab lists require a label.", severity: "error" });
        }

        if (tabs.length === 0 || selected.length !== 1) {
          issues.push({ id, message: "Tab lists require exactly one selected tab.", severity: "error" });
        }
      }

      if (role === "switch" && part.attributes?.["aria-checked"] == null) {
        issues.push({ id, message: "Toggle switches require aria-checked.", severity: "error" });
      }
    });
  }

  return issues;
}

export function assertPrimitiveA11ySmoke(definitions = createPrimitiveStoryFixtures()): void {
  const issues = runPrimitiveA11ySmoke(definitions);

  if (issues.length > 0) {
    const message = issues.map((issue) => `${issue.id}: ${issue.message}`).join("\n");
    throw new Error(`Primitive a11y smoke failed:\n${message}`);
  }
}
