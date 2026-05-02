import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ContractValidationError,
  assertProcedureSecurityCoverage,
  publicProcedureSecurityPolicies,
  validateSecurityEnvelope
} from "../src/index.js";

describe("API contract security policies", () => {
  const benchmarkSyncPayload = {
    consent: {
      benchmarkSync: true
    },
    session: {
      activeOptimizerProfile: "Safe",
      activePowerPlan: "Liiiraa Balanced",
      captures: [
        {
          averageFps: 182.4,
          capturedAtUtc: "2026-04-30T12:02:00Z",
          delayedFrames: 1,
          droppedFrames: 2,
          frametimeP50Ms: 5.4,
          frametimeP95Ms: 8.2,
          frametimeP99Ms: 11.8,
          generatedFramesDetected: false,
          id: "capture:before:001",
          latencyProxy: true,
          measurementSource: "presentmon-render-present",
          onePercentLowFps: 122,
          phase: "before",
          zeroPointOnePercentLowFps: 91.5
        }
      ],
      createdAtUtc: "2026-04-30T12:03:00Z",
      driverVersion: "551.86",
      game: "PUBG",
      id: "bench:session:pubg:001",
      sessionLabel: "Training route",
      windowsBuild: "22631.3527"
    }
  };

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

  it("validates the benchmark sync procedure with consent and aggregate metrics only", () => {
    const envelope = validateSecurityEnvelope({
      payload: benchmarkSyncPayload,
      procedure: "benchmarks.sync",
      requestId: "req_12345678"
    });

    assert.equal(envelope.payload.consent.benchmarkSync, true);
    assert.equal(envelope.payload.session.captures[0].phase, "before");
    assert.equal(envelope.policy, publicProcedureSecurityPolicies["benchmarks.sync"]);

    assert.throws(
      () =>
        validateSecurityEnvelope({
          payload: {
            ...benchmarkSyncPayload,
            session: {
              ...benchmarkSyncPayload.session,
              captures: [
                {
                  ...benchmarkSyncPayload.session.captures[0],
                  rawCsvPath: "C:\\Users\\liiiraa\\captures\\session.csv"
                }
              ]
            }
          },
          procedure: "benchmarks.sync",
          requestId: "req_12345678"
        }),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some(
          (issue) =>
            issue.code === "unknown_key" &&
            issue.path.join(".") === "payload.session.captures.0.rawCsvPath"
        )
    );
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
