import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const migrationDir = join(packageRoot, "drizzle");
const schemaPath = join(packageRoot, "src", "schema.js");

const requiredTables = [
  "auth_users",
  "license_entitlements",
  "devices",
  "app_releases",
  "tweak_catalog_versions",
  "tweak_catalog_entries",
  "benchmark_sessions",
  "benchmark_captures",
  "audit_events",
  "feature_flags",
  "feature_flag_overrides"
];

const requiredEnums = [
  "release_channel",
  "release_platform",
  "benchmark_phase",
  "audit_outcome",
  "license_status"
];

const requiredSchemaExports = [
  "authUsers",
  "licenseEntitlements",
  "devices",
  "appReleases",
  "tweakCatalogVersions",
  "tweakCatalogEntries",
  "benchmarkSessions",
  "benchmarkCaptures",
  "auditEvents",
  "featureFlags",
  "featureFlagOverrides"
];

export function verifyMigrations() {
  const migrationFiles = readdirSync(migrationDir)
    .filter((file) => /^\d{4}_.+\.sql$/.test(file))
    .sort();

  assert(migrationFiles.length > 0, "expected at least one SQL migration");

  const sql = migrationFiles
    .map((file) => readFileSync(join(migrationDir, file), "utf8"))
    .join("\n");
  const schema = readFileSync(schemaPath, "utf8");

  for (const enumName of requiredEnums) {
    assert(sql.includes(`CREATE TYPE ${enumName} AS ENUM`), `missing enum ${enumName}`);
  }

  for (const tableName of requiredTables) {
    assert(sql.includes(`CREATE TABLE ${tableName}`), `missing table ${tableName}`);
  }

  for (const exportName of requiredSchemaExports) {
    assert(schema.includes(`export const ${exportName} = pgTable`), `missing schema export ${exportName}`);
  }

  assert(sql.includes("REFERENCES devices(id)"), "benchmark/audit data must be linkable to devices");
  assert(sql.includes("REFERENCES auth_users(id)"), "optimizer data must be linkable to future auth users");
  assert(sql.includes("payload_sha256 varchar(64) NOT NULL"), "tweak catalogs must carry integrity metadata");
  assert(sql.includes("signature text NOT NULL"), "tweak catalogs must carry signature metadata");
  assert(sql.includes("ck_feature_flags_rollout_percent"), "feature flag rollout bounds are required");
  assert(sql.includes("ck_app_releases_rollout_percent"), "release rollout bounds are required");
  assert(sql.includes("remote_address_hash"), "audit events must store hashed remote address only");
  assert(sql.includes("user_agent_hash"), "audit events must store hashed user agent only");

  return {
    migrations: migrationFiles,
    tables: requiredTables
  };
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const result = verifyMigrations();
  console.log(`Verified ${result.migrations.length} migration with ${result.tables.length} tables.`);
}
