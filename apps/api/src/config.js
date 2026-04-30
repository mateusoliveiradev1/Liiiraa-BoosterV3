const DEFAULT_ALLOWED_ORIGINS = Object.freeze([
  "http://localhost:1420",
  "http://localhost:5173"
]);

const DEFAULT_RATE_LIMIT_WINDOW_MS = 60_000;
const DEFAULT_RATE_LIMIT_MAX = 120;
const DEFAULT_REQUEST_ID_HEADER = "x-request-id";

const BOOLEAN_TRUE = new Set(["1", "true", "yes", "on"]);
const BOOLEAN_FALSE = new Set(["0", "false", "no", "off"]);

export function parseAllowedOrigins(value) {
  if (value == null || value === "") {
    return [...DEFAULT_ALLOWED_ORIGINS];
  }

  const origins = String(value)
    .split(",")
    .map((origin) => origin.trim())
    .filter(Boolean);

  if (origins.length === 0) {
    return [...DEFAULT_ALLOWED_ORIGINS];
  }

  const unique = new Set();
  for (const origin of origins) {
    assertAllowedOrigin(origin);
    unique.add(origin);
  }

  return [...unique];
}

export function parsePositiveInteger(value, fallback, name) {
  if (value == null || value === "") {
    return fallback;
  }

  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }

  return parsed;
}

export function parseBoolean(value, fallback) {
  if (value == null || value === "") {
    return fallback;
  }

  const normalized = String(value).trim().toLowerCase();
  if (BOOLEAN_TRUE.has(normalized)) {
    return true;
  }

  if (BOOLEAN_FALSE.has(normalized)) {
    return false;
  }

  throw new Error(`Expected boolean-like value, received ${value}`);
}

export function normalizeRequestIdHeader(value) {
  const header = String(value || DEFAULT_REQUEST_ID_HEADER)
    .trim()
    .toLowerCase();

  if (!/^[a-z0-9][a-z0-9-]{0,62}$/.test(header)) {
    throw new Error("REQUEST_ID_HEADER must be an HTTP token header name");
  }

  return header;
}

export function loadApiSecurityConfig(env = process.env) {
  return {
    cors: {
      allowedOrigins: parseAllowedOrigins(env.API_CORS_ALLOWED_ORIGINS)
    },
    errorRedactionEnabled: parseBoolean(env.ERROR_REDACTION_ENABLED, true),
    logging: {
      level: env.LOG_LEVEL || "info"
    },
    rateLimit: {
      max: parsePositiveInteger(env.RATE_LIMIT_MAX, DEFAULT_RATE_LIMIT_MAX, "RATE_LIMIT_MAX"),
      windowMs: parsePositiveInteger(
        env.RATE_LIMIT_WINDOW_MS,
        DEFAULT_RATE_LIMIT_WINDOW_MS,
        "RATE_LIMIT_WINDOW_MS"
      )
    },
    requestIdHeader: normalizeRequestIdHeader(env.REQUEST_ID_HEADER)
  };
}

function assertAllowedOrigin(origin) {
  let parsed;
  try {
    parsed = new URL(origin);
  } catch {
    throw new Error(`Invalid CORS origin: ${origin}`);
  }

  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error(`Unsupported CORS origin protocol: ${parsed.protocol}`);
  }

  if (parsed.username || parsed.password || parsed.pathname !== "/" || parsed.search || parsed.hash) {
    throw new Error(`CORS origin must include only scheme, host, and optional port: ${origin}`);
  }
}
