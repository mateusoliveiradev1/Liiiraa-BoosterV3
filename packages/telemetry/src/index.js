export const TELEMETRY_REDACTION_VERSION = "0.1.0";
export const CRASH_REPORT_SCHEMA_VERSION = "0.1.0";

const DEFAULT_MAX_ARRAY_ITEMS = 24;
const DEFAULT_MAX_DEPTH = 6;
const DEFAULT_MAX_STRING_LENGTH = 2_000;
const LOCAL_PATH_PATTERN =
  /(?:[A-Za-z]:[\\/][^\s"'<>|]+|\\\\[^\s\\/"'<>|]+[\\/][^\s"'<>|]+|\/(?:Users|home|mnt|tmp|var)\/[^\s"'<>|]+)/g;
const EMAIL_PATTERN = /\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b/gi;
const IPV4_PATTERN =
  /\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b/g;
const MAC_PATTERN = /\b[0-9A-F]{2}(?::[0-9A-F]{2}){5}\b/gi;
const DATABASE_URL_PATTERN =
  /\b(?:postgres(?:ql)?|mysql|mssql|mongodb(?:\+srv)?|redis):\/\/[^\s"'<>]+/gi;
const SECRET_ASSIGNMENT_PATTERN =
  /\b(authorization|api[_-]?key|cookie|password|secret|session|token)\s*[:=]\s*("[^"]*"|'[^']*'|[^\s,;]+)/gi;
const BEARER_PATTERN = /\bBearer\s+[A-Za-z0-9._~+/=-]{8,}/gi;
const URL_PATTERN = /\bhttps?:\/\/[^\s"'<>]+/gi;
const DENYLISTED_KEY_PATTERN =
  /^(authorization|cookie|databaseUrl|environment|env|minidump|neonUrl|password|rawCrashDump|rawDump|rawPayload|rawRegistryDump|refreshToken|screenshot|secret|sessionToken|token|uploadUrl)$/i;

export function redactTelemetryString(value, options = {}) {
  const maxLength = options.maxStringLength ?? DEFAULT_MAX_STRING_LENGTH;
  let text = String(value ?? "");

  text = text
    .replace(DATABASE_URL_PATTERN, "[redacted-database-url]")
    .replace(BEARER_PATTERN, "Bearer [redacted-secret]")
    .replace(SECRET_ASSIGNMENT_PATTERN, "$1=[redacted-secret]")
    .replace(LOCAL_PATH_PATTERN, "[redacted-local-path]")
    .replace(EMAIL_PATTERN, "[redacted-email]")
    .replace(MAC_PATTERN, "[redacted-mac]")
    .replace(IPV4_PATTERN, "[redacted-ip]")
    .replace(URL_PATTERN, redactUrl);

  return truncate(text, maxLength);
}

export function redactTelemetryValue(value, options = {}) {
  return redactValue(value, options, 0);
}

export function createPrivacySafeCrashReport(input = {}, options = {}) {
  const source = normalizeCrashReportInput(input);
  const error = normalizeErrorLike(source.error ?? source);
  const context = redactTelemetryValue(source.context ?? {});
  const breadcrumbs = Array.isArray(source.breadcrumbs)
    ? source.breadcrumbs.slice(0, 20).map((breadcrumb) => redactTelemetryValue(breadcrumb))
    : [];
  const tags = redactTelemetryValue(source.tags ?? {});

  return {
    kind: "crash-report",
    report: {
      appVersion: safeText(source.appVersion, "0.0.0", 64),
      breadcrumbs,
      capturedAtUtc: normalizeTimestamp(source.capturedAtUtc, options),
      channel: safeText(source.channel, "unknown", 32),
      context,
      error: {
        message: safeText(error.message, "Unexpected error", 512),
        name: safeText(error.name, "Error", 96),
        stack: normalizeStack(error.stack)
      },
      id: safeIdentifier(source.id, "crash:local"),
      platform: safeText(source.platform, "windows", 32),
      severity: normalizeSeverity(source.severity),
      tags
    },
    schemaVersion: CRASH_REPORT_SCHEMA_VERSION
  };
}

export function assertTelemetryRedactionCoverage() {
  const redacted = redactTelemetryValue({
    email: "liiiraa@example.com",
    headers: {
      authorization: "Bearer super-secret-token",
      cookie: "sid=session-secret"
    },
    message:
      "Crash at C:\\Users\\Liiiraa\\AppData\\Local\\Liiiraa\\state.json with token=abc123 and postgres://user:pass@db.example/app",
    nested: {
      path: "/home/liiiraa/.config/liiiraa/settings.json",
      remoteAddress: "192.168.0.25"
    }
  });
  const serialized = JSON.stringify(redacted);

  assertNotContains(serialized, [
    "liiiraa@example.com",
    "C:\\Users\\Liiiraa",
    "/home/liiiraa",
    "abc123",
    "postgres://",
    "192.168.0.25",
    "super-secret-token",
    "session-secret"
  ]);

  const crashReport = createPrivacySafeCrashReport(
    {
      appVersion: "0.1.0",
      capturedAtUtc: "2026-05-02T12:00:00Z",
      channel: "stable",
      context: {
        activeRoute: "/settings?email=liiiraa@example.com",
        localDataPath: "C:\\Users\\Liiiraa\\AppData\\Local\\Liiiraa"
      },
      error: {
        message: "Failed for liiiraa@example.com",
        name: "TypeError",
        stack:
          "TypeError: Failed for liiiraa@example.com\n    at Settings (C:\\Users\\Liiiraa\\src\\settings.tsx:12:3)"
      },
      id: "crash:test:001",
      severity: "fatal"
    },
    {
      capturedAtUtc: "2026-05-02T12:00:00Z"
    }
  );
  const crashJson = JSON.stringify(crashReport);

  assertNotContains(crashJson, ["liiiraa@example.com", "C:\\Users\\Liiiraa"]);
  if (!crashJson.includes("[redacted-email]") || !crashJson.includes("[redacted-local-path]")) {
    throw new Error("Crash reports must preserve useful redaction markers.");
  }

  return true;
}

function redactValue(value, options, depth) {
  const maxDepth = options.maxDepth ?? DEFAULT_MAX_DEPTH;
  const maxArrayItems = options.maxArrayItems ?? DEFAULT_MAX_ARRAY_ITEMS;

  if (value == null || typeof value === "boolean" || typeof value === "number") {
    return value;
  }

  if (typeof value === "string") {
    return redactTelemetryString(value, options);
  }

  if (typeof value === "bigint") {
    return redactTelemetryString(value.toString(), options);
  }

  if (Array.isArray(value)) {
    if (depth >= maxDepth) {
      return "[redacted-nested-data]";
    }

    const items = value.slice(0, maxArrayItems).map((item) => redactValue(item, options, depth + 1));
    if (value.length > maxArrayItems) {
      items.push(`[truncated:${value.length - maxArrayItems}]`);
    }
    return items;
  }

  if (typeof value === "object") {
    if (depth >= maxDepth) {
      return "[redacted-nested-data]";
    }

    const output = {};
    for (const [key, child] of Object.entries(value)) {
      if (DENYLISTED_KEY_PATTERN.test(key)) {
        output[key] = "[redacted]";
        continue;
      }

      const redacted = redactValue(child, options, depth + 1);
      if (redacted !== undefined) {
        output[key] = redacted;
      }
    }
    return output;
  }

  return undefined;
}

function normalizeCrashReportInput(input) {
  if (input instanceof Error) {
    return {
      error: input
    };
  }

  return isPlainObject(input) ? input : {};
}

function normalizeErrorLike(value) {
  if (value instanceof Error) {
    return {
      message: value.message,
      name: value.name,
      stack: value.stack
    };
  }

  if (isPlainObject(value)) {
    return value;
  }

  return {
    message: String(value || "Unexpected error"),
    name: "Error"
  };
}

function normalizeSeverity(value) {
  return ["error", "fatal", "info", "warning"].includes(value) ? value : "error";
}

function normalizeStack(value) {
  if (typeof value !== "string" || value.trim().length === 0) {
    return undefined;
  }

  return redactTelemetryString(
    value
      .split(/\r?\n/)
      .slice(0, 40)
      .join("\n"),
    {
      maxStringLength: 4_000
    }
  );
}

function normalizeTimestamp(value, options) {
  if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(value)) {
    return value;
  }

  if (typeof options.capturedAtUtc === "string") {
    return options.capturedAtUtc;
  }

  if (typeof options.now === "function") {
    return new Date(options.now()).toISOString();
  }

  return new Date().toISOString();
}

function safeIdentifier(value, fallback) {
  if (typeof value !== "string") {
    return fallback;
  }

  const trimmed = value.trim();
  return /^[A-Za-z0-9][A-Za-z0-9._:-]{2,127}$/.test(trimmed) ? trimmed : fallback;
}

function safeText(value, fallback, maxLength) {
  if (typeof value !== "string") {
    return fallback;
  }

  const redacted = redactTelemetryString(value, {
    maxStringLength: maxLength
  }).trim();

  return redacted.length > 0 ? redacted : fallback;
}

function redactUrl(value) {
  try {
    const url = new URL(value);
    if (url.username || url.password || url.search) {
      url.username = "";
      url.password = "";
      url.search = "?[redacted-query]";
    }
    return url.toString();
  } catch {
    return "[redacted-url]";
  }
}

function truncate(value, maxLength) {
  if (!Number.isInteger(maxLength) || maxLength < 1 || value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, maxLength)}...[truncated]`;
}

function assertNotContains(value, needles) {
  for (const needle of needles) {
    if (value.includes(needle)) {
      throw new Error(`Telemetry redaction leaked ${needle}`);
    }
  }
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}
