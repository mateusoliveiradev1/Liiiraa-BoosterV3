import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  REQUIRED_TWEAK_DOCUMENTATION_FIELDS,
  parseTweakMatrix,
  validateTweakDocumentation
} from "../../../scripts/validate-tweak-documentation.mjs";

describe("tweak documentation gate", () => {
  it("passes a complete V1 tweak matrix and hardening contract", () => {
    const docs = completeDocsFixture();
    const result = validateTweakDocumentation(docs);

    assert.deepEqual(result.issues, []);
    assert.ok(result.tweakCount >= 100);
    assert.equal(result.sectionCount, 1);
  });

  it("reports missing required documentation fields", () => {
    const docs = completeDocsFixture({
      standard: completeStandardFixture().replace("sourceLinks", "sourceRefs")
    });
    const result = validateTweakDocumentation(docs);

    assert.equal(result.issues.some((issue) => issue.code === "missing_required_schema_field"), true);
  });

  it("parses every matrix row with a stable ID and section", () => {
    const docs = completeDocsFixture();
    const rows = parseTweakMatrix(docs.matrix);

    assert.ok(rows.every((row) => row.values.ID));
    assert.ok(rows.every((row) => row.section));
    assert.equal(new Set(rows.map((row) => row.values.ID)).size, rows.length);
  });
});

function completeDocsFixture(overrides = {}) {
  return {
    hardening: overrides.hardening ?? completeHardeningFixture(),
    matrix: overrides.matrix ?? completeMatrixFixture(),
    standard: overrides.standard ?? completeStandardFixture()
  };
}

function completeStandardFixture() {
  return `
type TweakDefinition = {
${REQUIRED_TWEAK_DOCUMENTATION_FIELDS.map((field) => `  ${field}: unknown;`).join("\n")}
};

A tweak with no source cannot be implemented.
Backup must complete before apply for any mutable tweak.
Verify must prove the target state or explain why verification is impossible.
Rollback must restore the exact previous state when possible.
hardware, driver, display, game, or anti-cheat sensitivity must declare applicability rules
must declare conflicts with related settings
side effects and conflicts
game and anti-cheat must be closed before apply
`;
}

function completeHardeningFixture() {
  return `
It satisfies \`tweak-definition-standard.md\`.
It has source links and evidence level.
It declares conflicts and side effects.
It has a dry-run plan.
It has backup and rollback.
It has verification.
It has negative tests for unsafe/default behavior.
`;
}

function completeMatrixFixture() {
  const rows = Array.from({ length: 100 }, (_, index) => {
    const suffix = String(index + 1).padStart(3, "0");
    return `| test.tweak.${suffix} | Safe | Read and document tweak ${suffix}. | Read-only; no rollback. | On |`;
  }).join("\n");

  return `
# V1 Tweak Matrix

Implementation must create either a real \`TweakDefinition\` or a blocked guardrail for every ID here.
Every applied tweak must support detect, precheck, dry-run plan, backup, apply, verify, rollback, do, dont, source links, risk notes, and anti-cheat notes.
PUBG/BattlEye sources make anti-cheat safety non-negotiable

## Fixture Tweaks

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
${rows}
`;
}
