import {
  CATALOG_SCHEMA_VERSION,
  CATALOG_SIGNATURE_ALGORITHM,
  deepFreeze
} from "./index.js";

export const DEFAULT_CATALOG_PUBLIC_KEY_JWK = deepFreeze({
  crv: "P-256",
  ext: true,
  kty: "EC",
  x: "IrTy7Jv2fVylBu7JZhHo2xiwlkDAibrbrpw2_PCT12s",
  y: "asRP9CgCdVFoepEB3ZgpVIgJ3-sItvrJJc5nzD_fNsY"
});

const BASE_ENTRIES = deepFreeze([
  {
    defaultEnabled: true,
    id: "sys.scan.inventory",
    mode: "Safe",
    operations: [
      {
        commandId: "scan.system_inventory",
        kind: "read",
        target: "system.inventory"
      }
    ],
    risk: "Low",
    sourceLinks: [
      {
        title: "V1 tweak matrix",
        url: "local:v1-tweak-matrix"
      }
    ],
    title: "System inventory scan"
  },
  {
    defaultEnabled: false,
    id: "blocked.defender.disable",
    mode: "Blocked",
    operations: [
      {
        commandId: "guardrail.deny",
        kind: "deny",
        target: "blocked.defender.disable"
      }
    ],
    risk: "Blocked",
    sourceLinks: [
      {
        title: "V1 blocked guardrails",
        url: "local:v1-tweak-matrix"
      }
    ],
    title: "Deny global Defender disable"
  }
]);

export const DEFAULT_SIGNED_TWEAK_CATALOGS = deepFreeze([
  createFixtureEnvelope(
    "dev",
    "2026.05.02-dev.1",
    "sha256:44eddec8fe55ea9c228c9846cff2edbc48a544fa300a33074dccbc75634ec27e",
    "z4DVKkOO2LR-KxzDdW2mawhrNAhVsQr6gKYAoLJ89M8loR1e7p8RcUWld8ERqHH-YGDla_rCcj9gCVgpi6ay6A"
  ),
  createFixtureEnvelope(
    "beta",
    "2026.05.02-beta.1",
    "sha256:737705eb9f93f0f2227cdf6839aa0611c1f184edac565b278f71ceebded72d12",
    "q31cW_AwNbb0RkQ0NqFL5ZPPx2HmdyA-54NS_hvBZejZeIDZNY6rBy1HvYAWDsJGS31AMMx9IVaWGnLnb0qoaw"
  ),
  createFixtureEnvelope(
    "stable",
    "2026.05.02-stable.1",
    "sha256:862b907b716a39c589121b54059f2490c290b660cb3a5e5d6652918ea87d7e40",
    "FAbOzY8ituY-bIlwFccs0B8qbE2Lu8s8fQSJD4go72FGgHLM0a4rXnutkHGJTfzDJDtwvLMhTiZ3q0JSZxAatw"
  )
]);

export const DEFAULT_SIGNED_TWEAK_CATALOG = DEFAULT_SIGNED_TWEAK_CATALOGS.find(
  (catalog) => catalog.payload.channel === "stable"
);

function createFixtureEnvelope(channel, catalogVersion, integrity, signatureValue) {
  return {
    integrity,
    payload: {
      blockedAppVersions: [],
      catalogVersion,
      channel,
      entries: BASE_ENTRIES,
      minimumAppVersion: "0.0.0",
      publishedAtUtc: "2026-05-02T00:00:00.000Z",
      revoked: false,
      rolloutPercentage: channel === "dev" ? 25 : 100,
      schemaVersion: CATALOG_SCHEMA_VERSION
    },
    signature: {
      algorithm: CATALOG_SIGNATURE_ALGORITHM,
      keyId: `liiiraa-catalog-${channel}-v1`,
      publicKeyJwk: DEFAULT_CATALOG_PUBLIC_KEY_JWK,
      value: signatureValue
    }
  };
}
