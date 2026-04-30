import type { SettingsTrustData, SettingsTrustTone } from "../../../packages/ui/src/settingsTrust.js";

export type PrivacyConsentKey = "benchmarkSync" | "crashReports" | "telemetry";
export type PrivacySignalKind = "benchmark-sync" | "crash-report" | "telemetry";
export type PrivacySignalDestination = "cloud" | "local" | "manual-export";

export type PrivacyConsentState = Record<PrivacyConsentKey, boolean>;

export type PrivacyGateResult = {
  action: "accept-for-sync" | "drop-upload" | "keep-local" | "store-local-crash";
  allowed: boolean;
  destination: PrivacySignalDestination;
  kind: PrivacySignalKind;
  message: string;
  requiredConsent: PrivacyConsentKey;
  tone: SettingsTrustTone;
  value: string;
};

export type PrivacyConsentGateSummary = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: SettingsTrustTone;
};

const privacyConsentRequirements: Record<
  PrivacySignalKind,
  {
    blockedAction: PrivacyGateResult["action"];
    consentKey: PrivacyConsentKey;
    label: string;
  }
> = {
  "benchmark-sync": {
    blockedAction: "keep-local",
    consentKey: "benchmarkSync",
    label: "Benchmark cloud sync"
  },
  "crash-report": {
    blockedAction: "store-local-crash",
    consentKey: "crashReports",
    label: "Crash reports"
  },
  telemetry: {
    blockedAction: "drop-upload",
    consentKey: "telemetry",
    label: "Performance telemetry"
  }
};

const consentControlCopy: Record<
  PrivacyConsentKey,
  {
    detailOff: string;
    detailOn: string;
    valueOff: string;
    valueOn: string;
  }
> = {
  benchmarkSync: {
    detailOff: "Before and after captures remain local unless benchmark sync consent is enabled.",
    detailOn: "Benchmark sessions can sync with metadata after explicit consent.",
    valueOff: "Local only",
    valueOn: "Cloud sync on"
  },
  crashReports: {
    detailOff: "Crash reports stay local and can be exported manually after redaction.",
    detailOn: "Redacted crash reports can be sent for diagnostics.",
    valueOff: "Off",
    valueOn: "On"
  },
  telemetry: {
    detailOff: "Performance telemetry, scan summaries, and diagnostics stay on this PC.",
    detailOn: "Performance telemetry can be uploaded without secrets or personal files.",
    valueOff: "Off",
    valueOn: "On"
  }
};

export function createDefaultPrivacyConsentState(
  input: Partial<PrivacyConsentState> = {}
): PrivacyConsentState {
  return {
    benchmarkSync: input.benchmarkSync === true,
    crashReports: input.crashReports === true,
    telemetry: input.telemetry === true
  };
}

export function evaluateDesktopPrivacyGate({
  consent = createDefaultPrivacyConsentState(),
  destination = "cloud",
  kind
}: {
  consent?: PrivacyConsentState;
  destination?: PrivacySignalDestination;
  kind: PrivacySignalKind;
}): PrivacyGateResult {
  const requirement = privacyConsentRequirements[kind];

  if (destination === "local" || destination === "manual-export") {
    return {
      action: "keep-local",
      allowed: true,
      destination,
      kind,
      message: "Local history and manual export remain available without cloud consent.",
      requiredConsent: requirement.consentKey,
      tone: "success",
      value: "Local allowed"
    };
  }

  const allowed = consent[requirement.consentKey] === true;

  return {
    action: allowed ? "accept-for-sync" : requirement.blockedAction,
    allowed,
    destination,
    kind,
    message: allowed
      ? `${requirement.label} consent is enabled.`
      : `${requirement.label} consent is required before cloud upload.`,
    requiredConsent: requirement.consentKey,
    tone: allowed ? "active" : "success",
    value: allowed ? "Allowed" : "Blocked"
  };
}

export function buildPrivacyConsentGateSummary(
  consent = createDefaultPrivacyConsentState()
): PrivacyConsentGateSummary[] {
  return (Object.keys(privacyConsentRequirements) as PrivacySignalKind[]).map((kind) => {
    const result = evaluateDesktopPrivacyGate({ consent, kind });

    return {
      id: `${kind}-gate`,
      label: privacyConsentRequirements[kind].label,
      value: result.value,
      detail: result.message,
      tone: result.tone
    };
  });
}

export function applyPrivacyConsentToSettings(
  data: SettingsTrustData,
  consent = createDefaultPrivacyConsentState()
): SettingsTrustData {
  const controlById: Record<string, PrivacyConsentKey> = {
    "benchmark-sync": "benchmarkSync",
    "crash-reports": "crashReports",
    telemetry: "telemetry"
  };

  return {
    ...data,
    privacyControls: data.privacyControls.map((control) => {
      const consentKey = controlById[control.id];
      if (!consentKey) {
        return control;
      }

      const copy = consentControlCopy[consentKey];
      const enabled = consent[consentKey];

      return {
        ...control,
        detail: enabled ? copy.detailOn : copy.detailOff,
        enabled,
        tone: enabled ? "active" : control.tone,
        value: enabled ? copy.valueOn : copy.valueOff
      };
    })
  };
}

export function assertDesktopPrivacyConsentCoverage() {
  const defaultConsent = createDefaultPrivacyConsentState();

  for (const kind of Object.keys(privacyConsentRequirements) as PrivacySignalKind[]) {
    const denied = evaluateDesktopPrivacyGate({ consent: defaultConsent, kind });
    if (denied.allowed || denied.value !== "Blocked") {
      throw new Error(`${kind} must be blocked before explicit consent.`);
    }

    const allowed = evaluateDesktopPrivacyGate({
      consent: createDefaultPrivacyConsentState({ [denied.requiredConsent]: true }),
      kind
    });
    if (!allowed.allowed || allowed.value !== "Allowed") {
      throw new Error(`${kind} must be allowed after explicit consent.`);
    }
  }

  const localBenchmark = evaluateDesktopPrivacyGate({
    consent: defaultConsent,
    destination: "local",
    kind: "benchmark-sync"
  });
  if (!localBenchmark.allowed || localBenchmark.action !== "keep-local") {
    throw new Error("Local benchmark history must not require cloud consent.");
  }

  return true;
}
