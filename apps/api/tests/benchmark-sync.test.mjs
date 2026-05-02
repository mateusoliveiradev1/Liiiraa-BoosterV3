import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { ContractValidationError } from "../../../packages/api-contract/src/index.js";
import {
  PrivacyConsentRequiredError,
  assertApiBenchmarkSyncCoverage,
  createBenchmarkSessionSyncDecision,
  requireBenchmarkSessionCloudSync
} from "../src/index.js";

describe("benchmark session cloud sync", () => {
  it("accepts aggregate benchmark sessions only after explicit consent", () => {
    const decision = createBenchmarkSessionSyncDecision(syncEnvelope());

    assert.equal(decision.accepted, true);
    assert.equal(decision.statusCode, 202);
    assert.equal(decision.payload.kind, "benchmark-session");
    assert.deepEqual(decision.payload.session.environment, {
      activeOptimizerProfile: "Safe",
      activePowerPlan: "Liiiraa Balanced",
      driverVersion: "551.86",
      windowsBuild: "22631.3527"
    });
    assert.deepEqual(Object.keys(decision.payload.session.captures[0]).sort(), [
      "capturedAtUtc",
      "generatedFramesDetected",
      "id",
      "latencyProxy",
      "measurementSource",
      "metrics",
      "phase"
    ]);
  });

  it("keeps benchmark sessions local without consent", () => {
    const decision = createBenchmarkSessionSyncDecision(
      syncEnvelope({
        consent: {}
      })
    );

    assert.equal(decision.accepted, false);
    assert.equal(decision.payload, undefined);
    assert.equal(decision.statusCode, 403);
    assert.equal(decision.gate.action, "keep-local");

    assert.throws(
      () =>
        requireBenchmarkSessionCloudSync(
          syncEnvelope({
            consent: {}
          })
        ),
      PrivacyConsentRequiredError
    );
  });

  it("rejects local paths and unknown raw capture fields before sync", () => {
    assert.throws(
      () =>
        createBenchmarkSessionSyncDecision(
          syncEnvelope({
            session: {
              ...benchmarkSession(),
              sessionLabel: "C:\\Users\\liiiraa\\captures\\session.csv"
            }
          })
        ),
      ContractValidationError
    );

    assert.throws(
      () =>
        createBenchmarkSessionSyncDecision(
          syncEnvelope({
            session: {
              ...benchmarkSession(),
              captures: [
                {
                  ...benchmarkSession().captures[0],
                  rawCsvPath: "C:\\Users\\liiiraa\\captures\\session.csv"
                }
              ]
            }
          })
        ),
      ContractValidationError
    );
  });

  it("covers the API benchmark sync consent contract", () => {
    assert.equal(assertApiBenchmarkSyncCoverage(), true);
  });
});

function syncEnvelope(overrides = {}) {
  return {
    payload: {
      consent: overrides.consent ?? {
        benchmarkSync: true
      },
      session: overrides.session ?? benchmarkSession()
    },
    procedure: "benchmarks.sync",
    requestId: "req_12345678"
  };
}

function benchmarkSession() {
  return {
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
  };
}
