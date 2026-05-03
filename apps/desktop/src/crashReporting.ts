import {
  createPrivacySafeCrashReport,
  type PrivacySafeCrashReport
} from "../../../packages/telemetry/src/index.js";
import {
  createDefaultPrivacyConsentState,
  evaluateDesktopPrivacyGate,
  type PrivacyConsentState,
  type PrivacyGateResult,
  type PrivacySignalDestination
} from "./privacyConsent";

export const DESKTOP_CRASH_REPORTING_VERSION = "0.1.0";

export type DesktopCrashReportDecision = {
  accepted: boolean;
  gate: PrivacyGateResult;
  localReport?: PrivacySafeCrashReport;
  uploadPayload?: PrivacySafeCrashReport;
  version: string;
};

export type DesktopCrashReportInput = {
  appVersion?: string;
  breadcrumbs?: unknown[];
  capturedAtUtc?: string;
  channel?: string;
  context?: Record<string, unknown>;
  error?: Error | { message?: string; name?: string; stack?: string };
  id?: string;
  platform?: string;
  severity?: "error" | "fatal" | "info" | "warning";
  tags?: Record<string, unknown>;
};

export function createDesktopCrashReport(input: DesktopCrashReportInput = {}): PrivacySafeCrashReport {
  return createPrivacySafeCrashReport({
    ...input,
    platform: input.platform ?? "windows"
  });
}

export function createDesktopCrashReportDecision({
  consent = createDefaultPrivacyConsentState(),
  destination = "cloud",
  report
}: {
  consent?: PrivacyConsentState;
  destination?: PrivacySignalDestination;
  report: DesktopCrashReportInput;
}): DesktopCrashReportDecision {
  const gate = evaluateDesktopPrivacyGate({
    consent,
    destination,
    kind: "crash-report"
  });
  const redactedReport = createDesktopCrashReport(report);
  const accepted = gate.allowed && destination === "cloud";

  const decision: DesktopCrashReportDecision = {
    accepted,
    gate,
    version: DESKTOP_CRASH_REPORTING_VERSION
  };

  if (accepted) {
    decision.uploadPayload = redactedReport;
  } else {
    decision.localReport = redactedReport;
  }

  return decision;
}

export function captureDesktopErrorReport(
  error: unknown,
  context: DesktopCrashReportInput["context"] = {}
): DesktopCrashReportInput {
  if (error instanceof Error) {
    return {
      context,
      error
    };
  }

  return {
    context,
    error: {
      message: String(error || "Unexpected desktop error"),
      name: "Error"
    }
  };
}

export function assertDesktopCrashReportingCoverage() {
  const defaultDecision = createDesktopCrashReportDecision({
    report: sampleDesktopCrashReport()
  });
  if (defaultDecision.accepted || defaultDecision.gate.action !== "store-local-crash") {
    throw new Error("Desktop crash reports must stay local before explicit consent.");
  }

  const consentedDecision = createDesktopCrashReportDecision({
    consent: createDefaultPrivacyConsentState({ crashReports: true }),
    report: sampleDesktopCrashReport()
  });
  if (!consentedDecision.accepted || !consentedDecision.uploadPayload) {
    throw new Error("Desktop crash reports must upload only after explicit consent.");
  }

  const serialized = JSON.stringify(consentedDecision.uploadPayload);
  for (const raw of ["liiiraa@example.com", "C:\\Users\\Liiiraa", "token=abc123"]) {
    if (serialized.includes(raw)) {
      throw new Error(`Desktop crash report leaked ${raw}`);
    }
  }

  return true;
}

function sampleDesktopCrashReport(): DesktopCrashReportInput {
  return {
    appVersion: "0.1.0",
    capturedAtUtc: "2026-05-02T12:00:00Z",
    channel: "stable",
    context: {
      activeRoute: "/settings?email=liiiraa@example.com",
      localDataPath: "C:\\Users\\Liiiraa\\AppData\\Local\\Liiiraa"
    },
    error: {
      message: "Renderer failed for liiiraa@example.com with token=abc123",
      name: "RendererCrash",
      stack:
        "RendererCrash: token=abc123\n    at Settings (C:\\Users\\Liiiraa\\AppData\\Local\\app.tsx:8:2)"
    },
    id: "crash:desktop:001",
    severity: "fatal"
  };
}
