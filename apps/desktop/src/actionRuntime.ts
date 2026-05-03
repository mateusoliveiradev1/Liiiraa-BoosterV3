import { invoke, isTauri } from "@tauri-apps/api/core";
import type { DesktopRouteId } from "./adapters/desktopState";

export type DesktopActionCommand =
  | "apply-safe-plan"
  | "benchmark"
  | "check-signed-update"
  | "confirm-required"
  | "export-audit"
  | "export-local-data"
  | "export-plan"
  | "export-report"
  | "open-data-folder"
  | "review-plan"
  | "rollback"
  | "run-read-only-scan"
  | "snapshot-config"
  | "stage-profile"
  | "status";

export type DesktopActionTone = "active" | "danger" | "neutral" | "rollback" | "success" | "warning";

export type DesktopActionDescriptor = {
  id: string;
  label: string;
  command?: DesktopActionCommand;
  targetRoute?: DesktopRouteId;
  feedback?: string;
  successFeedback?: string;
  errorFeedback?: string;
};

export type DesktopActionFeedback = {
  id: string;
  detail: string;
  label: string;
  tone: DesktopActionTone;
};

export const desktopActionFeedbackEvent = "liiiraa:desktop-action-feedback";

type NormalizedDesktopAction = Required<Pick<DesktopActionDescriptor, "id" | "label">> & {
  command: DesktopActionCommand;
  targetRoute?: DesktopRouteId;
  feedback: string;
  successFeedback?: string;
  errorFeedback?: string;
  tone: DesktopActionTone;
};

type SystemScanResponse = {
  commandId: string;
  requester: string;
};

type SignedUpdateCheckResponse = {
  channel: string;
  updateAvailable: boolean;
  version?: string | null;
};

export async function runDesktopAction(action: DesktopActionDescriptor) {
  const normalized = normalizeDesktopAction(action);

  if (normalized.targetRoute) {
    navigateToDesktopRoute(normalized.targetRoute);
  }

  dispatchDesktopActionFeedback({
    detail: normalized.feedback,
    id: normalized.id,
    label: normalized.label,
    tone: normalized.tone
  });

  try {
    const detail = await runDesktopActionCommand(normalized);

    if (detail || normalized.successFeedback) {
      dispatchDesktopActionFeedback({
        detail: normalized.successFeedback ?? detail ?? normalized.feedback,
        id: `${normalized.id}:complete`,
        label: normalized.label,
        tone: successToneForCommand(normalized.command)
      });
    }
  } catch (error) {
    dispatchDesktopActionFeedback({
      detail: normalized.errorFeedback ?? actionErrorMessage(error),
      id: `${normalized.id}:error`,
      label: normalized.label,
      tone: "danger"
    });
  }
}

export function navigateToDesktopRoute(route: DesktopRouteId) {
  if (typeof window === "undefined") {
    return;
  }

  const nextHash = `#${route}`;

  if (window.location.hash === nextHash) {
    window.dispatchEvent(new HashChangeEvent("hashchange"));
    scrollRouteIntoView(route);
    return;
  }

  window.location.hash = nextHash;
}

export function dispatchDesktopActionFeedback(feedback: DesktopActionFeedback) {
  if (typeof window === "undefined") {
    return;
  }

  window.dispatchEvent(new CustomEvent<DesktopActionFeedback>(desktopActionFeedbackEvent, { detail: feedback }));
}

export function subscribeDesktopActionFeedback(listener: (feedback: DesktopActionFeedback) => void) {
  if (typeof window === "undefined") {
    return () => {};
  }

  const handleFeedback = (event: Event) => {
    listener((event as CustomEvent<DesktopActionFeedback>).detail);
  };

  window.addEventListener(desktopActionFeedbackEvent, handleFeedback);

  return () => window.removeEventListener(desktopActionFeedbackEvent, handleFeedback);
}

function normalizeDesktopAction(action: DesktopActionDescriptor): NormalizedDesktopAction {
  const inferred = inferDesktopAction(action);
  const normalized: NormalizedDesktopAction = {
    command: action.command ?? inferred.command,
    feedback: action.feedback ?? inferred.feedback,
    id: action.id,
    label: action.label,
    tone: inferred.tone
  };

  const targetRoute = action.targetRoute ?? inferred.targetRoute;
  const successFeedback = action.successFeedback ?? inferred.successFeedback;

  if (targetRoute) {
    normalized.targetRoute = targetRoute;
  }

  if (successFeedback) {
    normalized.successFeedback = successFeedback;
  }

  if (action.errorFeedback) {
    normalized.errorFeedback = action.errorFeedback;
  }

  return normalized;
}

function inferDesktopAction(action: DesktopActionDescriptor): Omit<NormalizedDesktopAction, "id" | "label"> {
  const text = `${action.id} ${action.label}`.toLowerCase();

  if (/scan|anal/.test(text)) {
    return {
      command: /cancel/.test(text) ? "status" : "run-read-only-scan",
      feedback: /cancel/.test(text)
        ? "Smart Scan paused. No changes were applied."
        : "Opening Smart Scan. The desktop app checks this PC safely.",
      successFeedback: "Smart Scan is running or completed.",
      targetRoute: "scan",
      tone: "active"
    };
  }

  if (/rollback|restore|recovery|recuper|revers/.test(text)) {
    return {
      command: "rollback",
      feedback: "Opening recovery with restore sessions and audit context.",
      targetRoute: "rollback",
      tone: "rollback"
    };
  }

  if (/benchmark|capture|compare|dx/.test(text)) {
    return {
      command: "benchmark",
      feedback: "Opening benchmark proof and comparison context.",
      targetRoute: "benchmarks",
      tone: "active"
    };
  }

  if (/nvidia|gpu|profile|perfil|driver/.test(text)) {
    return {
      command: "stage-profile",
      feedback: "Opening GPU profile workflow with backup context.",
      targetRoute: "nvidia",
      tone: "active"
    };
  }

  if (/pubg|game|jogo|config|snapshot/.test(text)) {
    return {
      command: /snapshot|config/.test(text) ? "snapshot-config" : "review-plan",
      feedback: "Opening game optimization flow with anti-cheat-safe context.",
      targetRoute: "pubg",
      tone: "success"
    };
  }

  if (/power|balanced|energia/.test(text)) {
    return {
      command: "review-plan",
      feedback: "Opening scoped power-plan recommendations.",
      targetRoute: "power",
      tone: "warning"
    };
  }

  if (/update|atualiz|actualiz/.test(text)) {
    return {
      command: "check-signed-update",
      feedback: "Checking signed update metadata.",
      successFeedback: "Signed update check finished; update trust metadata remains visible.",
      targetRoute: "settings",
      tone: "active"
    };
  }

  if (/notification/.test(text)) {
    return {
      command: "status",
      feedback: "No new optimization notifications. Status and rollback context are current.",
      tone: "neutral"
    };
  }

  if (/data folder|pasta de dados|carpeta de datos/.test(text)) {
    return {
      command: "open-data-folder",
      feedback: "Local data folder action is staged for the desktop shell.",
      targetRoute: "settings",
      tone: "neutral"
    };
  }

  if (/export.*audit|audit|auditoria/.test(text)) {
    return {
      command: "export-audit",
      feedback: "Rollback audit export is staged with local-only data.",
      targetRoute: "rollback",
      tone: "rollback"
    };
  }

  if (/export.*report|relatorio|informe/.test(text)) {
    return {
      command: "export-report",
      feedback: "Benchmark report export is staged with visible metadata.",
      targetRoute: "benchmarks",
      tone: "active"
    };
  }

  if (/export[-\s].*plan|export.*plano|exportar.*plan|exportar.*plano/.test(text)) {
    return {
      command: "export-plan",
      feedback: "Smart Boost plan export is ready with impact and recovery details.",
      targetRoute: "optimize",
      tone: "active"
    };
  }

  if (/export|local data|dados locais|datos locales|history/.test(text)) {
    return {
      command: "export-local-data",
      feedback: "Local export is staged. Cloud sync stays blocked unless consent is enabled.",
      targetRoute: "settings",
      tone: "neutral"
    };
  }

  if (/delete|clear|limpar|apagar/.test(text)) {
    return {
      command: "confirm-required",
      feedback: "Destructive local-data actions require an explicit confirmation step.",
      targetRoute: "settings",
      tone: "warning"
    };
  }

  if (/safe|boost|apply|optimi|smart|review|lab|competitive|blocked|tweak/.test(text)) {
    return {
      command: /apply|boost/.test(text) ? "apply-safe-plan" : "review-plan",
      feedback: "Opening Smart Boost with safe tweaks, review gates, and rollback context.",
      targetRoute: "optimize",
      tone: /apply|boost/.test(text) ? "success" : "active"
    };
  }

  if (/settings|plan|license|help|notification/.test(text)) {
    return {
      command: "status",
      feedback: "Opening settings and local trust controls.",
      targetRoute: "settings",
      tone: "neutral"
    };
  }

  return {
    command: "status",
    feedback: "Action acknowledged.",
    tone: "neutral"
  };
}

async function runDesktopActionCommand(action: NormalizedDesktopAction) {
  if (action.command === "run-read-only-scan") {
    if (!isDesktopTauriRuntime()) {
      return "Smart Scan preview is active. Live PC checks run in the desktop app.";
    }

    const response = await invoke<SystemScanResponse>("run_read_only_system_scan", {
      payload: {
        requester: "main-window"
      }
    });

    return `${response.commandId} completed for ${response.requester}.`;
  }

  if (action.command === "check-signed-update") {
    if (!isDesktopTauriRuntime()) {
      return "Browser preview cannot contact the Tauri updater; signed metadata remains available in Settings.";
    }

    const response = await invoke<SignedUpdateCheckResponse>("check_signed_update", {
      payload: {
        channel: "stable"
      }
    });

    return response.updateAvailable
      ? `Signed ${response.channel} update available: ${response.version ?? "version pending"}.`
      : `No signed ${response.channel} update is currently available.`;
  }

  if (action.command === "confirm-required") {
    return "Confirmation is required before changing local stored data.";
  }

  if (action.command === "apply-safe-plan") {
    return "Safe Boost is selected; changes wait for backup and verification.";
  }

  return undefined;
}

function isDesktopTauriRuntime() {
  try {
    return isTauri();
  } catch {
    return typeof window !== "undefined" && Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__");
  }
}

function successToneForCommand(command: DesktopActionCommand): DesktopActionTone {
  if (command === "rollback" || command === "export-audit") {
    return "rollback";
  }

  if (command === "confirm-required") {
    return "warning";
  }

  return "success";
}

function actionErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return "The action could not complete in this shell.";
}

function scrollRouteIntoView(route: DesktopRouteId) {
  requestAnimationFrame(() => {
    document.getElementById(route)?.scrollIntoView({ block: "start" });
  });
}
