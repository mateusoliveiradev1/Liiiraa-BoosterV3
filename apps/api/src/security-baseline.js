import { createHash, randomUUID } from "node:crypto";
import {
  ContractValidationError,
  publicProcedureSecurityPolicies,
  validateSecurityEnvelope
} from "../../../packages/api-contract/src/index.js";
import { loadApiSecurityConfig } from "./config.js";
import { evaluateApiPrivacyConsentGate, requireApiPrivacyConsent } from "./privacy-consent.js";

const SENSITIVE_HEADER_PATTERN =
  /^(authorization|cookie|set-cookie|x-api-key|x-auth-token|x-csrf-token|x-session|proxy-authorization)$/i;
const SAFE_METHOD_PATTERN = /^(GET|POST|PUT|PATCH|DELETE|OPTIONS|HEAD)$/;
const SAFE_LOG_FIELD_LIMIT = 160;

export function createApiSecurityBaseline(options = {}) {
  const config = options.config ?? loadApiSecurityConfig(options.env);
  const cors = createCorsPolicy(config.cors);
  const rateLimiter = createRateLimiter(config.rateLimit);

  return {
    config,
    cors,
    rateLimiter,
    createContext(request) {
      return createRequestContext({
        ...request,
        requestIdHeader: config.requestIdHeader
      });
    },
    logEvent(request, outcome) {
      return createLeastPrivilegeLogEvent(request, outcome);
    },
    redactError(error, context) {
      return redactApiError(error, {
        enabled: config.errorRedactionEnabled,
        requestId: context?.requestId
      });
    },
    validateProcedure(request) {
      return validateApiProcedureRequest(request);
    },
    evaluateConsent(request, consentOptions) {
      return evaluateApiPrivacyConsentGate(request, consentOptions);
    },
    requireConsent(request, consentOptions) {
      return requireApiPrivacyConsent(request, consentOptions);
    }
  };
}

export function createCorsPolicy(options = {}) {
  const allowedOrigins = new Set((options.allowedOrigins ?? []).map(normalizeOrigin));

  return {
    allowedOrigins: [...allowedOrigins],
    evaluate(origin) {
      if (origin == null || origin === "") {
        return {
          allowed: true,
          headers: {
            vary: "Origin"
          }
        };
      }

      const normalizedOrigin = normalizeOrigin(origin);
      const allowed = allowedOrigins.has(normalizedOrigin);

      return {
        allowed,
        headers: allowed
          ? {
              "access-control-allow-credentials": "false",
              "access-control-allow-origin": normalizedOrigin,
              "access-control-expose-headers": "x-request-id",
              vary: "Origin"
            }
          : {
              vary: "Origin"
            },
        statusCode: allowed ? 204 : 403
      };
    }
  };
}

export function createRateLimiter(options = {}) {
  const windowMs = positiveInteger(options.windowMs, "windowMs");
  const max = positiveInteger(options.max, "max");
  const now = typeof options.now === "function" ? options.now : () => Date.now();
  const buckets = new Map();

  return {
    check(key) {
      const normalizedKey = normalizeRateLimitKey(key);
      const currentTime = now();
      const currentBucket = buckets.get(normalizedKey);
      const resetAt =
        currentBucket && currentTime < currentBucket.resetAt
          ? currentBucket.resetAt
          : currentTime + windowMs;

      const bucket =
        currentBucket && currentTime < currentBucket.resetAt
          ? currentBucket
          : { count: 0, resetAt };

      bucket.count += 1;
      buckets.set(normalizedKey, bucket);

      const remaining = Math.max(max - bucket.count, 0);
      return {
        allowed: bucket.count <= max,
        key: normalizedKey,
        limit: max,
        remaining,
        resetAt: bucket.resetAt,
        windowMs
      };
    },
    reset() {
      buckets.clear();
    }
  };
}

export function createRequestContext(options = {}) {
  const headers = normalizeHeaders(options.headers);
  const requestIdHeader = String(options.requestIdHeader || "x-request-id").toLowerCase();
  const requestId =
    sanitizeRequestId(headers[requestIdHeader]) ||
    sanitizeRequestId(options.requestId) ||
    options.idGenerator?.() ||
    randomUUID();
  const method = normalizeMethod(options.method);
  const url = normalizeUrl(options.url);

  return {
    headers: stripSensitiveHeaders(headers),
    method,
    origin: headers.origin,
    path: url.pathname,
    procedure: options.procedure,
    rateLimitKey: hashValue(options.remoteAddress || headers["x-forwarded-for"] || "anonymous"),
    requestId,
    startedAt: options.now?.() ?? Date.now(),
    userAgent: truncateLogValue(headers["user-agent"])
  };
}

export function validateApiProcedureRequest(request) {
  return validateSecurityEnvelope(request);
}

export function redactApiError(error, options = {}) {
  const requestId = sanitizeRequestId(options.requestId);

  if (options.enabled === false && error instanceof ContractValidationError) {
    return {
      error: {
        code: error.code,
        issues: error.issues,
        message: error.message,
        requestId
      }
    };
  }

  if (error instanceof ContractValidationError) {
    return {
      error: {
        code: error.code,
        issues: error.issues.map(redactIssue),
        message: "Request validation failed.",
        requestId
      }
    };
  }

  if (error?.code === "CONSENT_REQUIRED") {
    return {
      error: {
        code: "CONSENT_REQUIRED",
        message: "Explicit consent is required before cloud upload.",
        requestId
      }
    };
  }

  if (error?.code === "INVALID_CONSENT_SCOPE") {
    return {
      error: {
        code: "BAD_REQUEST",
        message: "Request validation failed.",
        requestId
      }
    };
  }

  const code = error?.code === "TOO_MANY_REQUESTS" ? "TOO_MANY_REQUESTS" : "INTERNAL_SERVER_ERROR";
  return {
    error: {
      code,
      message: code === "TOO_MANY_REQUESTS" ? "Rate limit exceeded." : "Unexpected server error.",
      requestId
    }
  };
}

export function createLeastPrivilegeLogEvent(requestContext, outcome = {}) {
  const statusCode = Number.isInteger(outcome.statusCode) ? outcome.statusCode : 500;
  const policy = requestContext?.procedure
    ? publicProcedureSecurityPolicies[requestContext.procedure]
    : undefined;
  const allowedFields = new Set(policy?.logging.fields ?? ["requestId", "method", "path", "statusCode"]);

  const fullEvent = {
    durationMs: Number.isFinite(outcome.durationMs) ? Math.max(0, outcome.durationMs) : undefined,
    method: requestContext?.method,
    origin: truncateLogValue(requestContext?.origin),
    path: truncateLogValue(requestContext?.path),
    procedure: requestContext?.procedure,
    rateLimited: Boolean(outcome.rateLimited),
    remoteAddressHash: requestContext?.rateLimitKey,
    requestId: requestContext?.requestId,
    statusCode,
    timestamp: new Date(outcome.now ?? Date.now()).toISOString(),
    userAgent: truncateLogValue(requestContext?.userAgent)
  };

  const logEvent = {
    timestamp: fullEvent.timestamp
  };
  for (const field of allowedFields) {
    if (fullEvent[field] !== undefined) {
      logEvent[field] = fullEvent[field];
    }
  }

  return logEvent;
}

function normalizeOrigin(origin) {
  const parsed = new URL(String(origin));
  return parsed.origin;
}

function normalizeRateLimitKey(key) {
  return hashValue(String(key || "anonymous"));
}

function hashValue(value) {
  return createHash("sha256").update(value).digest("hex").slice(0, 24);
}

function normalizeHeaders(headers = {}) {
  const normalized = {};

  if (headers instanceof Headers) {
    for (const [name, value] of headers.entries()) {
      normalized[name.toLowerCase()] = value;
    }
    return normalized;
  }

  for (const [name, value] of Object.entries(headers)) {
    if (value == null) {
      continue;
    }

    normalized[name.toLowerCase()] = Array.isArray(value) ? value.join(",") : String(value);
  }

  return normalized;
}

function stripSensitiveHeaders(headers) {
  const safeHeaders = {};
  for (const [name, value] of Object.entries(headers)) {
    if (!SENSITIVE_HEADER_PATTERN.test(name)) {
      safeHeaders[name] = truncateLogValue(value);
    }
  }
  return safeHeaders;
}

function normalizeMethod(method) {
  const normalized = String(method || "GET").toUpperCase();
  return SAFE_METHOD_PATTERN.test(normalized) ? normalized : "GET";
}

function normalizeUrl(value) {
  try {
    return new URL(value || "/", "http://api.local");
  } catch {
    return new URL("/", "http://api.local");
  }
}

function sanitizeRequestId(value) {
  if (typeof value !== "string") {
    return undefined;
  }

  const trimmed = value.trim();
  return /^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$/.test(trimmed) ? trimmed : undefined;
}

function positiveInteger(value, name) {
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

function redactIssue(issue) {
  return {
    code: issue.code,
    path: issue.path
  };
}

function truncateLogValue(value) {
  if (value == null) {
    return undefined;
  }

  const text = String(value);
  return text.length > SAFE_LOG_FIELD_LIMIT ? `${text.slice(0, SAFE_LOG_FIELD_LIMIT)}...` : text;
}
