import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  API_TRPC_PATH,
  ContractValidationError,
  apiProcedureContracts
} from "../../../packages/api-contract/src/index.js";
import {
  createFastifyApiServer,
  createTrpcApiRouter,
  registerFastifyTrpcApi
} from "../src/index.js";

describe("Fastify tRPC API scaffold", () => {
  it("creates typed callers for query and mutation procedures", async () => {
    const router = createTrpcApiRouter({
      build: "test-build",
      now: () => 1_700_000_000_000,
      publishedAtUtc: "2026-05-02T00:00:00Z",
      version: "0.1.0"
    });
    const caller = router.createCaller({ requestId: "req_abcdef12" });

    assert.equal(router._def.runtime, "trpc");
    assert.equal(router._def.procedures["system.health"].kind, "query");
    assert.deepEqual(Object.keys(router._def.procedures).sort(), Object.keys(apiProcedureContracts).sort());

    const health = await caller.query("system.health", { includeBuild: true });
    assert.deepEqual(health, {
      build: "test-build",
      ok: true,
      service: "@liiiraa/api",
      uptimeMs: 0,
      version: "0.1.0"
    });

    const catalog = await caller.query("catalog.latest", { channel: "beta" });
    assert.deepEqual(catalog, {
      catalogVersion: "local-dev",
      channel: "beta",
      publishedAtUtc: "2026-05-02T00:00:00Z"
    });

    const benchmark = await caller.mutation("benchmarks.sync", benchmarkSyncPayload());
    assert.equal(benchmark.accepted, true);
    assert.equal(benchmark.statusCode, 202);

    await assert.rejects(
      () => caller.query("benchmarks.sync", benchmarkSyncPayload()),
      ContractValidationError
    );
  });

  it("registers Fastify health and tRPC routes with scoped CORS and request IDs", async () => {
    const app = createFakeFastify();
    const registration = registerFastifyTrpcApi(app, {
      now: () => 1_700_000_000_000,
      publishedAtUtc: "2026-05-02T00:00:00Z"
    });

    assert.equal(registration.trpcPath, API_TRPC_PATH);
    assert.deepEqual(
      app.routes.map((route) => `${route.method} ${route.path}`).sort(),
      ["GET /health", "OPTIONS /trpc/:procedure", "POST /trpc/:procedure"]
    );

    const healthReply = createFakeReply();
    await app.route("GET", "/health").handler(
      {
        headers: {},
        method: "GET",
        url: "/health"
      },
      healthReply
    );
    assert.equal(healthReply.statusCode, 200);
    assert.equal(healthReply.body.ok, true);

    const trpcReply = createFakeReply();
    await app.route("POST", "/trpc/:procedure").handler(
      {
        body: {
          input: benchmarkSyncPayload()
        },
        headers: {
          origin: "http://localhost:1420",
          "x-request-id": "req_abcdef12"
        },
        method: "POST",
        params: {
          procedure: "benchmarks.sync"
        },
        url: "/trpc/benchmarks.sync"
      },
      trpcReply
    );
    assert.equal(trpcReply.statusCode, 202);
    assert.equal(trpcReply.headers["x-request-id"], "req_abcdef12");
    assert.equal(trpcReply.body.accepted, true);

    const deniedReply = createFakeReply();
    await app.route("POST", "/trpc/:procedure").handler(
      {
        body: {},
        headers: {
          origin: "https://evil.example"
        },
        method: "POST",
        params: {
          procedure: "system.health"
        },
        url: "/trpc/system.health"
      },
      deniedReply
    );
    assert.equal(deniedReply.statusCode, 403);
  });

  it("can create the Fastify app through an injected factory", async () => {
    const app = createFakeFastify();
    const created = await createFastifyApiServer({
      fastify: () => app,
      now: () => 1_700_000_000_000
    });

    assert.equal(created, app);
    assert.equal(app.route("POST", "/trpc/:procedure").path, "/trpc/:procedure");
  });
});

function benchmarkSyncPayload() {
  return {
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
}

function createFakeFastify() {
  const routes = [];

  return {
    routes,
    get(path, handler) {
      routes.push({ handler, method: "GET", path });
    },
    options(path, handler) {
      routes.push({ handler, method: "OPTIONS", path });
    },
    post(path, handler) {
      routes.push({ handler, method: "POST", path });
    },
    route(method, path) {
      return routes.find((route) => route.method === method && route.path === path);
    }
  };
}

function createFakeReply() {
  return {
    body: undefined,
    headers: {},
    statusCode: 200,
    code(statusCode) {
      this.statusCode = statusCode;
      return this;
    },
    header(name, value) {
      this.headers[name.toLowerCase()] = value;
      return this;
    },
    send(body) {
      this.body = body;
      return this;
    }
  };
}
