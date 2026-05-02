import {
  verifySignedCatalogEnvelope,
  type CatalogChannel,
  type CatalogPublicKeyJwk
} from "../../../packages/catalog/src/index.js";
import { DEFAULT_CATALOG_PUBLIC_KEY_JWK } from "../../../packages/catalog/src/fixture.js";

type StorageLike = {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
};

export type RemoteCatalogFetchResult = {
  catalog: unknown;
  fromCache: boolean;
  integrity: string;
  signatureKeyId: string;
};

export type RemoteCatalogFetcherOptions = {
  allowedPrivilegedCommandIds?: string[];
  channel?: CatalogChannel;
  clientVersion?: string;
  endpoint?: string;
  fetchImpl?: typeof fetch;
  publicKeyJwk?: CatalogPublicKeyJwk;
  storage?: StorageLike;
};

const DEFAULT_API_ENDPOINT = "https://api.liiiraa.example";
const CATALOG_CACHE_KEY = "liiiraa:last-known-good-catalog";

export function createRemoteCatalogFetcher(options: RemoteCatalogFetcherOptions = {}) {
  return {
    fetchLatestCatalog: () => fetchLatestRemoteCatalog(options),
    readLastKnownGood: () => readLastKnownGoodCatalog(options.storage)
  };
}

export async function fetchLatestRemoteCatalog(
  options: RemoteCatalogFetcherOptions = {}
): Promise<RemoteCatalogFetchResult> {
  const fetchImpl = options.fetchImpl ?? globalThis.fetch;

  if (typeof fetchImpl !== "function") {
    throw new Error("Remote catalog fetch requires a fetch implementation.");
  }

  try {
    const response = await fetchImpl(`${options.endpoint ?? DEFAULT_API_ENDPOINT}/trpc/catalog.latest`, {
      body: JSON.stringify({
        input: {
          channel: options.channel ?? "stable",
          clientVersion: options.clientVersion
        }
      }),
      headers: {
        "content-type": "application/json"
      },
      method: "POST"
    });

    if (!response.ok) {
      throw new Error(`Remote catalog request failed with HTTP ${response.status}.`);
    }

    const envelope = await response.json();
    const verified = await verifySignedCatalogEnvelope(envelope, {
      allowedPrivilegedCommandIds: options.allowedPrivilegedCommandIds,
      publicKeyJwk: options.publicKeyJwk ?? DEFAULT_CATALOG_PUBLIC_KEY_JWK
    });

    writeLastKnownGoodCatalog(options.storage, envelope);

    return {
      catalog: verified.payload,
      fromCache: false,
      integrity: verified.integrity,
      signatureKeyId: verified.signature.keyId
    };
  } catch (error) {
    const cached = readLastKnownGoodCatalog(options.storage);

    if (cached) {
      const verified = await verifySignedCatalogEnvelope(cached, {
        allowedPrivilegedCommandIds: options.allowedPrivilegedCommandIds,
        publicKeyJwk: options.publicKeyJwk ?? DEFAULT_CATALOG_PUBLIC_KEY_JWK
      });

      return {
        catalog: verified.payload,
        fromCache: true,
        integrity: verified.integrity,
        signatureKeyId: verified.signature.keyId
      };
    }

    throw error;
  }
}

export function readLastKnownGoodCatalog(storage: StorageLike | undefined = browserStorage()) {
  const raw = storage?.getItem(CATALOG_CACHE_KEY);

  if (!raw) {
    return undefined;
  }

  try {
    return JSON.parse(raw);
  } catch {
    return undefined;
  }
}

function writeLastKnownGoodCatalog(storage: StorageLike | undefined = browserStorage(), envelope: unknown) {
  storage?.setItem(CATALOG_CACHE_KEY, JSON.stringify(envelope));
}

function browserStorage(): StorageLike | undefined {
  try {
    return globalThis.localStorage;
  } catch {
    return undefined;
  }
}
