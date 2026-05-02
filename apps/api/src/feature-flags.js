export const API_FEATURE_FLAGS_VERSION = "0.1.0";

export const DEFAULT_FEATURE_FLAGS = deepFreeze([
  {
    channelOverrides: {
      beta: {
        enabled: true,
        reason: "Beta receives updater and privileged-agent soak flags before stable.",
        rolloutPercent: 100,
        variant: "soak"
      },
      dev: {
        enabled: true,
        reason: "Dev receives internal release diagnostics.",
        rolloutPercent: 100,
        variant: "internal"
      }
    },
    defaultVariant: "off",
    description: "Gate release-system diagnostics outside public stable builds.",
    enabled: false,
    key: "release.diagnostics",
    reason: "Stable defaults to signed public release behavior.",
    rolloutPercent: 0
  },
  {
    channelOverrides: {
      beta: {
        enabled: true,
        reason: "Lab tweak and privileged-agent changes ship to beta before stable.",
        rolloutPercent: 25,
        variant: "guarded"
      },
      dev: {
        enabled: true,
        reason: "Dev channel can exercise guarded feature surfaces early.",
        rolloutPercent: 100,
        variant: "internal"
      }
    },
    defaultVariant: "off",
    description: "Expose guarded Lab feature surfaces only outside stable by default.",
    enabled: false,
    key: "optimizer.labTweaks",
    reason: "Stable keeps Lab tweak surfaces disabled until explicit approval.",
    rolloutPercent: 0
  },
  {
    channelOverrides: {
      dev: {
        enabled: true,
        reason: "Dev channel validates new app-update metadata handling.",
        rolloutPercent: 100,
        variant: "preview"
      }
    },
    defaultVariant: "stable",
    description: "Gate app updater preview behavior.",
    enabled: false,
    key: "updater.preview",
    reason: "Updater previews stay out of beta and stable until signed release checks pass.",
    rolloutPercent: 0
  }
]);

export function createFeatureFlagEvaluationResponse(input = {}, options = {}) {
  const channel = input.channel ?? options.defaultChannel ?? "stable";
  const flags = options.featureFlags ?? DEFAULT_FEATURE_FLAGS;
  const overrides = options.featureFlagOverrides ?? [];

  return {
    channel,
    evaluations: input.flagKeys.map((key) =>
      evaluateFeatureFlag(key, {
        channel,
        deviceId: input.deviceId,
        flags,
        overrides,
        userId: input.userId
      })
    ),
    version: API_FEATURE_FLAGS_VERSION
  };
}

export function evaluateFeatureFlag(key, options = {}) {
  const flag = options.flags.find((candidate) => candidate.key === key);

  if (!flag) {
    return {
      enabled: false,
      key,
      reason: "Unknown flags default off.",
      rolloutPercent: 0,
      source: "default"
    };
  }

  const override = selectFeatureFlagOverride(key, options);
  if (override) {
    return normalizeEvaluation(key, override, override.source, options);
  }

  const channelOverride = flag.channelOverrides?.[options.channel];
  if (channelOverride) {
    return normalizeEvaluation(key, channelOverride, "channel", options);
  }

  return normalizeEvaluation(
    key,
    {
      enabled: flag.enabled,
      reason: flag.reason,
      rolloutPercent: flag.rolloutPercent,
      variant: flag.defaultVariant
    },
    "default",
    options
  );
}

function selectFeatureFlagOverride(key, options = {}) {
  const candidates = options.overrides.filter((override) => override.flagKey === key);
  const userOverride = candidates.find((override) => override.userId && override.userId === options.userId);
  if (userOverride) {
    return { ...userOverride, source: "user" };
  }

  const deviceOverride = candidates.find(
    (override) => override.deviceId && override.deviceId === options.deviceId && !override.userId
  );
  if (deviceOverride) {
    return { ...deviceOverride, source: "device" };
  }

  const channelOverride = candidates.find(
    (override) => override.channel && override.channel === options.channel && !override.deviceId && !override.userId
  );
  if (channelOverride) {
    return { ...channelOverride, source: "channel" };
  }

  return undefined;
}

function normalizeEvaluation(key, rule, source, options = {}) {
  const rolloutPercent = Math.max(0, Math.min(100, Number(rule.rolloutPercent ?? 0)));
  const enabled = Boolean(rule.enabled) && isInRollout(rolloutPercent, `${options.channel}:${options.deviceId ?? ""}:${key}`);

  return {
    enabled,
    key,
    reason: rule.reason ?? "Feature flag rule evaluated.",
    rolloutPercent,
    source,
    variant: rule.variant
  };
}

function isInRollout(rolloutPercent, target) {
  if (rolloutPercent >= 100) {
    return true;
  }

  if (rolloutPercent <= 0) {
    return false;
  }

  let hash = 0;
  for (let index = 0; index < target.length; index += 1) {
    hash = (hash * 31 + target.charCodeAt(index)) % 100;
  }

  return hash < rolloutPercent;
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
