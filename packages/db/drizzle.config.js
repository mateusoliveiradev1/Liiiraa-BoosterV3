import { defineConfig } from "drizzle-kit";
import { resolveMigrationDatabaseUrl } from "./src/connection.js";

export default defineConfig({
  dbCredentials: {
    url: resolveMigrationDatabaseUrl(process.env)
  },
  dialect: "postgresql",
  out: "./drizzle",
  schema: "./src/schema.js",
  strict: true,
  verbose: true
});
