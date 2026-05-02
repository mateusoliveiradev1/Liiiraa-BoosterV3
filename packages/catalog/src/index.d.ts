export type CatalogChannel = "dev" | "beta" | "stable";
export type CatalogMode = "Safe" | "Competitive" | "Lab" | "Blocked";
export type CatalogRisk = "Low" | "Medium" | "High" | "Blocked";
export type CatalogOperationKind = "read" | "write" | "deny" | "recommend" | "backup" | "verify";

export type CatalogPublicKeyJwk = {
  crv: "P-256";
  ext?: boolean;
  kty: "EC";
  x: string;
  y: string;
};

export type CatalogOperation = {
  commandId?: string;
  kind: CatalogOperationKind;
  target: string;
  value?: string;
};

export type CatalogSourceLink = {
  title: string;
  url: string;
};

export type CatalogEntry = {
  defaultEnabled: boolean;
  id: string;
  mode: CatalogMode;
  operations?: CatalogOperation[];
  risk: CatalogRisk;
  sourceLinks: CatalogSourceLink[];
  title: string;
};

export type CatalogPayload = {
  blockedAppVersions?: string[];
  catalogVersion: string;
  channel: CatalogChannel;
  entries: CatalogEntry[];
  minimumAppVersion?: string;
  publishedAtUtc: string;
  revoked: boolean;
  rolloutPercentage: number;
  schemaVersion: string;
};

export type CatalogSignature = {
  algorithm: "ECDSA_P256_SHA256";
  keyId: string;
  publicKeyJwk: CatalogPublicKeyJwk;
  value: string;
};

export type SignedCatalogEnvelope = {
  integrity: string;
  payload: CatalogPayload;
  signature: CatalogSignature;
};

export type CatalogValidationIssue = {
  code: string;
  expected: string;
  path: Array<string | number>;
};

export type CatalogValidationOptions = {
  allowedPrivilegedCommandIds?: string[];
  publicKeyJwk?: CatalogPublicKeyJwk;
};

export class CatalogValidationError extends Error {
  code: "BAD_CATALOG";
  issues: CatalogValidationIssue[];
}

export const CATALOG_SCHEMA_VERSION: "1";
export const CATALOG_SIGNATURE_ALGORITHM: "ECDSA_P256_SHA256";
export const CATALOG_INTEGRITY_PREFIX: "sha256:";
export const DEFAULT_ALLOWED_PRIVILEGED_COMMAND_IDS: readonly string[];

export function canonicalizeCatalogJson(value: unknown): string;
export function deepFreeze<T>(value: T): Readonly<T>;
export function digestCatalogPayload(payload: CatalogPayload): Promise<string>;
export function selectSignedCatalogForChannel(
  catalogs: SignedCatalogEnvelope[],
  channel?: CatalogChannel
): SignedCatalogEnvelope;
export function signCatalogPayload(
  payload: CatalogPayload,
  options: CatalogValidationOptions & {
    keyId?: string;
    privateKeyJwk: Record<string, unknown>;
  }
): Promise<SignedCatalogEnvelope>;
export function validateCatalogPayload(
  payload: unknown,
  options?: CatalogValidationOptions
): CatalogPayload;
export function validateSignedCatalogEnvelopeShape(envelope: unknown): SignedCatalogEnvelope;
export function verifySignedCatalogEnvelope(
  envelope: unknown,
  options?: CatalogValidationOptions
): Promise<{
  integrity: string;
  payload: CatalogPayload;
  signature: {
    algorithm: "ECDSA_P256_SHA256";
    keyId: string;
  };
}>;
