import { API_TRPC_PATH } from "../../../packages/api-contract/src/index.js";
import { createApiSecurityBaseline } from "./security-baseline.js";
import { createNativeTrpcApiRouter, createTrpcApiRouter } from "./trpc-router.js";

const DEFAULT_HOST = "127.0.0.1";
const DEFAULT_PORT = 8787;

export async function createFastifyApiServer(options = {}) {
  const fastifyFactory = options.fastify ?? (await import("fastify")).default;
  const app =
    typeof fastifyFactory === "function"
      ? fastifyFactory({
          logger: options.logger ?? false
        })
      : fastifyFactory;

  registerFastifyTrpcApi(app, options);
  return app;
}

export function registerFastifyTrpcApi(app, options = {}) {
  const baseline = options.baseline ?? createApiSecurityBaseline(options);
  const router =
    options.router ??
    createTrpcApiRouter({
      ...options,
      baseline
    });
  const trpcPath = options.trpcPath ?? API_TRPC_PATH;

  app.get?.("/health", async (request, reply) =>
    handleFastifyProcedureRequest({
      baseline,
      payload: { includeBuild: false },
      procedure: "system.health",
      reply,
      request,
      router
    })
  );

  app.options?.(`${trpcPath}/:procedure`, async (request, reply) =>
    handleCorsPreflight({
      baseline,
      reply,
      request
    })
  );

  app.post?.(`${trpcPath}/:procedure`, async (request, reply) =>
    handleFastifyProcedureRequest({
      baseline,
      payload: readTrpcPayload(request.body),
      procedure: request.params?.procedure,
      reply,
      request,
      router
    })
  );

  return {
    app,
    router,
    trpcPath
  };
}

export async function startApiServer(options = {}) {
  const app = options.app ?? (await createFastifyApiServer(options));
  const port = options.port ?? parsePort(options.env?.PORT, DEFAULT_PORT);
  const host = options.host ?? options.env?.HOST ?? DEFAULT_HOST;

  await app.listen({ host, port });
  return {
    app,
    host,
    port
  };
}

export async function handleFastifyProcedureRequest(options = {}) {
  const cors = options.baseline.cors.evaluate(readHeader(options.request, "origin"));
  if (!cors.allowed) {
    return sendJson(options.reply, cors.statusCode ?? 403, { error: { code: "FORBIDDEN" } }, cors.headers);
  }

  const context = options.baseline.createContext({
    headers: options.request.headers,
    method: options.request.method,
    procedure: options.procedure,
    remoteAddress: options.request.ip ?? options.request.socket?.remoteAddress,
    url: options.request.url
  });
  const rateLimit = options.baseline.rateLimiter.check(context.rateLimitKey);
  const responseHeaders = {
    ...cors.headers,
    [options.baseline.config.requestIdHeader]: context.requestId
  };

  if (!rateLimit.allowed) {
    const error = new Error("Rate limit exceeded.");
    error.code = "TOO_MANY_REQUESTS";
    return sendJson(options.reply, 429, options.baseline.redactError(error, context), responseHeaders);
  }

  try {
    const output = await options.router.call(options.procedure, options.payload, context);
    return sendJson(options.reply, output.statusCode ?? 200, output, responseHeaders);
  } catch (error) {
    return sendJson(
      options.reply,
      error.statusCode ?? (error.code === "BAD_REQUEST" ? 400 : 500),
      options.baseline.redactError(error, context),
      responseHeaders
    );
  }
}

export function createTrpcFastifyAdapterOptions(options = {}) {
  return {
    prefix: options.trpcPath ?? API_TRPC_PATH,
    trpcOptions: {
      createContext: options.createContext,
      router: options.router ?? createTrpcApiRouter(options)
    }
  };
}

export async function createNativeTrpcFastifyAdapterOptions(options = {}) {
  return {
    prefix: options.trpcPath ?? API_TRPC_PATH,
    trpcOptions: {
      createContext:
        options.createContext ??
        (({ req }) => ({
          requestId: req.headers?.["x-request-id"]
        })),
      router: options.router ?? (await createNativeTrpcApiRouter(options))
    }
  };
}

export async function registerNativeTrpcFastifyAdapter(app, options = {}) {
  const { fastifyTRPCPlugin } = await import("@trpc/server/adapters/fastify");
  const adapterOptions = await createNativeTrpcFastifyAdapterOptions(options);

  await app.register(fastifyTRPCPlugin, adapterOptions);
  return adapterOptions;
}

function handleCorsPreflight(options = {}) {
  const cors = options.baseline.cors.evaluate(readHeader(options.request, "origin"));
  return sendJson(options.reply, cors.statusCode ?? 204, undefined, {
    ...cors.headers,
    "access-control-allow-headers": "content-type,x-request-id",
    "access-control-allow-methods": "POST,OPTIONS"
  });
}

function readTrpcPayload(body) {
  if (body && typeof body === "object" && !Array.isArray(body) && Object.hasOwn(body, "input")) {
    return body.input ?? {};
  }

  return body ?? {};
}

function readHeader(request, name) {
  const headers = request.headers ?? {};
  const normalizedName = name.toLowerCase();

  for (const [headerName, value] of Object.entries(headers)) {
    if (headerName.toLowerCase() === normalizedName) {
      return value;
    }
  }

  return undefined;
}

function sendJson(reply, statusCode, body, headers = {}) {
  const responder = typeof reply.code === "function" ? reply.code(statusCode) : reply;

  if (typeof responder.status === "function") {
    responder.status(statusCode);
  } else {
    responder.statusCode = statusCode;
  }

  for (const [name, value] of Object.entries(headers)) {
    if (value !== undefined && typeof responder.header === "function") {
      responder.header(name, value);
    }
  }

  return typeof responder.send === "function" ? responder.send(body) : body;
}

function parsePort(value, fallback) {
  if (value == null || value === "") {
    return fallback;
  }

  const port = Number(value);
  if (!Number.isInteger(port) || port < 1 || port > 65_535) {
    throw new Error("PORT must be a valid TCP port");
  }

  return port;
}
