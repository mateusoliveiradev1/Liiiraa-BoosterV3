import {
  selectSignedCatalogForChannel,
  verifySignedCatalogEnvelope
} from "../../../packages/catalog/src/index.js";
import {
  DEFAULT_CATALOG_PUBLIC_KEY_JWK,
  DEFAULT_SIGNED_TWEAK_CATALOGS
} from "../../../packages/catalog/src/fixture.js";

export async function createCatalogDeliveryResponse(input = {}, options = {}) {
  const channel = input.channel ?? "stable";
  const catalogs = options.catalogs ?? DEFAULT_SIGNED_TWEAK_CATALOGS;
  const envelope = options.signedCatalog ?? selectSignedCatalogForChannel(catalogs, channel);
  const verification = await verifySignedCatalogEnvelope(envelope, {
    allowedPrivilegedCommandIds: options.allowedPrivilegedCommandIds,
    publicKeyJwk: options.publicKeyJwk ?? DEFAULT_CATALOG_PUBLIC_KEY_JWK
  });
  const payload = verification.payload;

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
