import { neon } from "@neondatabase/serverless";
import { drizzle } from "drizzle-orm/neon-http";
import { resolveRuntimeDatabaseUrl } from "./connection.js";
import * as schema from "./schema.js";

export function createNeonRuntimeDatabase(env = process.env, options = {}) {
  const sql = neon(resolveRuntimeDatabaseUrl(env), options.neonOptions);

  return drizzle(sql, {
    logger: options.logger ?? false,
    schema
  });
}
