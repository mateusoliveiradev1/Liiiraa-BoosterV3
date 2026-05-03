export type RedactionOptions = {
  maxArrayItems?: number;
  maxDepth?: number;
  maxStringLength?: number;
};

export type PrivacySafeCrashReport = {
  kind: "crash-report";
  report: {
    appVersion: string;
    breadcrumbs: unknown[];
    capturedAtUtc: string;
    channel: string;
    context: unknown;
    error: {
      message: string;
      name: string;
      stack?: string;
    };
    id: string;
    platform: string;
    severity: "error" | "fatal" | "info" | "warning";
    tags: unknown;
  };
  schemaVersion: string;
};

export const CRASH_REPORT_SCHEMA_VERSION: string;
export const TELEMETRY_REDACTION_VERSION: string;

export function assertTelemetryRedactionCoverage(): true;
export function createPrivacySafeCrashReport(
  input?: unknown,
  options?: { capturedAtUtc?: string; now?: () => number }
): PrivacySafeCrashReport;
export function redactTelemetryString(value: unknown, options?: RedactionOptions): string;
export function redactTelemetryValue(value: unknown, options?: RedactionOptions): unknown;
