import {
  resolveSignedCatalogForDelivery,
  verifySignedCatalogEnvelope
} from "../../../packages/catalog/src/index.js";
import {
  DEFAULT_CATALOG_PUBLIC_KEY_JWK,
  DEFAULT_SIGNED_TWEAK_CATALOGS
} from "../../../packages/catalog/src/fixture.js";
import { parseBoolean } from "./config.js";

const DEFAULT_CATALOG_CONTROL_AUTHOR = "liiiraa-release-ops";

export async function createCatalogDeliveryResponse(input = {}, options = {}) {
  const channel = input.channel ?? "stable";
  const catalogs = options.catalogs ?? DEFAULT_SIGNED_TWEAK_CATALOGS;
  const controls =
    options.catalogRollbackControls ?? options.catalogControls ?? loadCatalogRollbackControls(options.env);
  const resolution = resolveSignedCatalogForDelivery(
    options.signedCatalog ? [options.signedCatalog] : catalogs,
    {
      channel,
      clientVersion: input.clientVersion
    },
    controls
  );
  const envelope = resolution.envelope;
  const verification = await verifySignedCatalogEnvelope(envelope, {
    allowedPrivilegedCommandIds: options.allowedPrivilegedCommandIds,
    publicKeyJwk: options.publicKeyJwk ?? DEFAULT_CATALOG_PUBLIC_KEY_JWK
  });
  const payload = verification.payload;

  await writeCatalogAuditEvent(options, resolution.auditEvent);

  return {
    catalogVersion: payload.catalogVersion,
    channel: payload.channel,
    integrity: envelope.integrity,
    minimumAppVersion: payload.minimumAppVersion,
    payload,
    publishedAtUtc: payload.publishedAtUtc,
    schemaVersion: payload.schemaVersion,
    signature: envelope.signature
  };
}

export function loadCatalogRollbackControls(env = defaultCatalogControlEnv()) {
  const killSwitchEnabled = parseBoolean(env.CATALOG_KILL_SWITCH_ENABLED, false);
  const rolloutPaused = parseBoolean(env.CATALOG_ROLLOUT_PAUSED, false);

  return {
    audit: {
      author: normalizeOptionalCatalogControlValue(env.CATALOG_CONTROL_AUTHOR) ?? DEFAULT_CATALOG_CONTROL_AUTHOR,
      reason: normalizeOptionalCatalogControlValue(env.CATALOG_CONTROL_REASON),
      riskChange: normalizeOptionalCatalogControlValue(env.CATALOG_CONTROL_RISK_CHANGE),
      sourceReferences: parseCatalogControlList(env.CATALOG_CONTROL_SOURCE_REFERENCES),
      timestampUtc: normalizeOptionalCatalogControlValue(env.CATALOG_CONTROL_TIMESTAMP_UTC)
    },
    disabledCatalogVersions: parseCatalogControlList(env.CATALOG_DISABLED_VERSIONS),
    killSwitch: {
      enabled: killSwitchEnabled,
      reason: normalizeOptionalCatalogControlValue(env.CATALOG_KILL_SWITCH_REASON)
    },
    pausedChannels: parseCatalogControlList(env.CATALOG_PAUSED_CHANNELS),
    rollbackCatalogVersion: normalizeOptionalCatalogControlValue(env.CATALOG_ROLLBACK_VERSION),
    rolloutPaused
  };
}

async function writeCatalogAuditEvent(options, auditEvent) {
  if (Array.isArray(options.catalogAuditLog)) {
    options.catalogAuditLog.push(auditEvent);
  }

  if (typeof options.catalogAuditLogger === "function") {
    await options.catalogAuditLogger(auditEvent);
  }
}

function parseCatalogControlList(value) {
  if (value == null || value === "") {
    return [];
  }

  return String(value)
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizeOptionalCatalogControlValue(value) {
  if (value == null) {
    return undefined;
  }

  const trimmed = String(value).trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function defaultCatalogControlEnv() {
  return typeof process === "undefined" ? {} : process.env;
}
