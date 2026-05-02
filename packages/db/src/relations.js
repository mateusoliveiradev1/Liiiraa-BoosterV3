import { relations } from "drizzle-orm";
import {
  appReleases,
  auditEvents,
  authUsers,
  benchmarkCaptures,
  benchmarkSessions,
  devices,
  featureFlagOverrides,
  featureFlags,
  licenseEntitlements,
  tweakCatalogEntries,
  tweakCatalogVersions
} from "./schema.js";

export const authUsersRelations = relations(authUsers, ({ many }) => ({
  auditEvents: many(auditEvents),
  benchmarkSessions: many(benchmarkSessions),
  devices: many(devices),
  licenseEntitlements: many(licenseEntitlements)
}));

export const licenseEntitlementsRelations = relations(licenseEntitlements, ({ one }) => ({
  user: one(authUsers, {
    fields: [licenseEntitlements.userId],
    references: [authUsers.id]
  })
}));

export const devicesRelations = relations(devices, ({ many, one }) => ({
  auditEvents: many(auditEvents),
  benchmarkSessions: many(benchmarkSessions),
  user: one(authUsers, {
    fields: [devices.userId],
    references: [authUsers.id]
  })
}));

export const appReleasesRelations = relations(appReleases, () => ({}));

export const tweakCatalogVersionsRelations = relations(tweakCatalogVersions, ({ many }) => ({
  entries: many(tweakCatalogEntries)
}));

export const tweakCatalogEntriesRelations = relations(tweakCatalogEntries, ({ one }) => ({
  catalogVersion: one(tweakCatalogVersions, {
    fields: [tweakCatalogEntries.catalogVersionId],
    references: [tweakCatalogVersions.id]
  })
}));

export const benchmarkSessionsRelations = relations(benchmarkSessions, ({ many, one }) => ({
  captures: many(benchmarkCaptures),
  device: one(devices, {
    fields: [benchmarkSessions.deviceId],
    references: [devices.id]
  }),
  user: one(authUsers, {
    fields: [benchmarkSessions.userId],
    references: [authUsers.id]
  })
}));

export const benchmarkCapturesRelations = relations(benchmarkCaptures, ({ one }) => ({
  session: one(benchmarkSessions, {
    fields: [benchmarkCaptures.sessionId],
    references: [benchmarkSessions.id]
  })
}));

export const auditEventsRelations = relations(auditEvents, ({ one }) => ({
  device: one(devices, {
    fields: [auditEvents.deviceId],
    references: [devices.id]
  }),
  user: one(authUsers, {
    fields: [auditEvents.userId],
    references: [authUsers.id]
  })
}));

export const featureFlagsRelations = relations(featureFlags, ({ many }) => ({
  overrides: many(featureFlagOverrides)
}));

export const featureFlagOverridesRelations = relations(featureFlagOverrides, ({ one }) => ({
  device: one(devices, {
    fields: [featureFlagOverrides.deviceId],
    references: [devices.id]
  }),
  flag: one(featureFlags, {
    fields: [featureFlagOverrides.flagKey],
    references: [featureFlags.key]
  }),
  user: one(authUsers, {
    fields: [featureFlagOverrides.userId],
    references: [authUsers.id]
  })
}));
