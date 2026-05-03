import { liiiraaTokens } from "@liiiraa/ui/tokens";

const sharedColors = liiiraaTokens.colors;
const sharedComponents = liiiraaTokens.components;
const sharedShadow = liiiraaTokens.shadow;
const sharedTypography = liiiraaTokens.typography;
const sharedSpacing = liiiraaTokens.spacing;
const sharedRadius = liiiraaTokens.radius;
const sharedMotion = liiiraaTokens.motion;

export const desktopDesignTokens = {
  meta: {
    product: "Liiiraa Booster",
    theme: "graphite performance console",
    version: "0.2.0"
  },
  color: {
    background: {
      app: sharedColors.background.app,
      rail: sharedColors.surface.sunken,
      workspace: sharedColors.background.appSubtle,
      gridLine: "rgba(148, 163, 184, 0.035)"
    },
    surface: {
      panel: sharedColors.surface.panel,
      panelAlt: sharedColors.surface.panelAlt,
      raised: sharedColors.surface.raised,
      sunken: sharedColors.surface.sunken,
      selected: sharedColors.surface.selected,
      hover: "rgba(32, 42, 53, 0.82)",
      overlay: sharedColors.background.overlay,
      premium: sharedColors.surface.premium,
      lane: sharedColors.surface.lane,
      proof: sharedColors.surface.proof
    },
    border: {
      subtle: sharedColors.border.subtle,
      default: sharedColors.border.default,
      strong: sharedColors.border.strong,
      focus: sharedColors.border.focus
    },
    text: {
      primary: sharedColors.text.primary,
      secondary: sharedColors.text.secondary,
      muted: sharedColors.text.muted,
      inverse: sharedColors.text.inverse,
      disabled: sharedColors.text.disabled
    }
  },
  state: {
    active: sharedColors.status.active,
    success: sharedColors.status.success,
    warning: sharedColors.status.warning,
    danger: sharedColors.status.danger,
    lab: sharedColors.risk.lab,
    neutral: sharedColors.status.neutral,
    trust: sharedColors.status.trust,
    rollback: sharedColors.status.rollback,
    benchmark: sharedColors.status.benchmark,
    locked: sharedColors.status.locked
  },
  stateSurface: {
    active: sharedColors.status.activeSurface,
    success: sharedColors.status.successSurface,
    warning: sharedColors.status.warningSurface,
    danger: sharedColors.status.dangerSurface,
    lab: sharedColors.accent.violetSoft,
    neutral: sharedColors.status.neutralSurface,
    trust: sharedColors.status.trustSurface,
    rollback: sharedColors.status.rollbackSurface,
    benchmark: sharedColors.status.benchmarkSurface,
    locked: sharedColors.status.lockedSurface
  },
  chart: {
    averageFps: sharedColors.chart.fpsAverage,
    onePercentLow: sharedColors.chart.fpsLow,
    pointOnePercentLow: sharedColors.chart.gpu,
    p95FrameTime: sharedColors.chart.frametime,
    droppedFrames: sharedColors.chart.dropped,
    cpu: sharedColors.chart.cpu,
    gpu: sharedColors.chart.gpu,
    grid: sharedColors.chart.grid,
    axis: sharedColors.text.muted,
    track: sharedColors.surface.raised
  },
  typography: {
    fontFamily: {
      ui: sharedTypography.fontFamily.ui,
      metric: sharedTypography.fontFamily.metric
    },
    fontSize: {
      caption: sharedTypography.fontSize.caption,
      control: sharedTypography.fontSize.control,
      body: sharedTypography.fontSize.body,
      label: sharedTypography.fontSize.label,
      title: sharedTypography.fontSize.title,
      section: sharedTypography.fontSize.section,
      display: "2rem"
    },
    lineHeight: sharedTypography.lineHeight,
    fontWeight: sharedTypography.fontWeight,
    letterSpacing: {
      default: "0",
      caps: "0"
    }
  },
  spacing: sharedSpacing,
  radius: {
    none: sharedRadius.none,
    xs: sharedRadius.xs,
    sm: sharedRadius.sm,
    md: sharedRadius.md,
    card: sharedRadius.card,
    pill: sharedRadius.pill,
    round: "50%"
  },
  density: {
    railWidth: "15rem",
    railCompactWidth: "5rem",
    contentMaxWidth: "76rem",
    pagePadding: "1.35rem",
    controlHeight: sharedComponents.button.height.md,
    statusStripHeight: sharedComponents.statusStrip.height,
    metricTileMinHeight: sharedComponents.metricTile.minHeight,
    iconButtonSize: sharedComponents.iconButton.size.sm,
    tableMinWidth: "32rem",
    benchmarkTableMinWidth: "43rem"
  },
  shadow: {
    panel: sharedShadow.panel,
    raised: sharedShadow.raised,
    premium: sharedShadow.premium,
    focus: sharedShadow.focus,
    danger: sharedShadow.danger,
    insetHairline: sharedShadow.insetHairline,
    stateRing: sharedShadow.stateRing
  },
  component: {
    button: sharedComponents.button,
    iconButton: sharedComponents.iconButton,
    segmentedControl: sharedComponents.segmentedControl,
    tab: sharedComponents.tab,
    toggle: sharedComponents.toggle,
    card: sharedComponents.card,
    categoryLane: sharedComponents.categoryLane,
    proofTile: sharedComponents.proofTile,
    trustBadge: sharedComponents.trustBadge,
    stateBadge: sharedComponents.stateBadge,
    benchmarkDelta: sharedComponents.benchmarkDelta,
    drawer: sharedComponents.drawer
  },
  motion: {
    duration: {
      instant: sharedMotion.duration.instant,
      fast: sharedMotion.duration.fast,
      normal: sharedMotion.duration.normal,
      slow: sharedMotion.duration.slow
    },
    easing: sharedMotion.easing
  }
} as const;

export type DesktopDesignTokens = typeof desktopDesignTokens;
export type DesktopStateTone = keyof DesktopDesignTokens["state"];

export const desktopToneCssVars: Record<DesktopStateTone, string> = {
  active: "var(--desktop-state-active)",
  benchmark: "var(--desktop-state-benchmark)",
  danger: "var(--desktop-state-danger)",
  lab: "var(--desktop-state-lab)",
  locked: "var(--desktop-state-locked)",
  neutral: "var(--desktop-state-neutral)",
  rollback: "var(--desktop-state-rollback)",
  success: "var(--desktop-state-success)",
  trust: "var(--desktop-state-trust)",
  warning: "var(--desktop-state-warning)"
};

export const desktopChartCssVars = {
  averageFps: "var(--desktop-chart-average-fps)",
  axis: "var(--desktop-chart-axis)",
  droppedFrames: "var(--desktop-chart-dropped-frames)",
  grid: "var(--desktop-chart-grid)",
  onePercentLow: "var(--desktop-chart-one-percent-low)",
  p95FrameTime: "var(--desktop-chart-p95-frame-time)",
  pointOnePercentLow: "var(--desktop-chart-point-one-percent-low)",
  track: "var(--desktop-chart-track)"
} as const;

export const desktopDesignTokenCssVariables = {
  "--desktop-bg-app": desktopDesignTokens.color.background.app,
  "--desktop-bg-rail": desktopDesignTokens.color.background.rail,
  "--desktop-bg-workspace": desktopDesignTokens.color.background.workspace,
  "--desktop-bg-grid-line": desktopDesignTokens.color.background.gridLine,
  "--desktop-surface-panel": desktopDesignTokens.color.surface.panel,
  "--desktop-surface-panel-alt": desktopDesignTokens.color.surface.panelAlt,
  "--desktop-surface-raised": desktopDesignTokens.color.surface.raised,
  "--desktop-surface-sunken": desktopDesignTokens.color.surface.sunken,
  "--desktop-surface-selected": desktopDesignTokens.color.surface.selected,
  "--desktop-surface-hover": desktopDesignTokens.color.surface.hover,
  "--desktop-surface-overlay": desktopDesignTokens.color.surface.overlay,
  "--desktop-surface-premium": desktopDesignTokens.color.surface.premium,
  "--desktop-surface-lane": desktopDesignTokens.color.surface.lane,
  "--desktop-surface-proof": desktopDesignTokens.color.surface.proof,
  "--desktop-border-subtle": desktopDesignTokens.color.border.subtle,
  "--desktop-border-default": desktopDesignTokens.color.border.default,
  "--desktop-border-strong": desktopDesignTokens.color.border.strong,
  "--desktop-border-focus": desktopDesignTokens.color.border.focus,
  "--desktop-text-primary": desktopDesignTokens.color.text.primary,
  "--desktop-text-secondary": desktopDesignTokens.color.text.secondary,
  "--desktop-text-muted": desktopDesignTokens.color.text.muted,
  "--desktop-text-inverse": desktopDesignTokens.color.text.inverse,
  "--desktop-text-disabled": desktopDesignTokens.color.text.disabled,
  "--desktop-state-active": desktopDesignTokens.state.active,
  "--desktop-state-success": desktopDesignTokens.state.success,
  "--desktop-state-warning": desktopDesignTokens.state.warning,
  "--desktop-state-danger": desktopDesignTokens.state.danger,
  "--desktop-state-lab": desktopDesignTokens.state.lab,
  "--desktop-state-neutral": desktopDesignTokens.state.neutral,
  "--desktop-state-trust": desktopDesignTokens.state.trust,
  "--desktop-state-rollback": desktopDesignTokens.state.rollback,
  "--desktop-state-benchmark": desktopDesignTokens.state.benchmark,
  "--desktop-state-locked": desktopDesignTokens.state.locked,
  "--desktop-state-active-surface": desktopDesignTokens.stateSurface.active,
  "--desktop-state-success-surface": desktopDesignTokens.stateSurface.success,
  "--desktop-state-warning-surface": desktopDesignTokens.stateSurface.warning,
  "--desktop-state-danger-surface": desktopDesignTokens.stateSurface.danger,
  "--desktop-state-lab-surface": desktopDesignTokens.stateSurface.lab,
  "--desktop-state-neutral-surface": desktopDesignTokens.stateSurface.neutral,
  "--desktop-state-trust-surface": desktopDesignTokens.stateSurface.trust,
  "--desktop-state-rollback-surface": desktopDesignTokens.stateSurface.rollback,
  "--desktop-state-benchmark-surface": desktopDesignTokens.stateSurface.benchmark,
  "--desktop-state-locked-surface": desktopDesignTokens.stateSurface.locked,
  "--desktop-chart-average-fps": desktopDesignTokens.chart.averageFps,
  "--desktop-chart-one-percent-low": desktopDesignTokens.chart.onePercentLow,
  "--desktop-chart-point-one-percent-low": desktopDesignTokens.chart.pointOnePercentLow,
  "--desktop-chart-p95-frame-time": desktopDesignTokens.chart.p95FrameTime,
  "--desktop-chart-dropped-frames": desktopDesignTokens.chart.droppedFrames,
  "--desktop-chart-cpu": desktopDesignTokens.chart.cpu,
  "--desktop-chart-gpu": desktopDesignTokens.chart.gpu,
  "--desktop-chart-grid": desktopDesignTokens.chart.grid,
  "--desktop-chart-axis": desktopDesignTokens.chart.axis,
  "--desktop-chart-track": desktopDesignTokens.chart.track,
  "--desktop-font-ui": desktopDesignTokens.typography.fontFamily.ui,
  "--desktop-font-metric": desktopDesignTokens.typography.fontFamily.metric,
  "--desktop-text-caption": desktopDesignTokens.typography.fontSize.caption,
  "--desktop-text-control": desktopDesignTokens.typography.fontSize.control,
  "--desktop-text-body": desktopDesignTokens.typography.fontSize.body,
  "--desktop-text-label": desktopDesignTokens.typography.fontSize.label,
  "--desktop-text-title": desktopDesignTokens.typography.fontSize.title,
  "--desktop-text-section": desktopDesignTokens.typography.fontSize.section,
  "--desktop-text-display": desktopDesignTokens.typography.fontSize.display,
  "--desktop-line-tight": desktopDesignTokens.typography.lineHeight.tight,
  "--desktop-line-control": desktopDesignTokens.typography.lineHeight.control,
  "--desktop-line-body": desktopDesignTokens.typography.lineHeight.body,
  "--desktop-line-display": desktopDesignTokens.typography.lineHeight.display,
  "--desktop-letter-default": desktopDesignTokens.typography.letterSpacing.default,
  "--desktop-letter-caps": desktopDesignTokens.typography.letterSpacing.caps,
  "--desktop-space-0": desktopDesignTokens.spacing["0"],
  "--desktop-space-1": desktopDesignTokens.spacing["1"],
  "--desktop-space-2": desktopDesignTokens.spacing["2"],
  "--desktop-space-3": desktopDesignTokens.spacing["3"],
  "--desktop-space-4": desktopDesignTokens.spacing["4"],
  "--desktop-space-5": desktopDesignTokens.spacing["5"],
  "--desktop-space-6": desktopDesignTokens.spacing["6"],
  "--desktop-space-8": desktopDesignTokens.spacing["8"],
  "--desktop-radius-none": desktopDesignTokens.radius.none,
  "--desktop-radius-xs": desktopDesignTokens.radius.xs,
  "--desktop-radius-sm": desktopDesignTokens.radius.sm,
  "--desktop-radius-md": desktopDesignTokens.radius.md,
  "--desktop-radius-card": desktopDesignTokens.radius.card,
  "--desktop-radius-pill": desktopDesignTokens.radius.pill,
  "--desktop-radius-round": desktopDesignTokens.radius.round,
  "--desktop-density-rail-width": desktopDesignTokens.density.railWidth,
  "--desktop-density-rail-compact-width": desktopDesignTokens.density.railCompactWidth,
  "--desktop-density-content-max-width": desktopDesignTokens.density.contentMaxWidth,
  "--desktop-density-page-padding": desktopDesignTokens.density.pagePadding,
  "--desktop-density-control-height": desktopDesignTokens.density.controlHeight,
  "--desktop-density-status-strip-height": desktopDesignTokens.density.statusStripHeight,
  "--desktop-density-metric-tile-min-height": desktopDesignTokens.density.metricTileMinHeight,
  "--desktop-density-icon-button-size": desktopDesignTokens.density.iconButtonSize,
  "--desktop-density-table-min-width": desktopDesignTokens.density.tableMinWidth,
  "--desktop-density-benchmark-table-min-width": desktopDesignTokens.density.benchmarkTableMinWidth,
  "--desktop-shadow-panel": desktopDesignTokens.shadow.panel,
  "--desktop-shadow-raised": desktopDesignTokens.shadow.raised,
  "--desktop-shadow-premium": desktopDesignTokens.shadow.premium,
  "--desktop-shadow-focus": desktopDesignTokens.shadow.focus,
  "--desktop-shadow-danger": desktopDesignTokens.shadow.danger,
  "--desktop-shadow-inset-hairline": desktopDesignTokens.shadow.insetHairline,
  "--desktop-shadow-state-ring": desktopDesignTokens.shadow.stateRing,
  "--desktop-action-height-sm": desktopDesignTokens.component.button.height.sm,
  "--desktop-action-height-md": desktopDesignTokens.component.button.height.md,
  "--desktop-action-height-lg": desktopDesignTokens.component.button.height.lg,
  "--desktop-action-padding-x-sm": desktopDesignTokens.component.button.paddingX.sm,
  "--desktop-action-padding-x-md": desktopDesignTokens.component.button.paddingX.md,
  "--desktop-action-padding-x-lg": desktopDesignTokens.component.button.paddingX.lg,
  "--desktop-action-gap": desktopDesignTokens.component.button.gap,
  "--desktop-action-min-width": desktopDesignTokens.component.button.minWidth,
  "--desktop-action-icon-size": desktopDesignTokens.component.button.iconSize,
  "--desktop-icon-button-size-sm": desktopDesignTokens.component.iconButton.size.sm,
  "--desktop-icon-button-size-md": desktopDesignTokens.component.iconButton.size.md,
  "--desktop-icon-button-size-lg": desktopDesignTokens.component.iconButton.size.lg,
  "--desktop-segmented-min-height": desktopDesignTokens.component.segmentedControl.minHeight,
  "--desktop-segmented-padding": desktopDesignTokens.component.segmentedControl.padding,
  "--desktop-segmented-gap": desktopDesignTokens.component.segmentedControl.gap,
  "--desktop-tab-height": desktopDesignTokens.component.tab.height,
  "--desktop-tab-padding-x": desktopDesignTokens.component.tab.paddingX,
  "--desktop-toggle-width": desktopDesignTokens.component.toggle.width,
  "--desktop-toggle-height": desktopDesignTokens.component.toggle.height,
  "--desktop-toggle-thumb": desktopDesignTokens.component.toggle.thumb,
  "--desktop-card-padding": desktopDesignTokens.component.card.padding,
  "--desktop-card-gap": desktopDesignTokens.component.card.gap,
  "--desktop-category-lane-min-height": desktopDesignTokens.component.categoryLane.minHeight,
  "--desktop-category-lane-accent-width": desktopDesignTokens.component.categoryLane.accentWidth,
  "--desktop-category-lane-gap": desktopDesignTokens.component.categoryLane.gap,
  "--desktop-category-lane-summary-min-width": desktopDesignTokens.component.categoryLane.summaryMinWidth,
  "--desktop-proof-tile-min-height": desktopDesignTokens.component.proofTile.minHeight,
  "--desktop-proof-tile-metric-min-width": desktopDesignTokens.component.proofTile.metricMinWidth,
  "--desktop-proof-tile-accent-height": desktopDesignTokens.component.proofTile.accentHeight,
  "--desktop-trust-badge-min-height": desktopDesignTokens.component.trustBadge.minHeight,
  "--desktop-trust-badge-min-width": desktopDesignTokens.component.trustBadge.minWidth,
  "--desktop-trust-badge-gap": desktopDesignTokens.component.trustBadge.gap,
  "--desktop-state-badge-min-height": desktopDesignTokens.component.stateBadge.minHeight,
  "--desktop-state-badge-padding-x": desktopDesignTokens.component.stateBadge.paddingX,
  "--desktop-state-badge-gap": desktopDesignTokens.component.stateBadge.gap,
  "--desktop-benchmark-delta-min-height": desktopDesignTokens.component.benchmarkDelta.minHeight,
  "--desktop-benchmark-delta-track-height": desktopDesignTokens.component.benchmarkDelta.trackHeight,
  "--desktop-benchmark-delta-value-min-width": desktopDesignTokens.component.benchmarkDelta.valueMinWidth,
  "--desktop-drawer-width": desktopDesignTokens.component.drawer.width,
  "--desktop-drawer-max-width": desktopDesignTokens.component.drawer.maxWidth,
  "--desktop-drawer-scrim": desktopDesignTokens.component.drawer.scrim,
  "--desktop-motion-fast": desktopDesignTokens.motion.duration.fast,
  "--desktop-motion-normal": desktopDesignTokens.motion.duration.normal,
  "--desktop-ease-standard": desktopDesignTokens.motion.easing.standard,
  "--desktop-ease-emphasized": desktopDesignTokens.motion.easing.emphasized
} as const;

export function applyDesktopDesignTokens(root: HTMLElement = document.documentElement) {
  for (const [name, value] of Object.entries(desktopDesignTokenCssVariables)) {
    root.style.setProperty(name, value);
  }
}
