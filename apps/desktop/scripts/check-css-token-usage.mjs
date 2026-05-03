import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const desktopRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const reviewedFiles = [
  "src/styles.css",
  "src/components/OptimizationWorkflow.tsx",
  "src/components/settings/SettingsTrustSurfaces.tsx",
  "src/routes/BenchmarksRoute.tsx"
];

const retiredVisualPatterns = [
  {
    pattern: /radial-gradient/i,
    reason: "Use graphite surfaces and semantic strokes instead of radial page washes."
  },
  {
    pattern: /#(?:13d8ff|27d7ff|35ff8f|3af28f|b08cff|9b7cff|ffb13d|ffbd5a|ff4d6a|ff5a67)\b/i,
    reason: "Use designTokens.ts state or chart tokens instead of retired neon literals."
  },
  {
    pattern: /rgba\(\s*(?:19,\s*216,\s*255|39,\s*215,\s*255|53,\s*255,\s*143|176,\s*140,\s*255)\b/i,
    reason: "Use tokenized state-surface variables instead of one-off accent alpha colors."
  }
];

const failures = [];

for (const relativeFile of reviewedFiles) {
  const filePath = resolve(desktopRoot, relativeFile);
  const content = readFileSync(filePath, "utf8");

  for (const { pattern, reason } of retiredVisualPatterns) {
    if (pattern.test(content)) {
      failures.push(`${relativeFile}: ${reason}`);
    }
  }
}

if (failures.length > 0) {
  throw new Error(`Desktop visual token guard failed:\n${failures.join("\n")}`);
}

console.log("Desktop visual token guard passed.");
