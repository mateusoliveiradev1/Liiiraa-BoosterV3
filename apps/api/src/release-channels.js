import {
  ContractValidationError,
  RELEASE_CHANNELS,
  RELEASE_PLATFORMS
} from "../../../packages/api-contract/src/index.js";

export const API_RELEASE_CHANNELS_VERSION = "0.1.0";

export const releaseChannelDefinitions = deepFreeze([
  {
    description: "Internal builds for fast validation; may break and are never promoted directly to users.",
    id: "dev",
    requiresSignedArtifacts: false,
    riskyChangesFirst: true,
    title: "Dev"
  },
  {
    description: "Signed soak channel for updater, privileged-agent, and Lab tweak changes before stable.",
    id: "beta",
    requiresSignedArtifacts: true,
    riskyChangesFirst: true,
    title: "Beta"
  },
  {
    description: "Signed public channel after beta soak or explicit release approval.",
    id: "stable",
    requiresSignedArtifacts: true,
    riskyChangesFirst: false,
    title: "Stable"
  }
]);

export const DEFAULT_APP_RELEASES = deepFreeze([
  {
    artifactSha256: "0".repeat(64),
    artifactUrl: "https://updates.liiiraa.example/dev/windows-x64/LiiiraaBoost-0.2.0-dev.3.msi",
    channel: "dev",
    isCritical: false,
    minimumAppVersion: "0.0.0",
    platform: "windows-x64",
    publishedAtUtc: "2026-05-02T00:00:00Z",
    releaseNotesUrl: "https://updates.liiiraa.example/dev/0.2.0-dev.3",
    rolloutPercent: 100,
    signature: "dev-update-signature-placeholder",
    version: "0.2.0-dev.3"
  },
  {
    artifactSha256: "1".repeat(64),
    artifactUrl: "https://updates.liiiraa.example/beta/windows-x64/LiiiraaBoost-0.1.0-beta.2.msi",
    channel: "beta",
    isCritical: false,
    minimumAppVersion: "0.0.0",
    platform: "windows-x64",
    publishedAtUtc: "2026-05-02T00:00:00Z",
    releaseNotesUrl: "https://updates.liiiraa.example/beta/0.1.0-beta.2",
    rolloutPercent: 50,
    signature: "beta-update-signature-placeholder",
    version: "0.1.0-beta.2"
  },
  {
    artifactSha256: "2".repeat(64),
    artifactUrl: "https://updates.liiiraa.example/stable/windows-x64/LiiiraaBoost-0.1.0.msi",
    channel: "stable",
    isCritical: false,
    minimumAppVersion: "0.0.0",
    platform: "windows-x64",
    publishedAtUtc: "2026-05-02T00:00:00Z",
    releaseNotesUrl: "https://updates.liiiraa.example/stable/0.1.0",
    rolloutPercent: 100,
    signature: "stable-update-signature-placeholder",
    version: "0.1.0"
  }
]);

export function createReleaseChannelsResponse(options = {}) {
  return {
    channels: options.channels ?? releaseChannelDefinitions,
    defaultChannel: options.defaultChannel ?? "stable",
    version: API_RELEASE_CHANNELS_VERSION
  };
}

export function createLatestReleaseResponse(input = {}, options = {}) {
  const channel = normalizeReleaseChannel(input.channel ?? options.defaultChannel ?? "stable");
  const platform = normalizeReleasePlatform(input.platform ?? "windows-x64");
  const release = selectLatestReleaseForChannel(options.releases ?? DEFAULT_APP_RELEASES, {
    channel,
    now: options.releaseNow,
    platform
  });

  return {
    artifactSha256: release.artifactSha256,
    artifactUrl: release.artifactUrl,
    channel: release.channel,
    isCritical: Boolean(release.isCritical),
    minimumAppVersion: release.minimumAppVersion,
    platform: release.platform,
    publishedAtUtc: release.publishedAtUtc,
    releaseNotesUrl: release.releaseNotesUrl,
    rolloutPercent: release.rolloutPercent,
    signature: release.signature,
    updateAvailable: Boolean(input.clientVersion && input.clientVersion !== release.version),
    version: release.version
  };
}

export function selectLatestReleaseForChannel(releases, options = {}) {
  const channel = normalizeReleaseChannel(options.channel ?? "stable");
  const platform = normalizeReleasePlatform(options.platform ?? "windows-x64");
  const nowMs = typeof options.now === "function" ? options.now() : undefined;

  const candidates = releases
    .filter((release) => release.channel === channel && release.platform === platform)
    .filter((release) => nowMs === undefined || Date.parse(release.publishedAtUtc) <= nowMs)
    .sort((left, right) => Date.parse(right.publishedAtUtc) - Date.parse(left.publishedAtUtc));

  if (candidates.length === 0) {
    throw new ContractValidationError("No release metadata is available for the requested channel.", [
      {
        code: "release_not_found",
        expected: "published release metadata",
        path: ["payload", "channel"]
      }
    ]);
  }

  return candidates[0];
}

function normalizeReleaseChannel(channel) {
  if (!RELEASE_CHANNELS.includes(channel)) {
    throw new ContractValidationError("Unknown release channel.", [
      {
        code: "invalid_enum",
        expected: RELEASE_CHANNELS.join(" | "),
        path: ["payload", "channel"]
      }
    ]);
  }

  return channel;
}

function normalizeReleasePlatform(platform) {
  if (!RELEASE_PLATFORMS.includes(platform)) {
    throw new ContractValidationError("Unknown release platform.", [
      {
        code: "invalid_enum",
        expected: RELEASE_PLATFORMS.join(" | "),
        path: ["payload", "platform"]
      }
    ]);
  }

  return platform;
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
