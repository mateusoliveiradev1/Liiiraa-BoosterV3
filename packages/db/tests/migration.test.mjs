import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  MIGRATION_DATABASE_URL_ENV,
  RUNTIME_DATABASE_URL_ENV,
  inspectPostgresUrl,
  resolveMigrationDatabaseUrl,
  resolveRuntimeDatabaseUrl
} from "../src/connection.js";
import { verifyMigrations } from "../scripts/verify-migrations.mjs";

describe("Neon database migrations", () => {
  it("cover the required durable product data model", () => {
    const result = verifyMigrations();

    assert.deepEqual(result.migrations, ["0001_neon_product_schema.sql"]);
    assert.equal(result.tables.includes("devices"), true);
    assert.equal(result.tables.includes("app_releases"), true);
    assert.equal(result.tables.includes("tweak_catalog_versions"), true);
    assert.equal(result.tables.includes("benchmark_sessions"), true);
    assert.equal(result.tables.includes("audit_events"), true);
    assert.equal(result.tables.includes("feature_flags"), true);
    assert.deepEqual(result.releaseChannels, ["dev", "beta", "stable"]);
  });
});

describe("Neon connection URL policy", () => {
  it("uses pooled Neon URLs for runtime traffic", () => {
    const env = {
      [RUNTIME_DATABASE_URL_ENV]: "postgres://user:pass@ep-soft-pine-pooler.us-east-2.aws.neon.tech/db"
    };

    assert.equal(resolveRuntimeDatabaseUrl(env), env[RUNTIME_DATABASE_URL_ENV]);
  });

  it("uses direct Neon URLs for migrations when provided", () => {
    const env = {
      [MIGRATION_DATABASE_URL_ENV]: "postgres://user:pass@ep-soft-pine.us-east-2.aws.neon.tech/db",
      [RUNTIME_DATABASE_URL_ENV]: "postgres://user:pass@ep-soft-pine-pooler.us-east-2.aws.neon.tech/db"
    };

    assert.equal(resolveMigrationDatabaseUrl(env), env[MIGRATION_DATABASE_URL_ENV]);
  });

  it("rejects direct Neon runtime URLs and pooled migration URLs", () => {
    assert.throws(
      () =>
        resolveRuntimeDatabaseUrl({
          [RUNTIME_DATABASE_URL_ENV]: "postgres://user:pass@ep-soft-pine.us-east-2.aws.neon.tech/db"
        }),
      /pooled Neon connection/
    );

    assert.throws(
      () =>
        resolveMigrationDatabaseUrl({
          [MIGRATION_DATABASE_URL_ENV]:
            "postgres://user:pass@ep-soft-pine-pooler.us-east-2.aws.neon.tech/db"
        }),
      /direct Neon connection/
    );
  });

  it("classifies Postgres URLs without exposing credentials", () => {
    const info = inspectPostgresUrl("postgres://user:pass@ep-soft-pine-pooler.us-east-2.aws.neon.tech/db");

    assert.deepEqual(info, {
      hostname: "ep-soft-pine-pooler.us-east-2.aws.neon.tech",
      isNeon: true,
      isPooled: true,
      valid: true
    });
  });
});
