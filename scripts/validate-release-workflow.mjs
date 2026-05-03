import { promises as fs } from "node:fs";

const workflowPath = ".github/workflows/release.yml";
const requiredScripts = [
  "scripts/check-release-no-secrets.mjs",
  "scripts/generate-release-changelog.mjs",
];

const workflow = await fs.readFile(workflowPath, "utf8");
const errors = [];

function requireMatch(description, pattern) {
  if (!pattern.test(workflow)) {
    errors.push(description);
  }
}

requireMatch("release workflow must use tag push triggers", /tags:\s*\n\s*-\s+"v\*\.\*\.\*"/);
requireMatch("release workflow must support workflow_dispatch dry runs", /workflow_dispatch:/);
requireMatch("top-level permissions must default to read-all", /^permissions:\s*read-all$/m);
requireMatch("prepare job must run with contents: read", /prepare-release:[\s\S]*?permissions:\s*\n\s*contents:\s*read/);
requireMatch("publish job must request contents: write", /publish-release:[\s\S]*?permissions:\s*\n[\s\S]*?contents:\s*write/);
requireMatch("publish job must request attestations: write", /publish-release:[\s\S]*?attestations:\s*write/);
requireMatch("publish job must request id-token: write", /publish-release:[\s\S]*?id-token:\s*write/);
requireMatch("publishing must be skipped for dry runs", /if:\s*needs\.prepare-release\.outputs\.dry_run\s*!=\s*'true'/);
requireMatch("release tags must be verified as signed before publishing", /git verify-tag "\$\{?RELEASE_TAG\}?"/);
requireMatch("release workflow must run the no-secret scan", /node scripts\/check-release-no-secrets\.mjs/);
requireMatch("release workflow must generate a changelog", /node scripts\/generate-release-changelog\.mjs/);
requireMatch("release workflow must create an artifact attestation", /actions\/attest-build-provenance@[a-f0-9]{40}/);
requireMatch("release workflow must create a verified GitHub release", /gh release create "\$\{?RELEASE_TAG\}?"[\s\S]*--verify-tag/);

if (/pull_request_target:/.test(workflow)) {
  errors.push("release workflow must not use pull_request_target");
}

const usesReferences = [...workflow.matchAll(/uses:\s*([^\s#]+)/g)].map((match) => match[1]);
for (const reference of usesReferences) {
  if (!/@[a-f0-9]{40}$/.test(reference)) {
    errors.push(`action reference must be pinned to a full-length SHA: ${reference}`);
  }
}

for (const script of requiredScripts) {
  try {
    await fs.access(script);
  } catch {
    errors.push(`required script is missing: ${script}`);
  }
}

if (errors.length > 0) {
  console.error("Release workflow validation failed:");
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log("Release workflow validation passed.");
