import { sql } from "drizzle-orm";
import {
  boolean,
  index,
  integer,
  jsonb,
  numeric,
  pgEnum,
  pgTable,
  text,
  timestamp,
  uniqueIndex,
  uuid,
  varchar
} from "drizzle-orm/pg-core";

export const releaseChannelEnum = pgEnum("release_channel", ["dev", "beta", "stable"]);
export const releasePlatformEnum = pgEnum("release_platform", ["windows-x64"]);
export const benchmarkPhaseEnum = pgEnum("benchmark_phase", ["before", "after", "single"]);
export const auditOutcomeEnum = pgEnum("audit_outcome", ["allowed", "denied", "failed"]);
export const licenseStatusEnum = pgEnum("license_status", ["pending", "active", "expired", "revoked"]);

const metadata = () => jsonb("metadata").notNull().default(sql`'{}'::jsonb`);
const createdAt = () => timestamp("created_at", { withTimezone: true }).notNull().defaultNow();
const updatedAt = () => timestamp("updated_at", { withTimezone: true }).notNull().defaultNow();

export const authUsers = pgTable(
  "auth_users",
  {
    createdAt: createdAt(),
    displayName: varchar("display_name", { length: 160 }),
    emailHash: varchar("email_hash", { length: 128 }),
    externalSubject: varchar("external_subject", { length: 256 }),
    id: uuid("id").primaryKey().defaultRandom(),
    metadata: metadata(),
    updatedAt: updatedAt()
  },
  (table) => ({
    emailHashIdx: uniqueIndex("ux_auth_users_email_hash").on(table.emailHash),
    externalSubjectIdx: uniqueIndex("ux_auth_users_external_subject").on(table.externalSubject)
  })
);

export const licenseEntitlements = pgTable(
  "license_entitlements",
  {
    createdAt: createdAt(),
    expiresAt: timestamp("expires_at", { withTimezone: true }),
    id: uuid("id").primaryKey().defaultRandom(),
    issuedAt: timestamp("issued_at", { withTimezone: true }),
    licenseKeyHash: varchar("license_key_hash", { length: 128 }).notNull(),
    metadata: metadata(),
    plan: varchar("plan", { length: 64 }).notNull().default("future"),
    status: licenseStatusEnum("status").notNull().default("pending"),
    updatedAt: updatedAt(),
    userId: uuid("user_id").references(() => authUsers.id, { onDelete: "set null" })
  },
  (table) => ({
    licenseKeyHashIdx: uniqueIndex("ux_license_entitlements_key_hash").on(table.licenseKeyHash),
    userIdx: index("ix_license_entitlements_user_id").on(table.userId)
  })
);

export const devices = pgTable(
  "devices",
  {
    appVersion: varchar("app_version", { length: 64 }),
    cpuSummary: varchar("cpu_summary", { length: 240 }),
    createdAt: createdAt(),
    displayName: varchar("display_name", { length: 160 }),
    gpuSummary: varchar("gpu_summary", { length: 240 }),
    id: uuid("id").primaryKey().defaultRandom(),
    installId: varchar("install_id", { length: 128 }).notNull(),
    lastSeenAt: timestamp("last_seen_at", { withTimezone: true }),
    metadata: metadata(),
    osBuild: varchar("os_build", { length: 64 }),
    osName: varchar("os_name", { length: 64 }).notNull().default("windows"),
    stableDeviceHash: varchar("stable_device_hash", { length: 128 }).notNull(),
    updatedAt: updatedAt(),
    userId: uuid("user_id").references(() => authUsers.id, { onDelete: "set null" })
  },
  (table) => ({
    lastSeenIdx: index("ix_devices_last_seen_at").on(table.lastSeenAt),
    stableInstallIdx: uniqueIndex("ux_devices_stable_install").on(table.stableDeviceHash, table.installId),
    userIdx: index("ix_devices_user_id").on(table.userId)
  })
);

export const appReleases = pgTable(
  "app_releases",
  {
    artifactSha256: varchar("artifact_sha256", { length: 64 }),
    artifactUrl: text("artifact_url"),
    channel: releaseChannelEnum("channel").notNull(),
    createdAt: createdAt(),
    id: uuid("id").primaryKey().defaultRandom(),
    isCritical: boolean("is_critical").notNull().default(false),
    metadata: metadata(),
    minimumAppVersion: varchar("minimum_app_version", { length: 64 }),
    platform: releasePlatformEnum("platform").notNull().default("windows-x64"),
    publishedAt: timestamp("published_at", { withTimezone: true }),
    releaseNotesUrl: text("release_notes_url"),
    rolloutPercent: integer("rollout_percent").notNull().default(100),
    signature: text("signature"),
    version: varchar("version", { length: 64 }).notNull()
  },
  (table) => ({
    channelPublishedIdx: index("ix_app_releases_channel_published").on(table.channel, table.publishedAt),
    versionChannelPlatformIdx: uniqueIndex("ux_app_releases_version_channel_platform").on(
      table.version,
      table.channel,
      table.platform
    )
  })
);

export const tweakCatalogVersions = pgTable(
  "tweak_catalog_versions",
  {
    channel: releaseChannelEnum("channel").notNull(),
    createdAt: createdAt(),
    id: uuid("id").primaryKey().defaultRandom(),
    minimumAppVersion: varchar("minimum_app_version", { length: 64 }),
    payloadSha256: varchar("payload_sha256", { length: 64 }).notNull(),
    payloadUrl: text("payload_url"),
    publishedAt: timestamp("published_at", { withTimezone: true }),
    revokedAt: timestamp("revoked_at", { withTimezone: true }),
    schemaVersion: varchar("schema_version", { length: 32 }).notNull(),
    signature: text("signature").notNull(),
    version: varchar("version", { length: 96 }).notNull()
  },
  (table) => ({
    channelPublishedIdx: index("ix_tweak_catalog_versions_channel_published").on(table.channel, table.publishedAt),
    versionChannelIdx: uniqueIndex("ux_tweak_catalog_versions_version_channel").on(table.version, table.channel)
  })
);

export const tweakCatalogEntries = pgTable(
  "tweak_catalog_entries",
  {
    catalogVersionId: uuid("catalog_version_id")
      .notNull()
      .references(() => tweakCatalogVersions.id, { onDelete: "cascade" }),
    category: varchar("category", { length: 80 }).notNull(),
    createdAt: createdAt(),
    id: uuid("id").primaryKey().defaultRandom(),
    mode: varchar("mode", { length: 48 }).notNull(),
    payload: jsonb("payload").notNull().default(sql`'{}'::jsonb`),
    risk: varchar("risk", { length: 48 }).notNull(),
    sortOrder: integer("sort_order").notNull().default(0),
    summary: text("summary").notNull(),
    title: varchar("title", { length: 160 }).notNull(),
    tweakId: varchar("tweak_id", { length: 160 }).notNull()
  },
  (table) => ({
    catalogSortIdx: index("ix_tweak_catalog_entries_catalog_sort").on(table.catalogVersionId, table.sortOrder),
    tweakCatalogIdx: uniqueIndex("ux_tweak_catalog_entries_catalog_tweak").on(
      table.catalogVersionId,
      table.tweakId
    )
  })
);

export const benchmarkSessions = pgTable(
  "benchmark_sessions",
  {
    activeOptimizerProfile: varchar("active_optimizer_profile", { length: 96 }).notNull(),
    activePowerPlan: varchar("active_power_plan", { length: 96 }).notNull(),
    consentedAt: timestamp("consented_at", { withTimezone: true }),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull(),
    deviceId: uuid("device_id").references(() => devices.id, { onDelete: "set null" }),
    driverVersion: varchar("driver_version", { length: 64 }).notNull(),
    externalSessionId: varchar("external_session_id", { length: 128 }).notNull(),
    game: varchar("game", { length: 64 }).notNull(),
    id: uuid("id").primaryKey().defaultRandom(),
    metadata: metadata(),
    sessionLabel: varchar("session_label", { length: 96 }),
    uploadedAt: timestamp("uploaded_at", { withTimezone: true }).notNull().defaultNow(),
    userId: uuid("user_id").references(() => authUsers.id, { onDelete: "set null" }),
    windowsBuild: varchar("windows_build", { length: 64 }).notNull()
  },
  (table) => ({
    createdIdx: index("ix_benchmark_sessions_created_at").on(table.createdAt),
    deviceIdx: index("ix_benchmark_sessions_device_id").on(table.deviceId),
    externalSessionIdx: uniqueIndex("ux_benchmark_sessions_external_id").on(table.externalSessionId),
    userIdx: index("ix_benchmark_sessions_user_id").on(table.userId)
  })
);

export const benchmarkCaptures = pgTable(
  "benchmark_captures",
  {
    averageFps: numeric("average_fps", { precision: 10, scale: 3 }).notNull(),
    capturedAt: timestamp("captured_at", { withTimezone: true }).notNull(),
    delayedFrames: integer("delayed_frames").notNull().default(0),
    droppedFrames: integer("dropped_frames").notNull().default(0),
    externalCaptureId: varchar("external_capture_id", { length: 128 }).notNull(),
    frametimeP50Ms: numeric("frametime_p50_ms", { precision: 10, scale: 3 }).notNull(),
    frametimeP95Ms: numeric("frametime_p95_ms", { precision: 10, scale: 3 }).notNull(),
    frametimeP99Ms: numeric("frametime_p99_ms", { precision: 10, scale: 3 }).notNull(),
    generatedFramesDetected: boolean("generated_frames_detected").notNull().default(false),
    id: uuid("id").primaryKey().defaultRandom(),
    latencyProxy: boolean("latency_proxy").notNull().default(false),
    measurementSource: varchar("measurement_source", { length: 96 }).notNull(),
    metrics: jsonb("metrics").notNull().default(sql`'{}'::jsonb`),
    onePercentLowFps: numeric("one_percent_low_fps", { precision: 10, scale: 3 }).notNull(),
    phase: benchmarkPhaseEnum("phase").notNull(),
    sessionId: uuid("session_id")
      .notNull()
      .references(() => benchmarkSessions.id, { onDelete: "cascade" }),
    zeroPointOnePercentLowFps: numeric("zero_point_one_percent_low_fps", {
      precision: 10,
      scale: 3
    }).notNull()
  },
  (table) => ({
    capturedIdx: index("ix_benchmark_captures_captured_at").on(table.capturedAt),
    sessionCaptureIdx: uniqueIndex("ux_benchmark_captures_session_capture").on(
      table.sessionId,
      table.externalCaptureId
    ),
    sessionIdx: index("ix_benchmark_captures_session_id").on(table.sessionId)
  })
);

export const auditEvents = pgTable(
  "audit_events",
  {
    action: varchar("action", { length: 160 }).notNull(),
    actorType: varchar("actor_type", { length: 48 }).notNull().default("system"),
    createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
    deviceId: uuid("device_id").references(() => devices.id, { onDelete: "set null" }),
    entityId: varchar("entity_id", { length: 160 }),
    entityType: varchar("entity_type", { length: 96 }),
    id: uuid("id").primaryKey().defaultRandom(),
    metadata: metadata(),
    outcome: auditOutcomeEnum("outcome").notNull(),
    procedure: varchar("procedure", { length: 160 }),
    remoteAddressHash: varchar("remote_address_hash", { length: 128 }),
    requestId: varchar("request_id", { length: 128 }),
    userAgentHash: varchar("user_agent_hash", { length: 128 }),
    userId: uuid("user_id").references(() => authUsers.id, { onDelete: "set null" })
  },
  (table) => ({
    createdIdx: index("ix_audit_events_created_at").on(table.createdAt),
    deviceIdx: index("ix_audit_events_device_id").on(table.deviceId),
    requestIdx: index("ix_audit_events_request_id").on(table.requestId),
    userIdx: index("ix_audit_events_user_id").on(table.userId)
  })
);

export const featureFlags = pgTable("feature_flags", {
  constraints: jsonb("constraints").notNull().default(sql`'{}'::jsonb`),
  createdAt: createdAt(),
  defaultVariant: varchar("default_variant", { length: 96 }),
  description: text("description").notNull().default(""),
  enabled: boolean("enabled").notNull().default(false),
  key: varchar("key", { length: 128 }).primaryKey(),
  rolloutPercent: integer("rollout_percent").notNull().default(0),
  updatedAt: updatedAt()
});

export const featureFlagOverrides = pgTable(
  "feature_flag_overrides",
  {
    channel: releaseChannelEnum("channel"),
    createdAt: createdAt(),
    deviceId: uuid("device_id").references(() => devices.id, { onDelete: "cascade" }),
    enabled: boolean("enabled").notNull(),
    expiresAt: timestamp("expires_at", { withTimezone: true }),
    flagKey: varchar("flag_key", { length: 128 })
      .notNull()
      .references(() => featureFlags.key, { onDelete: "cascade" }),
    id: uuid("id").primaryKey().defaultRandom(),
    reason: text("reason"),
    startsAt: timestamp("starts_at", { withTimezone: true }),
    userId: uuid("user_id").references(() => authUsers.id, { onDelete: "cascade" }),
    variant: varchar("variant", { length: 96 })
  },
  (table) => ({
    channelIdx: index("ix_feature_flag_overrides_channel").on(table.channel),
    deviceIdx: index("ix_feature_flag_overrides_device_id").on(table.deviceId),
    flagIdx: index("ix_feature_flag_overrides_flag_key").on(table.flagKey),
    userIdx: index("ix_feature_flag_overrides_user_id").on(table.userId)
  })
);
