import { ContractValidationError } from "../../../packages/api-contract/src/index.js";
import { createPrivacySafeCrashReport } from "../../../packages/telemetry/src/index.js";
import { PrivacyConsentRequiredError, evaluateApiPrivacyConsentGate } from "./privacy-consent.js";

export const API_CRASH_REPORTING_VERSION = "0.1.0";
export const CRASH_REPORTING_PROCEDURE = "crashReports.ingest";

export function validateCrashReportEnvelope(envelope = {}) {
  const issues = [];

  if (!isPlainObject(envelope)) {
    throw new ContractValidationError("Crash report envelope must be an object", [
      issue([], "invalid_type", "object")
    ]);
  }

  const requestId = validateRequestId(envelope.requestId, ["requestId"], issues);
  const procedure = envelope.procedure ?? CRASH_REPORTING_PROCEDURE;
  if (procedure !== CRASH_REPORTING_PROCEDURE) {
    issues.push(issue(["procedure"], "invalid_procedure", CRASH_REPORTING_PROCEDURE));
  }

  const payload = isPlainObject(envelope.payload) ? envelope.payload : undefined;
  if (!payload) {
    issues.push(issue(["payload"], "invalid_type", "object"));
  }

  const report = isPlainObject(payload?.report) ? payload.report : undefined;
  if (!report) {
    issues.push(issue(["payload", "report"], "invalid_type", "object"));
  }

  const consent = isPlainObject(payload?.consent) ? payload.consent : {};

  if (issues.length > 0) {
    throw new ContractValidationError("Crash report failed API validation", issues);
  }

  return {
    payload: {
      consent,
      report
    },
    procedure,
    requestId
  };
}

export function createCrashReportIngestDecision(envelope = {}, options = {}) {
  const request = validateCrashReportEnvelope(envelope);
  const gate = evaluateApiPrivacyConsentGate({
    consent: request.payload.consent,
    destination: options.destination ?? "cloud",
    kind: "crash-report",
    requestId: request.requestId
  });
  const accepted = gate.allowed && gate.destination === "cloud";
  const redactedReport = createPrivacySafeCrashReport(request.payload.report, {
    capturedAtUtc: options.capturedAtUtc,
    now: options.now
  });

  return {
    accepted,
    gate,
    localReport: accepted ? undefined : redactedReport,
    payload: accepted ? redactedReport : undefined,
    procedure: CRASH_REPORTING_PROCEDURE,
    requestId: request.requestId,
    statusCode: gate.statusCode,
    version: API_CRASH_REPORTING_VERSION
  };
}

export function requireCrashReportCloudUpload(envelope = {}, options = {}) {
  const decision = createCrashReportIngestDecision(envelope, options);

  if (!decision.accepted) {
    throw new PrivacyConsentRequiredError(decision.gate);
  }

  return decision;
}

export function assertApiCrashReportingCoverage() {
  const accepted = createCrashReportIngestDecision(crashEnvelope(), {
    capturedAtUtc: "2026-05-02T12:00:00Z"
  });

  if (!accepted.accepted || accepted.statusCode !== 202 || !accepted.payload) {
    throw new Error("Consented crash reports must be accepted for cloud upload.");
  }

  assertNoRawPii(JSON.stringify(accepted.payload));

  const denied = createCrashReportIngestDecision(
    crashEnvelope({
      consent: {}
    }),
    {
      capturedAtUtc: "2026-05-02T12:00:00Z"
    }
  );
  if (denied.accepted || denied.statusCode !== 403 || denied.payload !== undefined) {
    throw new Error("Crash reports must not upload without explicit consent.");
  }
  if (!denied.localReport || denied.gate.action !== "store-local-crash") {
    throw new Error("Denied crash reports must be redacted for local storage only.");
  }

  try {
    validateCrashReportEnvelope({
      payload: {
        consent: {
          crashReports: true
        },
        report: {}
      },
      requestId: "bad id"
    });
  } catch (error) {
    if (error instanceof ContractValidationError) {
      return true;
    }
    throw error;
  }

  throw new Error("Crash report validation must reject unsafe request IDs.");
}

function crashEnvelope(overrides = {}) {
  return {
    payload: {
      consent: overrides.consent ?? {
        crashReports: true
      },
      report: overrides.report ?? {
        appVersion: "0.1.0",
        capturedAtUtc: "2026-05-02T12:00:00Z",
        channel: "stable",
        context: {
          activeRoute: "/settings?email=liiiraa@example.com",
          localDataPath: "C:\\Users\\Liiiraa\\AppData\\Local\\Liiiraa",
          remoteAddress: "192.168.0.25"
        },
        error: {
          message: "Renderer crashed for liiiraa@example.com with token=abc123",
          name: "RendererCrash",
          stack:
            "RendererCrash: token=abc123\n    at Settings (C:\\Users\\Liiiraa\\AppData\\Local\\app.tsx:8:2)"
        },
        id: "crash:test:001",
        severity: "fatal"
      }
    },
    procedure: CRASH_REPORTING_PROCEDURE,
    requestId: "req_crash001"
  };
}

function assertNoRawPii(value) {
  for (const raw of [
    "liiiraa@example.com",
    "C:\\Users\\Liiiraa",
    "192.168.0.25",
    "token=abc123"
  ]) {
    if (value.includes(raw)) {
      throw new Error(`Crash report leaked ${raw}`);
    }
  }
}

function validateRequestId(value, path, issues) {
  if (typeof value !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$/.test(value)) {
    issues.push(issue(path, "invalid_request_id", "8-128 safe request id characters"));
    return undefined;
  }

  return value;
}

function issue(path, code, expected) {
  return {
    code,
    expected,
    path
  };
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}
