import {
  API_TRPC_CONTRACT_VERSION,
  ContractValidationError,
  apiProcedureContracts,
  assertTypedApiContractCoverage,
  getApiProcedureContract,
  validateApiContractInput,
  validateApiContractOutput
} from "../../../packages/api-contract/src/index.js";
import { createBenchmarkSessionSyncDecision } from "./benchmark-sync.js";
import { createCatalogDeliveryResponse } from "./catalog-delivery.js";
import { createFeatureFlagEvaluationResponse } from "./feature-flags.js";
import {
  createLatestReleaseResponse,
  createReleaseChannelsResponse
} from "./release-channels.js";
import { createApiSecurityBaseline } from "./security-baseline.js";

const DEFAULT_REQUEST_ID = "req_local0000";

export function createApiProcedureHandlers(options = {}) {
  const now = typeof options.now === "function" ? options.now : () => Date.now();
  const startedAt = now();
  const buildInfo = {
    build: options.build ?? "local",
    service: options.service ?? "@liiiraa/api",
    version: options.version ?? "0.0.0"
  };

  return {
    "catalog.latest": async ({ input }) => createCatalogDeliveryResponse(input, options),
    "featureflags.evaluate": async ({ input }) => createFeatureFlagEvaluationResponse(input, options),
    "releases.channels": async () => createReleaseChannelsResponse(options),
    "releases.latest": async ({ input }) => createLatestReleaseResponse(input, options),
    "benchmarks.sync": async ({ envelope }) => {
      const decision = createBenchmarkSessionSyncDecision(envelope);

      return {
        accepted: decision.accepted,
        requestId: decision.requestId,
        statusCode: decision.statusCode,
        version: decision.version
      };
    },
    "system.health": async ({ input }) => {
      const response = {
        ok: true,
        service: buildInfo.service,
        uptimeMs: Math.max(0, now() - startedAt),
        version: buildInfo.version
      };

      if (input.includeBuild === true) {
        response.build = buildInfo.build;
      }

      return response;
    }
  };
}

export function createTrpcApiRouter(options = {}) {
  const contracts = options.contracts ?? apiProcedureContracts;
  const handlers = options.handlers ?? createApiProcedureHandlers(options);
  const baseline = options.baseline ?? createApiSecurityBaseline(options);

  assertTypedApiContractCoverage(contracts);

  return {
    _def: {
      procedures: Object.fromEntries(
        Object.entries(contracts).map(([procedure, contract]) => [
          procedure,
          {
            kind: contract.kind,
            path: contract.path,
            visibility: contract.visibility
          }
        ])
      ),
      runtime: "trpc",
      version: API_TRPC_CONTRACT_VERSION
    },
    call(procedure, payload = {}, context = {}) {
      return callApiProcedure({
        baseline,
        context,
        contracts,
        handlers,
        payload,
        procedure
      });
    },
    createCaller(context = {}) {
      return {
        call: (procedure, payload = {}) =>
          callApiProcedure({
            baseline,
            context,
            contracts,
            handlers,
            payload,
            procedure
          }),
        mutation: (procedure, payload = {}) =>
          callApiProcedure({
            baseline,
            context,
            contracts,
            handlers,
            kind: "mutation",
            payload,
            procedure
          }),
        query: (procedure, payload = {}) =>
          callApiProcedure({
            baseline,
            context,
            contracts,
            handlers,
            kind: "query",
            payload,
            procedure
          })
      };
    }
  };
}

export async function callApiProcedure(options = {}) {
  const procedure = options.procedure;
  const contract = getApiProcedureContract(procedure, options.contracts);

  if (options.kind && contract.kind !== options.kind) {
    throw new ContractValidationError("API procedure kind mismatch", [
      {
        code: "invalid_kind",
        expected: contract.kind,
        path: ["procedure"]
      }
    ]);
  }

  const handler = options.handlers?.[procedure];
  if (typeof handler !== "function") {
    throw new ContractValidationError("API procedure handler is not registered", [
      {
        code: "missing_handler",
        expected: "registered procedure handler",
        path: ["procedure"]
      }
    ]);
  }

  const requestId = options.context?.requestId ?? DEFAULT_REQUEST_ID;
  const envelope = {
    payload: options.payload ?? {},
    procedure,
    requestId
  };
  const validated = options.baseline.validateProcedure(envelope);
  const response = await handler({
    context: options.context ?? {},
    contract,
    envelope: {
      ...envelope,
      payload: validated.payload
    },
    input: validated.payload
  });

  return validateApiContractOutput(procedure, response, options.contracts);
}

export async function createNativeTrpcApiRouter(options = {}) {
  const { initTRPC } = await import("@trpc/server");
  const t = initTRPC.context().create();
  const router = createTrpcApiRouter(options);

  return t.router({
    benchmarks: t.router({
      sync: t.procedure
        .input(createTrpcInputParser("benchmarks.sync", options.contracts))
        .mutation(({ ctx, input }) => router.call("benchmarks.sync", input, normalizeTrpcContext(ctx)))
    }),
    catalog: t.router({
      latest: t.procedure
        .input(createTrpcInputParser("catalog.latest", options.contracts))
        .query(({ ctx, input }) => router.call("catalog.latest", input, normalizeTrpcContext(ctx)))
    }),
    featureflags: t.router({
      evaluate: t.procedure
        .input(createTrpcInputParser("featureflags.evaluate", options.contracts))
        .query(({ ctx, input }) => router.call("featureflags.evaluate", input, normalizeTrpcContext(ctx)))
    }),
    releases: t.router({
      channels: t.procedure
        .input(createTrpcInputParser("releases.channels", options.contracts))
        .query(({ ctx, input }) => router.call("releases.channels", input, normalizeTrpcContext(ctx))),
      latest: t.procedure
        .input(createTrpcInputParser("releases.latest", options.contracts))
        .query(({ ctx, input }) => router.call("releases.latest", input, normalizeTrpcContext(ctx)))
    }),
    system: t.router({
      health: t.procedure
        .input(createTrpcInputParser("system.health", options.contracts))
        .query(({ ctx, input }) => router.call("system.health", input, normalizeTrpcContext(ctx)))
    })
  });
}

function createTrpcInputParser(procedure, contracts) {
  return {
    parse(value) {
      return validateApiContractInput(procedure, value ?? {}, contracts);
    }
  };
}

function normalizeTrpcContext(context) {
  return context && typeof context === "object" ? context : {};
}
