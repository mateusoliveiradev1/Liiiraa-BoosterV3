import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  CatalogValidationError,
  canonicalizeCatalogJson,
  digestCatalogPayload,
  signCatalogPayload,
  validateCatalogPayload,
  verifySignedCatalogEnvelope
} from "../src/index.js";
import {
  DEFAULT_CATALOG_PUBLIC_KEY_JWK,
  DEFAULT_SIGNED_TWEAK_CATALOG,
  DEFAULT_SIGNED_TWEAK_CATALOGS
} from "../src/fixture.js";

describe("signed remote tweak catalogs", () => {
  it("verifies the default signed channel catalogs with pinned public key and integrity", async () => {
    for (const catalog of DEFAULT_SIGNED_TWEAK_CATALOGS) {
      const verified = await verifySignedCatalogEnvelope(catalog, {
        publicKeyJwk: DEFAULT_CATALOG_PUBLIC_KEY_JWK
      });

      assert.equal(verified.integrity, await digestCatalogPayload(catalog.payload));
      assert.equal(verified.payload.schemaVersion, "1");
      assert.equal(verified.payload.entries.some((entry) => entry.id === "sys.scan.inventory"), true);
    }
  });

  it("rejects payload tampering before signature trust is considered", async () => {
    const tampered = {
      ...DEFAULT_SIGNED_TWEAK_CATALOG,
      payload: {
        ...DEFAULT_SIGNED_TWEAK_CATALOG.payload,
        rolloutPercentage: 50
      }
    };

    await assert.rejects(
      () =>
        verifySignedCatalogEnvelope(tampered, {
          publicKeyJwk: DEFAULT_CATALOG_PUBLIC_KEY_JWK
        }),
      (error) =>
        error instanceof CatalogValidationError &&
        error.issues.some((issue) => issue.code === "integrity_mismatch")
    );
  });

  it("rejects invalid signatures even when payload integrity matches", async () => {
    const invalidSignature = {
      ...DEFAULT_SIGNED_TWEAK_CATALOG,
      signature: {
        ...DEFAULT_SIGNED_TWEAK_CATALOG.signature,
        value: DEFAULT_SIGNED_TWEAK_CATALOG.signature.value.replace(/.$/u, "A")
      }
    };

    await assert.rejects(
      () =>
        verifySignedCatalogEnvelope(invalidSignature, {
          publicKeyJwk: DEFAULT_CATALOG_PUBLIC_KEY_JWK
        }),
      (error) =>
        error instanceof CatalogValidationError &&
        error.issues.some((issue) => issue.code === "invalid_signature")
    );
  });

  it("rejects revoked catalogs and arbitrary script-like operations", () => {
    assert.throws(
      () =>
        validateCatalogPayload({
          ...DEFAULT_SIGNED_TWEAK_CATALOG.payload,
          revoked: true
        }),
      (error) =>
        error instanceof CatalogValidationError &&
        error.issues.some((issue) => issue.code === "revoked_catalog")
    );

    assert.throws(
      () =>
        validateCatalogPayload({
          ...DEFAULT_SIGNED_TWEAK_CATALOG.payload,
          entries: [
            {
              ...DEFAULT_SIGNED_TWEAK_CATALOG.payload.entries[0],
              operations: [
                {
                  kind: "write",
                  target: "shell:powershell",
                  value: "powershell -NoProfile -EncodedCommand AAA"
                }
              ]
            }
          ]
        }),
      (error) =>
        error instanceof CatalogValidationError &&
        error.issues.some((issue) => issue.code === "arbitrary_script_content")
    );
  });

  it("rejects catalog operations that reference commands absent from the signed app", () => {
    assert.throws(
      () =>
        validateCatalogPayload({
          ...DEFAULT_SIGNED_TWEAK_CATALOG.payload,
          entries: [
            {
              ...DEFAULT_SIGNED_TWEAK_CATALOG.payload.entries[0],
              operations: [
                {
                  commandId: "agent.new_privileged_command",
                  kind: "write",
                  target: "system.inventory"
                }
              ]
            }
          ]
        }),
      (error) =>
        error instanceof CatalogValidationError &&
        error.issues.some((issue) => issue.code === "unknown_privileged_command")
    );
  });

  it("can sign and verify a generated test catalog without storing a private key", async () => {
    const keyPair = await globalThis.crypto.subtle.generateKey(
      {
        name: "ECDSA",
        namedCurve: "P-256"
      },
      true,
      ["sign", "verify"]
    );
    const privateKeyJwk = await globalThis.crypto.subtle.exportKey("jwk", keyPair.privateKey);
    const publicKeyJwk = await globalThis.crypto.subtle.exportKey("jwk", keyPair.publicKey);
    const payload = {
      ...DEFAULT_SIGNED_TWEAK_CATALOG.payload,
      catalogVersion: "test-catalog.1",
      channel: "dev"
    };

    const signed = await signCatalogPayload(payload, {
      keyId: "test-catalog-key",
      privateKeyJwk,
      publicKeyJwk
    });
    const verified = await verifySignedCatalogEnvelope(signed, { publicKeyJwk });

    assert.equal(verified.payload.catalogVersion, "test-catalog.1");
    assert.equal(signed.integrity, await digestCatalogPayload(payload));
    assert.equal(canonicalizeCatalogJson(signed.payload), canonicalizeCatalogJson(payload));
  });
});
