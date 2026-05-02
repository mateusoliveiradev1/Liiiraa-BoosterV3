export const CATALOG_SCHEMA_VERSION = "1";
export const CATALOG_SIGNATURE_ALGORITHM = "ECDSA_P256_SHA256";
export const CATALOG_INTEGRITY_PREFIX = "sha256:";

export const DEFAULT_ALLOWED_PRIVILEGED_COMMAND_IDS = Object.freeze([
  "catalog.metadata.apply",
  "guardrail.deny",
  "scan.system_inventory"
]);

const CHANNELS = new Set(["dev", "beta", "stable"]);
const MODES = new Set(["Safe", "Competitive", "Lab", "Blocked"]);
const RISKS = new Set(["Low", "Medium", "High", "Blocked"]);
const OPERATION_KINDS = new Set(["read", "write", "deny", "recommend", "backup", "verify"]);
const SAFE_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/;
const TWEAK_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]{0,95}$/;
const SHA256_REF_PATTERN = /^sha256:[a-f0-9]{64}$/;
const BASE64URL_PATTERN = /^[A-Za-z0-9_-]+$/;

export class CatalogValidationError extends Error {
  constructor(message, issues = []) {
    super(message);
    this.name = "CatalogValidationError";
    this.code = "BAD_CATALOG";
    this.issues = issues;
  }
}

export function canonicalizeCatalogJson(value) {
  return JSON.stringify(toCanonicalJson(value));
}

export async function digestCatalogPayload(payload) {
  return `${CATALOG_INTEGRITY_PREFIX}${await sha256Hex(canonicalizeCatalogJson(payload))}`;
}

export async function signCatalogPayload(payload, options = {}) {
  validateCatalogPayload(payload, options);

  if (!options.privateKeyJwk) {
    throw new CatalogValidationError("Catalog signing requires a private key JWK.", [
      issue(["privateKeyJwk"], "missing_private_key", "ECDSA P-256 private JWK")
    ]);
  }

  const canonicalPayload = canonicalizeCatalogJson(payload);
  const privateKey = await importEcdsaKey(options.privateKeyJwk, "sign");
  const signature = await cryptoSubtle().sign(
    {
      name: "ECDSA",
      hash: "SHA-256"
    },
    privateKey,
    encodeUtf8(canonicalPayload)
  );

  return {
    integrity: await digestCatalogPayload(payload),
    payload,
    signature: {
      algorithm: CATALOG_SIGNATURE_ALGORITHM,
      keyId: readSafeString(options.keyId ?? "catalog-signing-key", ["keyId"], {
        maxLength: 96,
        pattern: SAFE_ID_PATTERN
      }),
      publicKeyJwk: normalizePublicJwk(options.publicKeyJwk ?? options.privateKeyJwk),
      value: bytesToBase64Url(new Uint8Array(signature))
    }
  };
}

export async function verifySignedCatalogEnvelope(envelope, options = {}) {
  const parsed = validateSignedCatalogEnvelopeShape(envelope);
  const expectedIntegrity = await digestCatalogPayload(parsed.payload);

  if (parsed.integrity !== expectedIntegrity) {
    throw new CatalogValidationError("Catalog integrity check failed.", [
      issue(["integrity"], "integrity_mismatch", expectedIntegrity)
    ]);
  }

  validateCatalogPayload(parsed.payload, options);

  const publicKeyJwk = options.publicKeyJwk ?? parsed.signature.publicKeyJwk;
  const signatureValid = await verifyCatalogSignature({
    canonicalPayload: canonicalizeCatalogJson(parsed.payload),
    publicKeyJwk,
    signatureValue: parsed.signature.value
  });

  if (!signatureValid) {
    throw new CatalogValidationError("Catalog signature verification failed.", [
      issue(["signature", "value"], "invalid_signature", "valid ECDSA P-256 signature")
    ]);
  }

  return {
    integrity: parsed.integrity,
    payload: parsed.payload,
    signature: {
      algorithm: parsed.signature.algorithm,
      keyId: parsed.signature.keyId
    }
  };
}

export function validateCatalogPayload(payload, options = {}) {
  const issues = [];
  const allowedPrivilegedCommandIds = new Set(
    options.allowedPrivilegedCommandIds ?? DEFAULT_ALLOWED_PRIVILEGED_COMMAND_IDS
  );

  if (!isPlainObject(payload)) {
    throw new CatalogValidationError("Catalog payload must be an object.", [
      issue([], "invalid_type", "object")
    ]);
  }

  readExactString(payload.schemaVersion, ["schemaVersion"], CATALOG_SCHEMA_VERSION, issues);
  readSafeString(payload.catalogVersion, ["catalogVersion"], {
    issues,
    maxLength: 96,
    pattern: SAFE_ID_PATTERN
  });
  readEnum(payload.channel, ["channel"], CHANNELS, issues);
  readIsoTimestamp(payload.publishedAtUtc, ["publishedAtUtc"], issues);
  readOptionalString(payload.minimumAppVersion, ["minimumAppVersion"], issues, { maxLength: 64 });
  readBoolean(payload.revoked, ["revoked"], issues);
  readInteger(payload.rolloutPercentage, ["rolloutPercentage"], issues, { max: 100, min: 0 });
  readStringArray(payload.blockedAppVersions ?? [], ["blockedAppVersions"], issues, {
    maxItems: 128,
    maxLength: 64
  });

  if (payload.revoked === true) {
    issues.push(issue(["revoked"], "revoked_catalog", "active catalog"));
  }

  if (!Array.isArray(payload.entries) || payload.entries.length === 0) {
    issues.push(issue(["entries"], "invalid_length", "1 or more catalog entries"));
  } else if (payload.entries.length > 512) {
    issues.push(issue(["entries"], "invalid_length", "512 or fewer catalog entries"));
  } else {
    const seenTweakIds = new Set();

    payload.entries.forEach((entry, index) => {
      validateCatalogEntry(entry, ["entries", index], issues, allowedPrivilegedCommandIds);

      if (isPlainObject(entry) && typeof entry.id === "string") {
        if (seenTweakIds.has(entry.id)) {
          issues.push(issue(["entries", index, "id"], "duplicate_tweak_id", "unique tweak id"));
        }

        seenTweakIds.add(entry.id);
      }
    });
  }

  rejectArbitraryScriptContent(payload, [], issues);

  if (issues.length > 0) {
    throw new CatalogValidationError("Catalog payload validation failed.", issues);
  }

  return payload;
}

export function validateSignedCatalogEnvelopeShape(envelope) {
  if (!isPlainObject(envelope)) {
    throw new CatalogValidationError("Catalog envelope must be an object.", [
      issue([], "invalid_type", "object")
    ]);
  }

  const issues = [];
  const payload = readPlainObject(envelope.payload, ["payload"], issues);
  const integrity = readSafeString(envelope.integrity, ["integrity"], {
    issues,
    maxLength: 96,
    pattern: SHA256_REF_PATTERN
  });
  const signature = readPlainObject(envelope.signature, ["signature"], issues);

  let parsedSignature;
  if (signature) {
    parsedSignature = {
      algorithm: readExactString(
        signature.algorithm,
        ["signature", "algorithm"],
        CATALOG_SIGNATURE_ALGORITHM,
        issues
      ),
      keyId: readSafeString(signature.keyId, ["signature", "keyId"], {
        issues,
        maxLength: 96,
        pattern: SAFE_ID_PATTERN
      }),
      publicKeyJwk: validatePublicJwk(signature.publicKeyJwk, ["signature", "publicKeyJwk"], issues),
      value: readSafeString(signature.value, ["signature", "value"], {
        issues,
        maxLength: 256,
        pattern: BASE64URL_PATTERN
      })
    };
  }

  if (issues.length > 0) {
    throw new CatalogValidationError("Catalog envelope validation failed.", issues);
  }

  return {
    integrity,
    payload,
    signature: parsedSignature
  };
}

export function selectSignedCatalogForChannel(catalogs, channel = "stable") {
  const requestedChannel = CHANNELS.has(channel) ? channel : "stable";
  const catalog = catalogs.find((candidate) => candidate?.payload?.channel === requestedChannel);

  if (!catalog) {
    throw new CatalogValidationError("No signed catalog exists for requested channel.", [
      issue(["channel"], "unknown_channel_catalog", requestedChannel)
    ]);
  }

  return catalog;
}

export function deepFreeze(value) {
  if (!value || typeof value !== "object" || Object.isFrozen(value)) {
    return value;
  }

  Object.freeze(value);
  for (const child of Object.values(value)) {
    deepFreeze(child);
  }

  return value;
}

async function verifyCatalogSignature({ canonicalPayload, publicKeyJwk, signatureValue }) {
  const publicKey = await importEcdsaKey(publicKeyJwk, "verify");
  return cryptoSubtle().verify(
    {
      name: "ECDSA",
      hash: "SHA-256"
    },
    publicKey,
    base64UrlToBytes(signatureValue),
    encodeUtf8(canonicalPayload)
  );
}

async function importEcdsaKey(jwk, usage) {
  const normalized = usage === "verify" ? validatePublicJwk(jwk, ["publicKeyJwk"], []) : jwk;
  return cryptoSubtle().importKey(
    "jwk",
    normalized,
    {
      name: "ECDSA",
      namedCurve: "P-256"
    },
    false,
    [usage]
  );
}

function validateCatalogEntry(entry, path, issues, allowedPrivilegedCommandIds) {
  if (!isPlainObject(entry)) {
    issues.push(issue(path, "invalid_type", "object"));
    return;
  }

  readSafeString(entry.id, [...path, "id"], {
    issues,
    maxLength: 96,
    pattern: TWEAK_ID_PATTERN
  });
  readSafeString(entry.title, [...path, "title"], { issues, maxLength: 120 });
  readEnum(entry.mode, [...path, "mode"], MODES, issues);
  readEnum(entry.risk, [...path, "risk"], RISKS, issues);
  readBoolean(entry.defaultEnabled, [...path, "defaultEnabled"], issues);

  if (entry.mode === "Blocked" && entry.defaultEnabled === true) {
    issues.push(issue([...path, "defaultEnabled"], "blocked_default_enabled", "false for blocked entries"));
  }

  validateSourceLinks(entry.sourceLinks, [...path, "sourceLinks"], issues);
  validateOperations(entry.operations ?? [], [...path, "operations"], issues, allowedPrivilegedCommandIds);
}

function validateSourceLinks(sourceLinks, path, issues) {
  if (!Array.isArray(sourceLinks) || sourceLinks.length === 0) {
    issues.push(issue(path, "invalid_length", "1 or more source links"));
    return;
  }

  if (sourceLinks.length > 16) {
    issues.push(issue(path, "invalid_length", "16 or fewer source links"));
    return;
  }

  sourceLinks.forEach((sourceLink, index) => {
    const linkPath = [...path, index];
    if (!isPlainObject(sourceLink)) {
      issues.push(issue(linkPath, "invalid_type", "object"));
      return;
    }

    readSafeString(sourceLink.title, [...linkPath, "title"], { issues, maxLength: 120 });

    if (
      typeof sourceLink.url !== "string" ||
      !(sourceLink.url.startsWith("https://") || sourceLink.url.startsWith("local:"))
    ) {
      issues.push(issue([...linkPath, "url"], "invalid_url", "https:// or local: source link"));
    }
  });
}

function validateOperations(operations, path, issues, allowedPrivilegedCommandIds) {
  if (!Array.isArray(operations)) {
    issues.push(issue(path, "invalid_type", "array"));
    return;
  }

  if (operations.length > 32) {
    issues.push(issue(path, "invalid_length", "32 or fewer operations"));
    return;
  }

  operations.forEach((operation, index) => {
    const operationPath = [...path, index];
    if (!isPlainObject(operation)) {
      issues.push(issue(operationPath, "invalid_type", "object"));
      return;
    }

    readEnum(operation.kind, [...operationPath, "kind"], OPERATION_KINDS, issues);
    readSafeString(operation.target, [...operationPath, "target"], { issues, maxLength: 160 });

    if (operation.value !== undefined) {
      readSafeString(operation.value, [...operationPath, "value"], { issues, maxLength: 256 });
    }

    if (operation.commandId !== undefined) {
      readSafeString(operation.commandId, [...operationPath, "commandId"], {
        issues,
        maxLength: 96,
        pattern: SAFE_ID_PATTERN
      });

      if (!allowedPrivilegedCommandIds.has(operation.commandId)) {
        issues.push(issue([...operationPath, "commandId"], "unknown_privileged_command", "signed app command id"));
      }
    }
  });
}

function rejectArbitraryScriptContent(value, path, issues) {
  if (typeof value === "string") {
    if (looksLikeArbitraryScript(value)) {
      issues.push(issue(path, "arbitrary_script_content", "typed catalog data only"));
    }
    return;
  }

  if (Array.isArray(value)) {
    value.forEach((item, index) => rejectArbitraryScriptContent(item, [...path, index], issues));
    return;
  }

  if (isPlainObject(value)) {
    Object.entries(value).forEach(([key, child]) =>
      rejectArbitraryScriptContent(child, [...path, key], issues)
    );
  }
}

function looksLikeArbitraryScript(value) {
  const normalized = value.trim().toLowerCase();

  return (
    normalized.startsWith("script:") ||
    normalized.startsWith("shell:") ||
    normalized.startsWith("exec:") ||
    normalized.includes("powershell") ||
    normalized.includes("cmd.exe") ||
    normalized.includes("bash -") ||
    normalized.includes(".ps1") ||
    (normalized.includes("curl ") && normalized.includes("|"))
  );
}

function readPlainObject(value, path, issues) {
  if (!isPlainObject(value)) {
    issues.push(issue(path, "invalid_type", "object"));
    return undefined;
  }

  return value;
}

function readExactString(value, path, expected, issues = []) {
  if (value !== expected) {
    issues.push(issue(path, "invalid_value", expected));
    return undefined;
  }

  return value;
}

function readEnum(value, path, values, issues) {
  if (!values.has(value)) {
    issues.push(issue(path, "invalid_enum", [...values].join(" | ")));
    return undefined;
  }

  return value;
}

function readSafeString(value, path, options = {}) {
  const issues = options.issues ?? [];

  if (typeof value !== "string") {
    issues.push(issue(path, "invalid_type", "string"));
    return undefined;
  }

  const trimmed = value.trim();
  const maxLength = options.maxLength ?? 256;
  if (trimmed.length === 0 || trimmed.length > maxLength) {
    issues.push(issue(path, "invalid_length", `1-${maxLength} characters`));
    return undefined;
  }

  if (options.pattern && !options.pattern.test(trimmed)) {
    issues.push(issue(path, "invalid_format", "safe identifier"));
    return undefined;
  }

  return trimmed;
}

function readOptionalString(value, path, issues, options = {}) {
  if (value === undefined) {
    return undefined;
  }

  return readSafeString(value, path, { ...options, issues });
}

function readBoolean(value, path, issues) {
  if (typeof value !== "boolean") {
    issues.push(issue(path, "invalid_type", "boolean"));
    return undefined;
  }

  return value;
}

function readInteger(value, path, issues, options = {}) {
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

function readIsoTimestamp(value, path, issues) {
  const timestamp = readSafeString(value, path, { issues, maxLength: 40 });

  if (timestamp && Number.isNaN(Date.parse(timestamp))) {
    issues.push(issue(path, "invalid_timestamp", "ISO-8601 timestamp"));
  }

  return timestamp;
}

function readStringArray(value, path, issues, options = {}) {
  if (!Array.isArray(value)) {
    issues.push(issue(path, "invalid_type", "array"));
    return undefined;
  }

  if (value.length > (options.maxItems ?? 128)) {
    issues.push(issue(path, "invalid_length", `${options.maxItems ?? 128} or fewer items`));
    return undefined;
  }

  value.forEach((item, index) =>
    readSafeString(item, [...path, index], {
      issues,
      maxLength: options.maxLength ?? 128
    })
  );

  return value;
}

function validatePublicJwk(value, path, issues) {
  if (!isPlainObject(value)) {
    issues.push(issue(path, "invalid_type", "public JWK"));
    return undefined;
  }

  const jwk = normalizePublicJwk(value);

  if (jwk.kty !== "EC" || jwk.crv !== "P-256" || !jwk.x || !jwk.y) {
    issues.push(issue(path, "invalid_public_key", "P-256 public JWK"));
    return undefined;
  }

  return jwk;
}

function normalizePublicJwk(jwk) {
  return {
    crv: jwk.crv,
    ext: true,
    kty: jwk.kty,
    x: jwk.x,
    y: jwk.y
  };
}

function toCanonicalJson(value, path = []) {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return value;
  }

  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new CatalogValidationError("Catalog JSON contains a non-finite number.", [
        issue(path, "invalid_number", "finite number")
      ]);
    }

    return value;
  }

  if (Array.isArray(value)) {
    return value.map((item, index) => toCanonicalJson(item, [...path, index]));
  }

  if (isPlainObject(value)) {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => {
          if (value[key] === undefined) {
            throw new CatalogValidationError("Catalog JSON cannot contain undefined values.", [
              issue([...path, key], "invalid_type", "JSON value")
            ]);
          }

          return [key, toCanonicalJson(value[key], [...path, key])];
        })
    );
  }

  throw new CatalogValidationError("Catalog JSON contains an unsupported value.", [
    issue(path, "invalid_type", "JSON value")
  ]);
}

async function sha256Hex(value) {
  const digest = await cryptoSubtle().digest("SHA-256", encodeUtf8(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function encodeUtf8(value) {
  return new TextEncoder().encode(value);
}

function bytesToBase64Url(bytes) {
  const binary = String.fromCharCode(...bytes);
  const base64 =
    typeof btoa === "function" ? btoa(binary) : Buffer.from(binary, "binary").toString("base64");

  return base64.replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/u, "");
}

function base64UrlToBytes(value) {
  const base64 = value.replaceAll("-", "+").replaceAll("_", "/");
  const padded = base64.padEnd(Math.ceil(base64.length / 4) * 4, "=");
  const binary =
    typeof atob === "function" ? atob(padded) : Buffer.from(padded, "base64").toString("binary");

  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function cryptoSubtle() {
  if (!globalThis.crypto?.subtle) {
    throw new CatalogValidationError("WebCrypto SubtleCrypto is required for catalog trust checks.", [
      issue(["crypto"], "missing_subtle_crypto", "globalThis.crypto.subtle")
    ]);
  }

  return globalThis.crypto.subtle;
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
