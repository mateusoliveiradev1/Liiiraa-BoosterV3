export const RUNTIME_DATABASE_URL_ENV = "NEON_DATABASE_URL";
export const MIGRATION_DATABASE_URL_ENV = "NEON_DATABASE_DIRECT_URL";

const POSTGRES_PROTOCOLS = new Set(["postgres:", "postgresql:"]);

export function resolveRuntimeDatabaseUrl(env = process.env) {
  return requireDatabaseUrl(env[RUNTIME_DATABASE_URL_ENV], RUNTIME_DATABASE_URL_ENV, {
    preferPooled: true
  });
}

export function resolveMigrationDatabaseUrl(env = process.env) {
  const directUrl = env[MIGRATION_DATABASE_URL_ENV];
  if (directUrl) {
    return requireDatabaseUrl(directUrl, MIGRATION_DATABASE_URL_ENV, {
      preferDirect: true
    });
  }

  return requireDatabaseUrl(env[RUNTIME_DATABASE_URL_ENV], RUNTIME_DATABASE_URL_ENV, {
    preferPooled: false
  });
}

export function requireDatabaseUrl(value, envName, options = {}) {
  if (!value || String(value).trim() === "") {
    throw new Error(`${envName} is required for Neon Postgres access.`);
  }

  const normalized = String(value).trim();
  const info = inspectPostgresUrl(normalized);

  if (!info.valid) {
    throw new Error(`${envName} must be a postgres:// or postgresql:// URL.`);
  }

  if (options.preferPooled && info.isNeon && !info.isPooled) {
    throw new Error(`${envName} must use a pooled Neon connection for API runtime traffic.`);
  }

  if (options.preferDirect && info.isNeon && info.isPooled) {
    throw new Error(`${envName} must use a direct Neon connection for migrations.`);
  }

  return normalized;
}

export function inspectPostgresUrl(value) {
  try {
    const parsed = new URL(value);
    const hostname = parsed.hostname.toLowerCase();

    return {
      hostname,
      isNeon: hostname.endsWith(".neon.tech"),
      isPooled: hostname.includes("-pooler.") || hostname.includes("-pooler"),
      valid: POSTGRES_PROTOCOLS.has(parsed.protocol)
    };
  } catch {
    return {
      hostname: "",
      isNeon: false,
      isPooled: false,
      valid: false
    };
  }
}
