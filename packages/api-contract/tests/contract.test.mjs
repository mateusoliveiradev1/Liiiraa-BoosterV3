import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  API_AUTH_BOUNDARY_CONTRACT_VERSION,
  API_TRPC_PATH,
  ContractValidationError,
  RELEASE_CHANNELS,
  RELEASE_PLATFORMS,
  assertAuthReadyBoundaryCoverage,
  assertProcedureSecurityCoverage,
  assertTypedApiContractCoverage,
  apiProcedureContracts,
  listPrivateApiProcedures,
  listPublicApiProcedures,
  privateApiProcedureContracts,
  privateProcedureSecurityPolicies,
  publicProcedureSecurityPolicies,
  validateApiContractInput,
  validateApiContractOutput,
  validateSecurityEnvelope
} from "../src/index.js";
import {
  DEFAULT_SIGNED_TWEAK_CATALOG
} from "../../catalog/src/fixture.js";

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
    assert.equal(assertTypedApiContractCoverage(), true);

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

  it("declares tRPC contract metadata for every public procedure", () => {
    assert.deepEqual(listPublicApiProcedures(), [
      "benchmarks.sync",
      "catalog.latest",
      "featureflags.evaluate",
      "releases.channels",
      "releases.latest",
      "system.health"
    ]);

    for (const procedure of listPublicApiProcedures()) {
      assert.equal(apiProcedureContracts[procedure].path, API_TRPC_PATH);
      assert.equal(apiProcedureContracts[procedure].visibility, "public");
      assert.equal(
        publicProcedureSecurityPolicies[procedure].inputSchema,
        apiProcedureContracts[procedure].inputSchema
      );
    }
  });

  it("keeps reserved private procedures outside the shipped public API", () => {
    assert.equal(API_AUTH_BOUNDARY_CONTRACT_VERSION, "0.1.0");
    assert.equal(assertAuthReadyBoundaryCoverage(), true);
    assert.deepEqual(listPrivateApiProcedures(), [
      "account.profile",
      "devices.register",
      "licenses.status"
    ]);
    assert.deepEqual(listPrivateApiProcedures(apiProcedureContracts), []);

    for (const procedure of listPrivateApiProcedures()) {
      assert.equal(apiProcedureContracts[procedure], undefined);
      assert.equal(publicProcedureSecurityPolicies[procedure], undefined);
      assert.equal(privateApiProcedureContracts[procedure].visibility, "private");
      assert.equal(privateProcedureSecurityPolicies[procedure].auth.required, true);
      assert.equal(privateProcedureSecurityPolicies[procedure].errorRedaction, "private");
    }

    assert.throws(
      () =>
        validateSecurityEnvelope({
          payload: {
            appVersion: "0.1.0",
            deviceId: "device:abc123",
            installId: "install:abc123"
          },
          procedure: "devices.register",
          requestId: "req_12345678"
        }),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some(
          (issue) => issue.code === "unknown_procedure" && issue.path.join(".") === "procedure"
        )
    );
  });

  it("requires future private contracts to stay auth-gated and separate", () => {
    assert.deepEqual(
      validateApiContractInput(
        "devices.register",
        {
          appVersion: "0.1.0",
          channel: "beta",
          deviceId: "device:abc123",
          installId: "install:abc123"
        },
        privateApiProcedureContracts
      ),
      {
        appVersion: "0.1.0",
        channel: "beta",
        deviceId: "device:abc123",
        installId: "install:abc123"
      }
    );
    assert.deepEqual(
      validateApiContractOutput(
        "licenses.status",
        {
          canSyncBenchmarks: true,
          plan: "pro",
          status: "active"
        },
        privateApiProcedureContracts
      ),
      {
        canSyncBenchmarks: true,
        plan: "pro",
        status: "active"
      }
    );

    assert.throws(
      () =>
        assertAuthReadyBoundaryCoverage(
          {
            ...apiProcedureContracts,
            "devices.register": privateApiProcedureContracts["devices.register"]
          },
          privateApiProcedureContracts,
          publicProcedureSecurityPolicies,
          privateProcedureSecurityPolicies
        ),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some((issue) => issue.code === "private_procedure_exposed")
    );

    assert.throws(
      () =>
        assertAuthReadyBoundaryCoverage(
          apiProcedureContracts,
          privateApiProcedureContracts,
          publicProcedureSecurityPolicies,
          {
            ...privateProcedureSecurityPolicies,
            "devices.register": {
              ...privateProcedureSecurityPolicies["devices.register"],
              auth: {
                required: false,
                scopes: []
              }
            }
          }
        ),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some((issue) => issue.code === "missing_auth_requirement")
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

  it("declares dev beta stable release channel contracts", () => {
    assert.deepEqual(RELEASE_CHANNELS, ["dev", "beta", "stable"]);
    assert.deepEqual(RELEASE_PLATFORMS, ["windows-x64"]);

    assert.deepEqual(validateApiContractInput("releases.latest", { channel: "beta", platform: "windows-x64" }), {
      channel: "beta",
      platform: "windows-x64"
    });
    assert.deepEqual(
      validateApiContractOutput("releases.channels", {
        channels: [
          {
            description: "Internal builds.",
            id: "dev",
            requiresSignedArtifacts: false,
            riskyChangesFirst: true,
            title: "Dev"
          },
          {
            description: "Signed soak builds.",
            id: "beta",
            requiresSignedArtifacts: true,
            riskyChangesFirst: true,
            title: "Beta"
          },
          {
            description: "Public signed builds.",
            id: "stable",
            requiresSignedArtifacts: true,
            riskyChangesFirst: false,
            title: "Stable"
          }
        ],
        defaultChannel: "stable",
        version: "0.1.0"
      }).channels.map((channel) => channel.id),
      ["dev", "beta", "stable"]
    );
  });

  it("validates public feature flag evaluation contracts", () => {
    const input = validateApiContractInput("featureflags.evaluate", {
      channel: "stable",
      deviceId: "device:abc123",
      flagKeys: ["optimizer.labTweaks"]
    });

    assert.deepEqual(input, {
      channel: "stable",
      deviceId: "device:abc123",
      flagKeys: ["optimizer.labTweaks"]
    });

    assert.deepEqual(
      validateApiContractOutput("featureflags.evaluate", {
        channel: "stable",
        evaluations: [
          {
            enabled: false,
            key: "optimizer.labTweaks",
            reason: "Stable keeps Lab tweak surfaces disabled until explicit approval.",
            rolloutPercent: 0,
            source: "default",
            variant: "off"
          }
        ],
        version: "0.1.0"
      }).evaluations[0].source,
      "default"
    );

    assert.throws(
      () =>
        validateApiContractInput("featureflags.evaluate", {
          channel: "stable",
          flagKeys: []
        }),
      ContractValidationError
    );
  });

  it("validates typed procedure input and output contracts", () => {
    assert.deepEqual(validateApiContractInput("system.health", { includeBuild: true }), {
      includeBuild: true
    });
    assert.deepEqual(
      validateApiContractOutput("system.health", {
        build: "local",
        ok: true,
        service: "@liiiraa/api",
        uptimeMs: 12,
        version: "0.0.0"
      }),
      {
        build: "local",
        ok: true,
        service: "@liiiraa/api",
        uptimeMs: 12,
        version: "0.0.0"
      }
    );

    assert.throws(
      () =>
        validateApiContractOutput("catalog.latest", {
          catalogVersion: "v1",
          channel: "stable",
          integrity: DEFAULT_SIGNED_TWEAK_CATALOG.integrity,
          payload: DEFAULT_SIGNED_TWEAK_CATALOG.payload,
          publishedAtUtc: "2026-05-02T00:00:00Z",
          schemaVersion: "1",
          signature: DEFAULT_SIGNED_TWEAK_CATALOG.signature,
          neonUrl: "postgres://secret"
        }),
      (error) =>
        error instanceof ContractValidationError &&
        error.issues.some((issue) => issue.code === "unknown_key" && issue.path.join(".") === "output.neonUrl")
    );
  });
});
