import { ContractValidationError } from "../../../packages/api-contract/src/index.js";
import { PrivacyConsentRequiredError, evaluateApiPrivacyConsentGate } from "./privacy-consent.js";
import { validateApiProcedureRequest } from "./security-baseline.js";

export const API_BENCHMARK_SYNC_VERSION = "0.1.0";
export const BENCHMARK_SYNC_PROCEDURE = "benchmarks.sync";

const LOCAL_PATH_PATTERN = /(?:[A-Za-z]:[\\/]|\\\\[^\\/\s]+[\\/]|\/(?:Users|home|mnt|tmp|var)\/)/i;

export function validateBenchmarkSyncEnvelope(envelope = {}) {
  const request = validateApiProcedureRequest({
    ...envelope,
    procedure: envelope.procedure ?? BENCHMARK_SYNC_PROCEDURE
  });

  if (request.procedure !== BENCHMARK_SYNC_PROCEDURE) {
    throw contractError(
      "Benchmark sync requests must use the benchmark sync procedure.",
      ["procedure"],
      "invalid_procedure",
      BENCHMARK_SYNC_PROCEDURE
    );
  }

  assertNoLocalPaths(request.payload.session, ["payload", "session"]);
  return request;
}

export function createBenchmarkSessionSyncDecision(envelope = {}, options = {}) {
  const request = validateBenchmarkSyncEnvelope(envelope);
  const gate = evaluateApiPrivacyConsentGate({
    consent: request.payload.consent,
    destination: options.destination ?? "cloud",
    kind: "benchmark-sync",
    requestId: request.requestId
  });
  const accepted = gate.allowed && gate.destination === "cloud";

  return {
    accepted,
    gate,
    payload: accepted ? createBenchmarkSessionSyncPayload(request.payload.session) : undefined,
    procedure: BENCHMARK_SYNC_PROCEDURE,
    requestId: request.requestId,
    statusCode: gate.statusCode,
    version: API_BENCHMARK_SYNC_VERSION
  };
}

export function requireBenchmarkSessionCloudSync(envelope = {}, options = {}) {
  const decision = createBenchmarkSessionSyncDecision(envelope, options);

  if (!decision.accepted) {
    throw new PrivacyConsentRequiredError(decision.gate);
  }

  return decision;
}

export function createBenchmarkSessionSyncPayload(session) {
  return {
    kind: "benchmark-session",
    schemaVersion: API_BENCHMARK_SYNC_VERSION,
    session: {
      captures: session.captures.map((capture) => ({
        generatedFramesDetected: capture.generatedFramesDetected,
        id: capture.id,
        capturedAtUtc: capture.capturedAtUtc,
        latencyProxy: capture.latencyProxy,
        measurementSource: capture.measurementSource,
        metrics: {
          averageFps: capture.averageFps,
          delayedFrames: capture.delayedFrames,
          droppedFrames: capture.droppedFrames,
          frametimeP50Ms: capture.frametimeP50Ms,
          frametimeP95Ms: capture.frametimeP95Ms,
          frametimeP99Ms: capture.frametimeP99Ms,
          onePercentLowFps: capture.onePercentLowFps,
          zeroPointOnePercentLowFps: capture.zeroPointOnePercentLowFps
        },
        phase: capture.phase
      })),
      createdAtUtc: session.createdAtUtc,
      environment: {
        activeOptimizerProfile: session.activeOptimizerProfile,
        activePowerPlan: session.activePowerPlan,
        driverVersion: session.driverVersion,
        windowsBuild: session.windowsBuild
      },
      game: session.game,
      id: session.id,
      sessionLabel: session.sessionLabel
    }
  };
}

export function assertApiBenchmarkSyncCoverage() {
  const envelope = {
    payload: {
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
    },
    procedure: BENCHMARK_SYNC_PROCEDURE,
    requestId: "req_12345678"
  };

  const accepted = createBenchmarkSessionSyncDecision(envelope);
  if (!accepted.accepted || accepted.payload.session.captures.length !== 1) {
    throw new Error("Benchmark sync must accept consented aggregate benchmark sessions.");
  }

  const denied = createBenchmarkSessionSyncDecision({
    ...envelope,
    payload: {
      ...envelope.payload,
      consent: {}
    }
  });
  if (denied.accepted || denied.statusCode !== 403) {
    throw new Error("Benchmark sync must be rejected without explicit consent.");
  }

  try {
    createBenchmarkSessionSyncDecision({
      ...envelope,
      payload: {
        ...envelope.payload,
        session: {
          ...envelope.payload.session,
          sessionLabel: "C:\\Users\\liiiraa\\captures\\session.csv"
        }
      }
    });
  } catch (error) {
    if (error instanceof ContractValidationError) {
      return true;
    }
    throw error;
  }

  throw new Error("Benchmark sync must reject local path strings.");
}

function assertNoLocalPaths(value, path) {
  if (typeof value === "string") {
    if (LOCAL_PATH_PATTERN.test(value)) {
      throw contractError(
        "Benchmark sync payload contains local path data.",
        path,
        "unsafe_local_path",
        "redacted benchmark metadata"
      );
    }
    return;
  }

  if (Array.isArray(value)) {
    value.forEach((item, index) => assertNoLocalPaths(item, [...path, index]));
    return;
  }

  if (value && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) {
      assertNoLocalPaths(child, [...path, key]);
    }
  }
}

function contractError(message, path, code, expected) {
  return new ContractValidationError(message, [
    {
      code,
      expected,
      path
    }
  ]);
}
