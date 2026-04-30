import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ContractValidationError,
  assertProcedureSecurityCoverage,
  publicProcedureSecurityPolicies,
  validateSecurityEnvelope
} from "../src/index.js";

describe("API contract security policies", () => {
  it("requires every public procedure to define baseline controls", () => {
    assert.equal(assertProcedureSecurityCoverage(), true);

    assert.throws(
      () =>
        assertProcedureSecurityCoverage({
          "unsafe.missing": {
            inputSchema: undefined,
            logging: {},
            rateLimit: undefined
          }
        }),
      ContractValidationError
    );
  });

  it("validates known procedure input and returns its policy", () => {
    const envelope = validateSecurityEnvelope({
      payload: {
        channel: "stable",
        clientVersion: "0.1.0"
      },
      procedure: "catalog.latest",
      requestId: "req_12345678"
    });

    assert.deepEqual(envelope.payload, {
      channel: "stable",
      clientVersion: "0.1.0"
    });
    assert.equal(envelope.policy, publicProcedureSecurityPolicies["catalog.latest"]);
  });

  it("denies unknown procedures and extra payload keys", () => {
    assert.throws(
      () =>
        validateSecurityEnvelope({
          payload: {},
          procedure: "admin.dumpSecrets",
          requestId: "req_12345678"
        }),
      /Request failed API contract validation/
    );

    assert.throws(
      () =>
        validateSecurityEnvelope({
          payload: {
            includeBuild: true,
            password: "do-not-log"
          },
          procedure: "system.health",
          requestId: "req_12345678"
        }),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some((issue) => issue.code === "unknown_key" && issue.path.join(".") === "payload.password")
    );
  });

  it("normalizes optional string fields without accepting empty values", () => {
    const envelope = validateSecurityEnvelope({
      payload: {
        clientVersion: "  1.2.3  "
      },
      procedure: "catalog.latest",
      requestId: "req_12345678"
    });

    assert.equal(envelope.payload.clientVersion, "1.2.3");

    assert.throws(
      () =>
        validateSecurityEnvelope({
          payload: {
            clientVersion: "   "
          },
          procedure: "catalog.latest",
          requestId: "req_12345678"
        }),
      ContractValidationError
    );
  });
});
