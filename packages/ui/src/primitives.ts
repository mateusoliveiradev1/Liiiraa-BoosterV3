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
  | "button"
  | "icon-button"
  | "metric-tile"
  | "mode-segmented-control"
  | "risk-badge"
  | "status-strip"
  | "toolbar"
  | "tooltip";

export type PrimitiveSize = "sm" | "md" | "lg";
export type PrimitiveDensity = "compact" | "comfortable";
export type PrimitiveTone = "neutral" | "success" | "active" | "warning" | "danger" | "lab";
export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
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
    "border-[var(--color-liiiraa-violet)] bg-[var(--color-liiiraa-violet-soft)] text-[var(--color-liiiraa-violet)]"
};

const buttonVariantClassNames: Record<ButtonVariant, string> = {
  primary:
    "border-[var(--color-liiiraa-telemetry)] bg-[var(--color-liiiraa-telemetry)] text-[var(--liiiraa-color-background-app)] hover:bg-[var(--color-liiiraa-performance)] hover:border-[var(--color-liiiraa-performance)]",
  secondary:
    "border-[var(--liiiraa-color-border-default)] bg-[var(--liiiraa-color-surface-raised)] text-[var(--liiiraa-color-text-primary)] hover:border-[var(--liiiraa-color-border-strong)] hover:bg-[var(--liiiraa-color-surface-panel-alt)]",
  ghost:
    "border-transparent bg-transparent text-[var(--liiiraa-color-text-secondary)] hover:border-[var(--liiiraa-color-border-subtle)] hover:bg-[var(--liiiraa-color-surface-panel)] hover:text-[var(--liiiraa-color-text-primary)]",
  danger:
    "border-[var(--color-liiiraa-danger)] bg-[var(--color-liiiraa-danger-surface)] text-[var(--color-liiiraa-danger)] hover:bg-[var(--color-liiiraa-danger)] hover:text-[var(--liiiraa-color-background-app)]"
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
  { label: string; tone: PrimitiveTone; icon: PrimitiveIconName; shape: string }
> = {
  low: { label: "Low risk", tone: "success", icon: "shield-check", shape: "rounded-[var(--radius-liiiraa-sm)]" },
  medium: { label: "Moderate risk", tone: "active", icon: "info", shape: "rounded-[var(--radius-liiiraa-sm)]" },
  high: { label: "High risk", tone: "warning", icon: "triangle-alert", shape: "rounded-[var(--radius-liiiraa-md)]" },
  critical: { label: "Critical risk", tone: "danger", icon: "octagon-alert", shape: "rounded-none" },
  lab: { label: "Lab only", tone: "lab", icon: "flask-conical", shape: "rounded-full" }
};

export const defaultModeOptions: ModeSegmentedControlOption[] = [
  {
    value: "safe",
    label: "Safe",
    description: "Low-risk reversible changes only.",
    icon: "shield-check"
  },
  {
    value: "competitive",
    label: "Competitive",
    description: "Performance tradeoffs with explicit review.",
    icon: "zap"
  },
  {
    value: "lab",
    label: "Lab",
    description: "Experimental changes behind per-category opt-in.",
    icon: "flask-conical"
  },
  {
    value: "blocked",
    label: "Blocked",
    description: "Educational items that cannot be applied.",
    icon: "ban",
    disabled: true
  }
];

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
  "aria-disabled": state?.disabled ? "true" : undefined,
  "aria-expanded": state?.expanded == null ? undefined : String(state.expanded),
  "aria-invalid": state?.invalid ? "true" : undefined,
  "aria-pressed": state?.pressed == null ? undefined : String(state.pressed),
  "aria-selected": state?.selected == null ? undefined : String(state.selected),
  "data-busy": state?.busy ? "true" : undefined,
  "data-disabled": state?.disabled ? "true" : undefined,
  "data-selected": state?.selected ? "true" : undefined
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
  const variant = options.variant ?? "secondary";
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
      disabled: options.state?.disabled ? true : undefined,
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
  const variant = options.variant ?? "ghost";
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
      disabled: options.state?.disabled ? true : undefined,
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
  const label = options.label ?? meta.label;
  const id = options.id ?? toId(`risk-${options.level}-${label}`);
  const ariaLabel = options.detail ? `Risk: ${label}. ${options.detail}` : `Risk: ${label}`;

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
  const metricValue = options.loading ? "Measuring" : options.value;
  const unitLabel = options.unit ? ` ${options.unit}` : "";
  const ariaLabel = `${options.label}: ${metricValue}${unitLabel}${
    options.delta ? `. Delta ${options.delta}` : ""
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

export function createPrimitiveStoryFixtures(): PrimitiveDefinition[] {
  return [
    createStatusStripPrimitive({
      label: "System status",
      items: [
        { id: "signed", label: "Trust", value: "Signed by Liiiraa", tone: "success", icon: "shield-check" },
        { id: "scan", label: "Scan", value: "Ready", tone: "active", icon: "activity" },
        { id: "rollback", label: "Rollback", value: "Available", tone: "neutral", icon: "history" }
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
      label: "Optimization mode",
      value: "safe"
    }),
    createRiskBadgePrimitive({
      level: "high",
      detail: "Requires explicit review, backup, and rollback."
    }),
    createToolbarPrimitive({
      label: "Plan actions",
      actions: [
        { label: "Apply safe plan", icon: "play", tooltip: "Apply only safe reversible changes.", variant: "secondary" },
        { label: "Rollback session", icon: "rotate-ccw", tooltip: "Restore the previous optimization session.", variant: "ghost" },
        {
          label: "Inspect lab items",
          icon: "flask-conical",
          tooltip: "Open Lab-only recommendations without applying them.",
          variant: "ghost",
          state: { disabled: true }
        }
      ]
    }),
    createButtonPrimitive({
      label: "Export plan",
      variant: "secondary",
      leadingIcon: "sliders-horizontal"
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

const renderPart = (part: PrimitivePart) => {
  const attributes = renderAttributes({
    ...part.attributes,
    class: part.className
  });
  const children = Object.values(part.parts ?? {}).map(renderPart).join("");
  const content = part.content ? escapeHtml(part.content) : "";

  return `<${part.element}${attributes ? ` ${attributes}` : ""}>${content}${children}</${part.element}>`;
};

export function renderPrimitiveStoryHtml(definitions = createPrimitiveStoryFixtures()): string {
  const rendered = definitions.map(renderPart).join("");

  return `<section data-liiiraa-story="primitives" aria-label="Liiiraa primitive story render">${rendered}</section>`;
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
