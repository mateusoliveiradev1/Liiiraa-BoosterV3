import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  createApiSecurityBaseline,
  createCorsPolicy,
  createLeastPrivilegeLogEvent,
  createRateLimiter,
  createRequestContext,
  loadApiSecurityConfig,
  redactApiError,
  validateApiProcedureRequest
} from "../src/index.js";
import { ContractValidationError } from "../../../packages/api-contract/src/index.js";

describe("API security baseline", () => {
  it("loads a constrained CORS and rate-limit configuration from env", () => {
    const config = loadApiSecurityConfig({
      API_CORS_ALLOWED_ORIGINS: "http://localhost:1420,https://app.liiiraa.example",
      ERROR_REDACTION_ENABLED: "true",
      LOG_LEVEL: "debug",
      RATE_LIMIT_MAX: "2",
      RATE_LIMIT_WINDOW_MS: "1000",
      REQUEST_ID_HEADER: "X-Request-Id"
    });

    assert.deepEqual(config.cors.allowedOrigins, [
      "http://localhost:1420",
      "https://app.liiiraa.example"
    ]);
    assert.equal(config.rateLimit.max, 2);
    assert.equal(config.rateLimit.windowMs, 1000);
    assert.equal(config.requestIdHeader, "x-request-id");
    assert.equal(config.errorRedactionEnabled, true);
  });

  it("rejects CORS origins outside the allowlist", () => {
    const cors = createCorsPolicy({
      allowedOrigins: ["http://localhost:1420"]
    });

    assert.equal(cors.evaluate("http://localhost:1420").allowed, true);
    assert.equal(cors.evaluate("https://evil.example").allowed, false);
    assert.equal(cors.evaluate("https://evil.example").statusCode, 403);
  });

  it("enforces a fixed-window rate limit by hashed key", () => {
    let now = 10_000;
    const limiter = createRateLimiter({
      max: 2,
      now: () => now,
      windowMs: 1_000
    });

    const first = limiter.check("192.0.2.10");
    const second = limiter.check("192.0.2.10");
    const third = limiter.check("192.0.2.10");

    assert.equal(first.allowed, true);
    assert.equal(second.allowed, true);
    assert.equal(third.allowed, false);
    assert.notEqual(first.key, "192.0.2.10");

    now += 1_001;
    assert.equal(limiter.check("192.0.2.10").allowed, true);
  });

  it("validates procedure envelopes before API handling", () => {
    const request = validateApiProcedureRequest({
      payload: { includeBuild: true },
      procedure: "system.health",
      requestId: "req_12345678"
    });

    assert.deepEqual(request.payload, { includeBuild: true });
    assert.equal(request.policy.rateLimit.max, 60);

    assert.throws(
      () =>
        validateApiProcedureRequest({
          payload: { includeBuild: "yes" },
          procedure: "system.health",
          requestId: "req_12345678"
        }),
      ContractValidationError
    );
  });

  it("redacts unexpected errors and validation values", () => {
    const validationError = new ContractValidationError("Invalid password=secret", [
      {
        code: "invalid_type",
        expected: "boolean",
        path: ["payload", "includeBuild"],
        received: "password=secret"
      }
    ]);

    const validationResponse = redactApiError(validationError, {
      requestId: "req_12345678"
    });
    const serverResponse = redactApiError(
      new Error("Neon URL postgres://user:pass@example.invalid/db leaked"),
      { requestId: "req_12345678" }
    );

    assert.deepEqual(validationResponse, {
      error: {
        code: "BAD_REQUEST",
        issues: [{ code: "invalid_type", path: ["payload", "includeBuild"] }],
        message: "Request validation failed.",
        requestId: "req_12345678"
      }
    });
    assert.equal(serverResponse.error.message, "Unexpected server error.");
    assert.equal(JSON.stringify(serverResponse).includes("postgres://"), false);
  });

  it("creates least-privilege request context and log events", () => {
    const context = createRequestContext({
      headers: {
        authorization: "Bearer secret",
        cookie: "sid=secret",
        origin: "http://localhost:1420",
        "user-agent": "node-test",
        "x-request-id": "req_abcdef12"
      },
      method: "POST",
      procedure: "system.health",
      remoteAddress: "192.0.2.30",
      url: "/trpc/system.health"
    });

    const logEvent = createLeastPrivilegeLogEvent(context, {
      durationMs: 12,
      now: 1_700_000_000_000,
      statusCode: 200
    });

    assert.equal(context.requestId, "req_abcdef12");
    assert.equal(context.headers.authorization, undefined);
    assert.equal(context.headers.cookie, undefined);
    assert.equal(logEvent.remoteAddressHash, undefined);
    assert.equal(context.rateLimitKey === "192.0.2.30", false);
    assert.equal(logEvent.statusCode, 200);
  });

  it("wires CORS, validation, rate limit, redaction, and logging together", () => {
    const baseline = createApiSecurityBaseline({
      config: {
        cors: { allowedOrigins: ["http://localhost:1420"] },
        errorRedactionEnabled: true,
        logging: { level: "info" },
        rateLimit: { max: 1, windowMs: 60_000 },
        requestIdHeader: "x-request-id"
      }
    });

    const context = baseline.createContext({
      headers: {
        origin: "http://localhost:1420",
        "x-request-id": "req_abcdef12"
      },
      method: "POST",
      procedure: "system.health",
      remoteAddress: "127.0.0.1",
      url: "/trpc/system.health"
    });

    assert.equal(baseline.cors.evaluate(context.origin).allowed, true);
    assert.equal(baseline.rateLimiter.check(context.rateLimitKey).allowed, true);
    assert.equal(baseline.rateLimiter.check(context.rateLimitKey).allowed, false);
    assert.equal(
      baseline.validateProcedure({
        payload: {},
        procedure: "system.health",
        requestId: context.requestId
      }).procedure,
      "system.health"
    );
    assert.equal(baseline.logEvent(context, { statusCode: 200 }).requestId, "req_abcdef12");
  });
});
