export const API_PRIVACY_CONSENT_VERSION = "0.1.0";

export const privacyConsentRequirements = Object.freeze({
  "benchmark-sync": {
    consentKey: "benchmarkSync",
    label: "Benchmark cloud sync",
    blockedAction: "keep-local"
  },
  "crash-report": {
    consentKey: "crashReports",
    label: "Crash reports",
    blockedAction: "store-local-crash"
  },
  telemetry: {
    consentKey: "telemetry",
    label: "Performance telemetry",
    blockedAction: "drop-upload"
  }
});

const LOCAL_DESTINATIONS = new Set(["local", "manual-export"]);

export class PrivacyConsentRequiredError extends Error {
  constructor(result) {
    super(result.message);
    this.name = "PrivacyConsentRequiredError";
    this.code = result.code;
    this.statusCode = result.statusCode;
    this.requiredConsent = result.requiredConsent;
  }
}

export function createApiPrivacyConsentState(input = {}) {
  return {
    benchmarkSync: input.benchmarkSync === true,
    crashReports: input.crashReports === true,
    telemetry: input.telemetry === true
  };
}

export function evaluateApiPrivacyConsentGate(request = {}, options = {}) {
  const kind = normalizeConsentKind(request.kind ?? request.type ?? request.signal);
  const destination = normalizeDestination(request.destination ?? options.destination ?? "cloud");

  if (!kind) {
    return {
      allowed: false,
      code: "INVALID_CONSENT_SCOPE",
      destination,
      message: "Telemetry upload request did not match a known privacy consent scope.",
      statusCode: 400
    };
  }

  const requirement = privacyConsentRequirements[kind];
  if (LOCAL_DESTINATIONS.has(destination)) {
    return {
      action: "keep-local",
      allowed: true,
      code: "CONSENT_NOT_REQUIRED",
      destination,
      kind,
      message: "Local capture and manual export remain available without cloud consent.",
      requiredConsent: requirement.consentKey,
      statusCode: 200
    };
  }

  const consent = createApiPrivacyConsentState(request.consent ?? options.consent);
  const allowed = consent[requirement.consentKey] === true;

  return {
    action: allowed ? "accept-for-sync" : requirement.blockedAction,
    allowed,
    code: allowed ? "CONSENT_GRANTED" : "CONSENT_REQUIRED",
    destination,
    kind,
    message: allowed
      ? `${requirement.label} consent is enabled.`
      : `${requirement.label} consent is required before cloud upload.`,
    requiredConsent: requirement.consentKey,
    statusCode: allowed ? 202 : 403
  };
}

export function requireApiPrivacyConsent(request = {}, options = {}) {
  const result = evaluateApiPrivacyConsentGate(request, options);

  if (!result.allowed) {
    throw new PrivacyConsentRequiredError(result);
  }

  return result;
}

export function createPrivacySafeSyncDecision(request = {}, options = {}) {
  const result = evaluateApiPrivacyConsentGate(request, options);

  return {
    accepted: result.allowed && result.destination === "cloud",
    gate: result,
    requestId: sanitizeRequestId(request.requestId),
    version: API_PRIVACY_CONSENT_VERSION
  };
}

export function assertApiPrivacyConsentCoverage() {
  for (const kind of Object.keys(privacyConsentRequirements)) {
    const denied = evaluateApiPrivacyConsentGate({ consent: {}, kind });
    if (denied.allowed || denied.statusCode !== 403) {
      throw new Error(`${kind} must be denied without explicit consent.`);
    }

    const allowed = evaluateApiPrivacyConsentGate({
      consent: { [denied.requiredConsent]: true },
      kind
    });
    if (!allowed.allowed || allowed.statusCode !== 202) {
      throw new Error(`${kind} must be allowed with explicit consent.`);
    }
  }

  const localBenchmark = evaluateApiPrivacyConsentGate({
    destination: "local",
    kind: "benchmark-sync"
  });
  if (!localBenchmark.allowed || localBenchmark.action !== "keep-local") {
    throw new Error("Local benchmark history must remain available without cloud consent.");
  }

  const invalid = evaluateApiPrivacyConsentGate({ kind: "unknown" });
  if (invalid.allowed || invalid.statusCode !== 400) {
    throw new Error("Unknown telemetry scopes must be rejected.");
  }

  return true;
}

function normalizeConsentKind(value) {
  const normalized = String(value || "")
    .trim()
    .toLowerCase();

  return Object.hasOwn(privacyConsentRequirements, normalized) ? normalized : undefined;
}

function normalizeDestination(value) {
  const normalized = String(value || "cloud")
    .trim()
    .toLowerCase();

  return LOCAL_DESTINATIONS.has(normalized) ? normalized : "cloud";
}

function sanitizeRequestId(value) {
  if (typeof value !== "string") {
    return undefined;
  }

  const trimmed = value.trim();
  return /^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$/.test(trimmed) ? trimmed : undefined;
}
