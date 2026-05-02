export const API_SECURITY_CONTRACT_VERSION = "0.1.0";

const REQUEST_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$/;
const PROCEDURE_NAME_PATTERN = /^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$/;
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
    inputSchema: objectSchema({
      channel: optionalEnumSchema(["dev", "beta", "stable"]),
      clientVersion: optionalStringSchema({ maxLength: 64 })
    }),
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
    inputSchema: objectSchema({
      consent: benchmarkSyncConsentSchema,
      session: benchmarkSyncSessionSchema
    }),
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
  "system.health": {
    errorRedaction: "public",
    inputSchema: objectSchema({
      includeBuild: optionalBooleanSchema()
    }),
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
