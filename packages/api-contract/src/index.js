export const API_SECURITY_CONTRACT_VERSION = "0.1.0";
export const API_AUTH_BOUNDARY_CONTRACT_VERSION = "0.1.0";
export const API_TRPC_CONTRACT_VERSION = "0.2.0";
export const API_TRPC_PATH = "/trpc";
export const RELEASE_CHANNELS = deepFreeze(["dev", "beta", "stable"]);
export const RELEASE_PLATFORMS = deepFreeze(["windows-x64"]);
export const RESERVED_PRIVATE_AUTH_SCOPES = deepFreeze([
  "account:read",
  "devices:write",
  "licenses:read"
]);

const REQUEST_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$/;
const PROCEDURE_NAME_PATTERN = /^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$/;
const PRIVATE_LOG_FIELD_DENY_LIST = new Set([
  "authorization",
  "cookie",
  "databaseUrl",
  "neonUrl",
  "payload",
  "rawPayload",
  "sessionToken",
  "token"
]);
const emptyInputSchema = objectSchema({});
const releaseChannelSchema = enumSchema(RELEASE_CHANNELS);
const releasePlatformSchema = enumSchema(RELEASE_PLATFORMS);
const catalogLatestInputSchema = objectSchema({
  channel: optionalEnumSchema(RELEASE_CHANNELS),
  clientVersion: optionalStringSchema({ maxLength: 64 })
});
const catalogPayloadSchema = plainObjectSchema();
const catalogSignaturePublicKeySchema = objectSchema({
  crv: requiredStringSchema({ maxLength: 16 }),
  ext: optionalBooleanSchema(),
  kty: requiredStringSchema({ maxLength: 16 }),
  x: requiredStringSchema({ maxLength: 128 }),
  y: requiredStringSchema({ maxLength: 128 })
});
const catalogSignatureSchema = objectSchema({
  algorithm: enumSchema(["ECDSA_P256_SHA256"]),
  keyId: requiredStringSchema({ maxLength: 96 }),
  publicKeyJwk: catalogSignaturePublicKeySchema,
  value: requiredStringSchema({ maxLength: 256 })
});
const catalogLatestOutputSchema = objectSchema({
  catalogVersion: requiredStringSchema({ maxLength: 96 }),
  channel: releaseChannelSchema,
  integrity: requiredStringSchema({ maxLength: 96 }),
  minimumAppVersion: optionalStringSchema({ maxLength: 64 }),
  payload: catalogPayloadSchema,
  publishedAtUtc: requiredStringSchema({ maxLength: 40 }),
  schemaVersion: requiredStringSchema({ maxLength: 16 }),
  signature: catalogSignatureSchema
});
const releaseChannelInfoSchema = objectSchema({
  description: requiredStringSchema({ maxLength: 240 }),
  id: releaseChannelSchema,
  requiresSignedArtifacts: booleanSchema(),
  riskyChangesFirst: booleanSchema(),
  title: requiredStringSchema({ maxLength: 64 })
});
const releaseChannelsOutputSchema = objectSchema({
  channels: arraySchema(releaseChannelInfoSchema, { maxLength: 3, minLength: 3 }),
  defaultChannel: releaseChannelSchema,
  version: requiredStringSchema({ maxLength: 32 })
});
const releaseLatestInputSchema = objectSchema({
  channel: optionalEnumSchema(RELEASE_CHANNELS),
  clientVersion: optionalStringSchema({ maxLength: 64 }),
  platform: optionalEnumSchema(RELEASE_PLATFORMS)
});
const releaseLatestOutputSchema = objectSchema({
  artifactSha256: requiredStringSchema({ maxLength: 64 }),
  artifactUrl: requiredStringSchema({ maxLength: 512 }),
  channel: releaseChannelSchema,
  isCritical: booleanSchema(),
  minimumAppVersion: optionalStringSchema({ maxLength: 64 }),
  platform: releasePlatformSchema,
  publishedAtUtc: requiredStringSchema({ maxLength: 40 }),
  releaseNotesUrl: requiredStringSchema({ maxLength: 512 }),
  rolloutPercent: integerSchema({ max: 100, min: 0 }),
  signature: requiredStringSchema({ maxLength: 512 }),
  updateAvailable: booleanSchema(),
  version: requiredStringSchema({ maxLength: 64 })
});
const systemHealthInputSchema = objectSchema({
  includeBuild: optionalBooleanSchema()
});
const systemHealthOutputSchema = objectSchema({
  build: optionalStringSchema({ maxLength: 96 }),
  ok: booleanSchema(),
  service: requiredStringSchema({ maxLength: 64 }),
  uptimeMs: integerSchema({ min: 0 }),
  version: requiredStringSchema({ maxLength: 32 })
});
const benchmarkSyncConsentSchema = objectSchema({
  benchmarkSync: optionalBooleanSchema(),
  crashReports: optionalBooleanSchema(),
  telemetry: optionalBooleanSchema()
});
const benchmarkSyncCaptureSchema = objectSchema({
  averageFps: numberSchema({ min: 0 }),
  capturedAtUtc: requiredStringSchema({ maxLength: 40 }),
  delayedFrames: integerSchema({ min: 0 }),
  droppedFrames: integerSchema({ min: 0 }),
  frametimeP50Ms: numberSchema({ min: 0 }),
  frametimeP95Ms: numberSchema({ min: 0 }),
  frametimeP99Ms: numberSchema({ min: 0 }),
  generatedFramesDetected: booleanSchema(),
  id: requiredStringSchema({ maxLength: 128 }),
  latencyProxy: booleanSchema(),
  measurementSource: requiredStringSchema({ maxLength: 96 }),
  onePercentLowFps: numberSchema({ min: 0 }),
  phase: enumSchema(["before", "after", "single"]),
  zeroPointOnePercentLowFps: numberSchema({ min: 0 })
});
const benchmarkSyncSessionSchema = objectSchema({
  activeOptimizerProfile: requiredStringSchema({ maxLength: 96 }),
  activePowerPlan: requiredStringSchema({ maxLength: 96 }),
  captures: arraySchema(benchmarkSyncCaptureSchema, { maxLength: 12, minLength: 1 }),
  createdAtUtc: requiredStringSchema({ maxLength: 40 }),
  driverVersion: requiredStringSchema({ maxLength: 64 }),
  game: requiredStringSchema({ maxLength: 64 }),
  id: requiredStringSchema({ maxLength: 128 }),
  sessionLabel: optionalStringSchema({ maxLength: 96 }),
  windowsBuild: requiredStringSchema({ maxLength: 64 })
});
const benchmarkSyncOutputSchema = objectSchema({
  accepted: booleanSchema(),
  requestId: requiredStringSchema({ maxLength: 128 }),
  statusCode: integerSchema({ min: 100 }),
  version: requiredStringSchema({ maxLength: 32 })
});
const benchmarkSyncInputSchema = objectSchema({
  consent: benchmarkSyncConsentSchema,
  session: benchmarkSyncSessionSchema
});
const featureFlagsEvaluateInputSchema = objectSchema({
  channel: optionalEnumSchema(RELEASE_CHANNELS),
  deviceId: optionalStringSchema({ maxLength: 128 }),
  flagKeys: arraySchema(requiredStringSchema({ maxLength: 128 }), { maxLength: 64, minLength: 1 }),
  userId: optionalStringSchema({ maxLength: 128 })
});
const featureFlagEvaluationSchema = objectSchema({
  enabled: booleanSchema(),
  key: requiredStringSchema({ maxLength: 128 }),
  reason: requiredStringSchema({ maxLength: 160 }),
  rolloutPercent: integerSchema({ max: 100, min: 0 }),
  source: enumSchema(["default", "channel", "device", "user"]),
  variant: optionalStringSchema({ maxLength: 96 })
});
const featureFlagsEvaluateOutputSchema = objectSchema({
  channel: releaseChannelSchema,
  evaluations: arraySchema(featureFlagEvaluationSchema, { maxLength: 64, minLength: 1 }),
  version: requiredStringSchema({ maxLength: 32 })
});
const accountProfileInputSchema = objectSchema({});
const accountProfileOutputSchema = objectSchema({
  accountId: requiredStringSchema({ maxLength: 128 }),
  displayName: optionalStringSchema({ maxLength: 96 }),
  licenseStatus: enumSchema(["active", "inactive", "trial", "unknown"])
});
const deviceRegisterInputSchema = objectSchema({
  appVersion: requiredStringSchema({ maxLength: 64 }),
  channel: optionalEnumSchema(RELEASE_CHANNELS),
  deviceId: requiredStringSchema({ maxLength: 128 }),
  installId: requiredStringSchema({ maxLength: 128 })
});
const deviceRegisterOutputSchema = objectSchema({
  deviceId: requiredStringSchema({ maxLength: 128 }),
  linked: booleanSchema(),
  profileId: optionalStringSchema({ maxLength: 128 })
});
const licenseStatusInputSchema = objectSchema({
  deviceId: optionalStringSchema({ maxLength: 128 })
});
const licenseStatusOutputSchema = objectSchema({
  canSyncBenchmarks: booleanSchema(),
  expiresAtUtc: optionalStringSchema({ maxLength: 40 }),
  plan: enumSchema(["free", "pro", "team", "unknown"]),
  status: enumSchema(["active", "inactive", "trial", "unknown"])
});

export class ContractValidationError extends Error {
  constructor(message, issues = []) {
    super(message);
    this.name = "ContractValidationError";
    this.code = "BAD_REQUEST";
    this.issues = issues;
  }
}

export const publicProcedureSecurityPolicies = deepFreeze({
  "catalog.latest": {
    errorRedaction: "public",
    inputSchema: catalogLatestInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "origin", "statusCode", "durationMs", "remoteAddressHash"]
    },
    rateLimit: {
      key: "ip",
      max: 30,
      windowMs: 60_000
    }
  },
  "benchmarks.sync": {
    errorRedaction: "public",
    inputSchema: benchmarkSyncInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "origin", "statusCode", "durationMs"]
    },
    rateLimit: {
      key: "ip",
      max: 15,
      windowMs: 60_000
    }
  },
  "featureflags.evaluate": {
    errorRedaction: "public",
    inputSchema: featureFlagsEvaluateInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "origin", "statusCode", "durationMs", "remoteAddressHash"]
    },
    rateLimit: {
      key: "ip",
      max: 30,
      windowMs: 60_000
    }
  },
  "releases.channels": {
    errorRedaction: "public",
    inputSchema: emptyInputSchema,
    logging: {
      audit: false,
      fields: ["requestId", "procedure", "origin", "statusCode"]
    },
    rateLimit: {
      key: "ip",
      max: 60,
      windowMs: 60_000
    }
  },
  "releases.latest": {
    errorRedaction: "public",
    inputSchema: releaseLatestInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "origin", "statusCode", "durationMs", "remoteAddressHash"]
    },
    rateLimit: {
      key: "ip",
      max: 30,
      windowMs: 60_000
    }
  },
  "system.health": {
    errorRedaction: "public",
    inputSchema: systemHealthInputSchema,
    logging: {
      audit: false,
      fields: ["requestId", "procedure", "origin", "statusCode"]
    },
    rateLimit: {
      key: "ip",
      max: 60,
      windowMs: 60_000
    }
  }
});

export const privateProcedureSecurityPolicies = deepFreeze({
  "account.profile": {
    auth: {
      required: true,
      scopes: ["account:read"],
      strategy: "future-session-or-device-attestation"
    },
    errorRedaction: "private",
    inputSchema: accountProfileInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "statusCode", "durationMs", "principalHash"]
    },
    rateLimit: {
      key: "principal",
      max: 60,
      windowMs: 60_000
    }
  },
  "devices.register": {
    auth: {
      required: true,
      scopes: ["devices:write"],
      strategy: "future-session-or-device-attestation"
    },
    errorRedaction: "private",
    inputSchema: deviceRegisterInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "statusCode", "durationMs", "principalHash"]
    },
    rateLimit: {
      key: "principal",
      max: 20,
      windowMs: 60_000
    }
  },
  "licenses.status": {
    auth: {
      required: true,
      scopes: ["licenses:read"],
      strategy: "future-session-or-device-attestation"
    },
    errorRedaction: "private",
    inputSchema: licenseStatusInputSchema,
    logging: {
      audit: true,
      fields: ["requestId", "procedure", "statusCode", "durationMs", "principalHash"]
    },
    rateLimit: {
      key: "principal",
      max: 60,
      windowMs: 60_000
    }
  }
});

export const apiProcedureContracts = deepFreeze({
  "catalog.latest": {
    description: "Read the latest signed tweak catalog metadata for a release channel.",
    inputSchema: catalogLatestInputSchema,
    kind: "query",
    outputSchema: catalogLatestOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  },
  "featureflags.evaluate": {
    description: "Evaluate public feature flags for a release channel without exposing rule internals.",
    inputSchema: featureFlagsEvaluateInputSchema,
    kind: "query",
    outputSchema: featureFlagsEvaluateOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  },
  "releases.channels": {
    description: "List supported app release channels and their rollout policy.",
    inputSchema: emptyInputSchema,
    kind: "query",
    outputSchema: releaseChannelsOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  },
  "releases.latest": {
    description: "Read latest signed app release metadata for a dev, beta, or stable channel.",
    inputSchema: releaseLatestInputSchema,
    kind: "query",
    outputSchema: releaseLatestOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  },
  "benchmarks.sync": {
    description: "Sync an aggregate benchmark session only after explicit consent.",
    inputSchema: benchmarkSyncInputSchema,
    kind: "mutation",
    outputSchema: benchmarkSyncOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  },
  "system.health": {
    description: "Report API liveness without exposing infrastructure secrets.",
    inputSchema: systemHealthInputSchema,
    kind: "query",
    outputSchema: systemHealthOutputSchema,
    path: API_TRPC_PATH,
    visibility: "public"
  }
});

export const privateApiProcedureContracts = deepFreeze({
  "account.profile": {
    description: "Reserved authenticated account profile contract for a future auth change.",
    inputSchema: accountProfileInputSchema,
    kind: "query",
    outputSchema: accountProfileOutputSchema,
    path: API_TRPC_PATH,
    visibility: "private"
  },
  "devices.register": {
    description: "Reserved authenticated device linking contract for a future auth change.",
    inputSchema: deviceRegisterInputSchema,
    kind: "mutation",
    outputSchema: deviceRegisterOutputSchema,
    path: API_TRPC_PATH,
    visibility: "private"
  },
  "licenses.status": {
    description: "Reserved authenticated entitlement status contract for a future auth change.",
    inputSchema: licenseStatusInputSchema,
    kind: "query",
    outputSchema: licenseStatusOutputSchema,
    path: API_TRPC_PATH,
    visibility: "private"
  }
});

export function validateSecurityEnvelope(envelope, policies = publicProcedureSecurityPolicies) {
  const issues = [];

  if (!isPlainObject(envelope)) {
    throw new ContractValidationError("Request envelope must be an object", [
      issue([], "invalid_type", "object")
    ]);
  }

  const requestId = validateRequestId(envelope.requestId, ["requestId"], issues);
  const procedure = validateProcedureName(envelope.procedure, ["procedure"], issues);
  const policy = procedure ? policies[procedure] : undefined;

  if (procedure && !policy) {
    issues.push(issue(["procedure"], "unknown_procedure", "registered public procedure"));
  }

  let payload;
  if (policy) {
    payload = policy.inputSchema.parse(envelope.payload ?? {}, ["payload"], issues);
  }

  if (issues.length > 0) {
    throw new ContractValidationError("Request failed API contract validation", issues);
  }

  return {
    payload,
    policy,
    procedure,
    requestId
  };
}

export function assertProcedureSecurityCoverage(policies = publicProcedureSecurityPolicies) {
  const issues = [];

  for (const [procedure, policy] of Object.entries(policies)) {
    if (!PROCEDURE_NAME_PATTERN.test(procedure)) {
      issues.push(issue([procedure], "invalid_procedure_name", "dot-separated procedure name"));
    }

    if (!policy.inputSchema || typeof policy.inputSchema.parse !== "function") {
      issues.push(issue([procedure, "inputSchema"], "missing_validation", "input schema parser"));
    }

    if (!policy.rateLimit || !Number.isInteger(policy.rateLimit.max) || policy.rateLimit.max <= 0) {
      issues.push(issue([procedure, "rateLimit"], "missing_rate_limit", "positive limit"));
    }

    if (!["public", "private"].includes(policy.errorRedaction)) {
      issues.push(issue([procedure, "errorRedaction"], "missing_error_redaction", "redaction mode"));
    }

    if (!policy.logging || !Array.isArray(policy.logging.fields) || policy.logging.fields.length === 0) {
      issues.push(issue([procedure, "logging"], "missing_logging_policy", "least-privilege fields"));
    }
  }

  if (issues.length > 0) {
    throw new ContractValidationError("Procedure security coverage failed", issues);
  }

  return true;
}

export function listPublicApiProcedures(contracts = apiProcedureContracts) {
  return Object.entries(contracts)
    .filter(([, contract]) => contract.visibility === "public")
    .map(([procedure]) => procedure)
    .sort();
}

export function listPrivateApiProcedures(contracts = privateApiProcedureContracts) {
  return Object.entries(contracts)
    .filter(([, contract]) => contract.visibility === "private")
    .map(([procedure]) => procedure)
    .sort();
}

export function getApiProcedureContract(procedure, contracts = apiProcedureContracts) {
  const contract = contracts[procedure];

  if (!contract) {
    throw new ContractValidationError("Unknown API procedure contract", [
      issue(["procedure"], "unknown_procedure", "registered API procedure contract")
    ]);
  }

  return contract;
}

export function validateApiContractInput(procedure, payload = {}, contracts = apiProcedureContracts) {
  const contract = getApiProcedureContract(procedure, contracts);
  return parseContractSchema(contract.inputSchema, payload, ["payload"], "input");
}

export function validateApiContractOutput(procedure, output = {}, contracts = apiProcedureContracts) {
  const contract = getApiProcedureContract(procedure, contracts);
  return parseContractSchema(contract.outputSchema, output, ["output"], "output");
}

export function assertTypedApiContractCoverage(
  contracts = apiProcedureContracts,
  policies = publicProcedureSecurityPolicies
) {
  const issues = [];
  const procedureNames = Object.keys(contracts).sort();

  for (const procedure of procedureNames) {
    const contract = contracts[procedure];
    const policy = policies[procedure];

    if (!PROCEDURE_NAME_PATTERN.test(procedure)) {
      issues.push(issue([procedure], "invalid_procedure_name", "dot-separated procedure name"));
    }

    if (!policy) {
      issues.push(issue([procedure], "missing_security_policy", "public procedure security policy"));
    }

    if (!["query", "mutation"].includes(contract.kind)) {
      issues.push(issue([procedure, "kind"], "invalid_kind", "query | mutation"));
    }

    if (contract.path !== API_TRPC_PATH) {
      issues.push(issue([procedure, "path"], "invalid_trpc_path", API_TRPC_PATH));
    }

    if (!contract.inputSchema || typeof contract.inputSchema.parse !== "function") {
      issues.push(issue([procedure, "inputSchema"], "missing_validation", "input schema parser"));
    }

    if (!contract.outputSchema || typeof contract.outputSchema.parse !== "function") {
      issues.push(issue([procedure, "outputSchema"], "missing_validation", "output schema parser"));
    }

    if (policy && policy.inputSchema !== contract.inputSchema) {
      issues.push(issue([procedure, "inputSchema"], "contract_policy_mismatch", "shared input schema"));
    }
  }

  for (const procedure of Object.keys(policies)) {
    if (!contracts[procedure]) {
      issues.push(issue([procedure], "missing_contract", "typed API procedure contract"));
    }
  }

  if (issues.length > 0) {
    throw new ContractValidationError("Typed API contract coverage failed", issues);
  }

  return true;
}

export function assertAuthReadyBoundaryCoverage(
  publicContracts = apiProcedureContracts,
  privateContracts = privateApiProcedureContracts,
  publicPolicies = publicProcedureSecurityPolicies,
  privatePolicies = privateProcedureSecurityPolicies
) {
  const issues = [];
  const publicNames = new Set(Object.keys(publicContracts));

  for (const [procedure, contract] of Object.entries(publicContracts)) {
    if (contract.visibility !== "public") {
      issues.push(issue([procedure, "visibility"], "invalid_visibility", "public"));
    }
  }

  for (const [procedure, contract] of Object.entries(privateContracts)) {
    const policy = privatePolicies[procedure];

    if (publicNames.has(procedure) || publicPolicies[procedure]) {
      issues.push(issue([procedure], "private_procedure_exposed", "private-only contract"));
    }

    if (!PROCEDURE_NAME_PATTERN.test(procedure)) {
      issues.push(issue([procedure], "invalid_procedure_name", "dot-separated procedure name"));
    }

    if (contract.visibility !== "private") {
      issues.push(issue([procedure, "visibility"], "invalid_visibility", "private"));
    }

    if (!["query", "mutation"].includes(contract.kind)) {
      issues.push(issue([procedure, "kind"], "invalid_kind", "query | mutation"));
    }

    if (contract.path !== API_TRPC_PATH) {
      issues.push(issue([procedure, "path"], "invalid_trpc_path", API_TRPC_PATH));
    }

    if (!contract.inputSchema || typeof contract.inputSchema.parse !== "function") {
      issues.push(issue([procedure, "inputSchema"], "missing_validation", "input schema parser"));
    }

    if (!contract.outputSchema || typeof contract.outputSchema.parse !== "function") {
      issues.push(issue([procedure, "outputSchema"], "missing_validation", "output schema parser"));
    }

    if (!policy) {
      issues.push(issue([procedure], "missing_private_policy", "private procedure security policy"));
      continue;
    }

    if (policy.inputSchema !== contract.inputSchema) {
      issues.push(issue([procedure, "inputSchema"], "contract_policy_mismatch", "shared input schema"));
    }

    if (policy.errorRedaction !== "private") {
      issues.push(issue([procedure, "errorRedaction"], "invalid_error_redaction", "private"));
    }

    if (policy.auth?.required !== true) {
      issues.push(issue([procedure, "auth"], "missing_auth_requirement", "required future auth gate"));
    }

    if (!Array.isArray(policy.auth?.scopes) || policy.auth.scopes.length === 0) {
      issues.push(issue([procedure, "auth", "scopes"], "missing_auth_scope", "at least one private scope"));
    }

    if (!policy.rateLimit || policy.rateLimit.key !== "principal") {
      issues.push(issue([procedure, "rateLimit"], "invalid_private_rate_limit", "principal-scoped limit"));
    }

    if (!policy.logging || !Array.isArray(policy.logging.fields) || policy.logging.fields.length === 0) {
      issues.push(issue([procedure, "logging"], "missing_logging_policy", "least-privilege fields"));
    } else {
      for (const field of policy.logging.fields) {
        if (PRIVATE_LOG_FIELD_DENY_LIST.has(field)) {
          issues.push(issue([procedure, "logging", field], "unsafe_private_log_field", "redacted metadata only"));
        }
      }
    }
  }

  for (const procedure of Object.keys(privatePolicies)) {
    if (!privateContracts[procedure]) {
      issues.push(issue([procedure], "missing_private_contract", "private API procedure contract"));
    }
  }

  if (issues.length > 0) {
    throw new ContractValidationError("Auth-ready API boundary coverage failed", issues);
  }

  return true;
}

function parseContractSchema(schema, value, path, direction) {
  const issues = [];
  const parsed = schema.parse(value, path, issues);

  if (issues.length > 0) {
    throw new ContractValidationError(`API contract ${direction} validation failed`, issues);
  }

  return parsed;
}

export function objectSchema(shape) {
  return {
    parse(value, path = [], issues = []) {
      if (!isPlainObject(value)) {
        issues.push(issue(path, "invalid_type", "object"));
        return undefined;
      }

      const parsed = {};
      for (const [key, schema] of Object.entries(shape)) {
        const fieldValue = schema.parse(value[key], [...path, key], issues);
        if (fieldValue !== undefined) {
          parsed[key] = fieldValue;
        }
      }

      for (const key of Object.keys(value)) {
        if (!(key in shape)) {
          issues.push(issue([...path, key], "unknown_key", "known API contract field"));
        }
      }

      return parsed;
    }
  };
}

export function plainObjectSchema() {
  return {
    parse(value, path = [], issues = []) {
      if (!isPlainObject(value)) {
        issues.push(issue(path, "invalid_type", "object"));
        return undefined;
      }

      return value;
    }
  };
}

export function optionalBooleanSchema() {
  return {
    parse(value, path = [], issues = []) {
      if (value === undefined) {
        return undefined;
      }

      if (typeof value !== "boolean") {
        issues.push(issue(path, "invalid_type", "boolean"));
        return undefined;
      }

      return value;
    }
  };
}

export function booleanSchema() {
  return {
    parse(value, path = [], issues = []) {
      if (typeof value !== "boolean") {
        issues.push(issue(path, "invalid_type", "boolean"));
        return undefined;
      }

      return value;
    }
  };
}

export function enumSchema(values) {
  const allowed = new Set(values);

  return {
    parse(value, path = [], issues = []) {
      if (!allowed.has(value)) {
        issues.push(issue(path, "invalid_enum", values.join(" | ")));
        return undefined;
      }

      return value;
    }
  };
}

export function optionalEnumSchema(values) {
  const allowed = new Set(values);

  return {
    parse(value, path = [], issues = []) {
      if (value === undefined) {
        return undefined;
      }

      if (!allowed.has(value)) {
        issues.push(issue(path, "invalid_enum", values.join(" | ")));
        return undefined;
      }

      return value;
    }
  };
}

export function requiredStringSchema(options = {}) {
  const maxLength = options.maxLength ?? 256;

  return {
    parse(value, path = [], issues = []) {
      if (typeof value !== "string") {
        issues.push(issue(path, "invalid_type", "string"));
        return undefined;
      }

      const trimmed = value.trim();
      if (trimmed.length === 0 || trimmed.length > maxLength) {
        issues.push(issue(path, "invalid_length", `1-${maxLength} characters`));
        return undefined;
      }

      return trimmed;
    }
  };
}

export function optionalStringSchema(options = {}) {
  const maxLength = options.maxLength ?? 256;

  return {
    parse(value, path = [], issues = []) {
      if (value === undefined) {
        return undefined;
      }

      if (typeof value !== "string") {
        issues.push(issue(path, "invalid_type", "string"));
        return undefined;
      }

      const trimmed = value.trim();
      if (trimmed.length === 0 || trimmed.length > maxLength) {
        issues.push(issue(path, "invalid_length", `1-${maxLength} characters`));
        return undefined;
      }

      return trimmed;
    }
  };
}

export function numberSchema(options = {}) {
  return {
    parse(value, path = [], issues = []) {
      if (typeof value !== "number" || !Number.isFinite(value)) {
        issues.push(issue(path, "invalid_type", "finite number"));
        return undefined;
      }

      if (options.min !== undefined && value < options.min) {
        issues.push(issue(path, "invalid_range", `>= ${options.min}`));
        return undefined;
      }

      if (options.max !== undefined && value > options.max) {
        issues.push(issue(path, "invalid_range", `<= ${options.max}`));
        return undefined;
      }

      return value;
    }
  };
}

export function integerSchema(options = {}) {
  return {
    parse(value, path = [], issues = []) {
      if (!Number.isInteger(value)) {
        issues.push(issue(path, "invalid_type", "integer"));
        return undefined;
      }

      if (options.min !== undefined && value < options.min) {
        issues.push(issue(path, "invalid_range", `>= ${options.min}`));
        return undefined;
      }

      if (options.max !== undefined && value > options.max) {
        issues.push(issue(path, "invalid_range", `<= ${options.max}`));
        return undefined;
      }

      return value;
    }
  };
}

export function arraySchema(elementSchema, options = {}) {
  const minLength = options.minLength ?? 0;
  const maxLength = options.maxLength ?? Number.POSITIVE_INFINITY;

  return {
    parse(value, path = [], issues = []) {
      if (!Array.isArray(value)) {
        issues.push(issue(path, "invalid_type", "array"));
        return undefined;
      }

      if (value.length < minLength || value.length > maxLength) {
        issues.push(issue(path, "invalid_length", `${minLength}-${maxLength} items`));
        return undefined;
      }

      return value.map((item, index) => elementSchema.parse(item, [...path, index], issues));
    }
  };
}

function validateRequestId(value, path, issues) {
  if (typeof value !== "string" || !REQUEST_ID_PATTERN.test(value)) {
    issues.push(issue(path, "invalid_request_id", "8-128 safe request id characters"));
    return undefined;
  }

  return value;
}

function validateProcedureName(value, path, issues) {
  if (typeof value !== "string" || !PROCEDURE_NAME_PATTERN.test(value)) {
    issues.push(issue(path, "invalid_procedure", "dot-separated procedure name"));
    return undefined;
  }

  return value;
}

function issue(path, code, expected) {
  return {
    code,
    expected,
    path
  };
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function deepFreeze(value) {
  if (!value || typeof value !== "object" || Object.isFrozen(value)) {
    return value;
  }

  Object.freeze(value);
  for (const child of Object.values(value)) {
    deepFreeze(child);
  }

  return value;
}
