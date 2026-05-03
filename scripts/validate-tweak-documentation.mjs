import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const CHANGE_DIR = path.join(REPO_ROOT, "openspec", "changes", "build-liiiraa-boost-platform");

export const DEFAULT_DOCUMENT_PATHS = Object.freeze({
  hardeningReview: path.join(CHANGE_DIR, "tweak-hardening-review.md"),
  tweakDefinitionStandard: path.join(CHANGE_DIR, "tweak-definition-standard.md"),
  v1TweakMatrix: path.join(CHANGE_DIR, "v1-tweak-matrix.md")
});

export const REQUIRED_TWEAK_DOCUMENTATION_FIELDS = Object.freeze([
  "sourceLinks",
  "do",
  "dont",
  "backup",
  "verify",
  "rollback",
  "risk",
  "antiCheatNotes",
  "supportedOs",
  "supportedHardware",
  "supportedDrivers",
  "unsupportedWhen",
  "conflictsWith",
  "knownSideEffects"
]);

const REQUIRED_STANDARD_PHRASES = Object.freeze([
  "A tweak with no source cannot be implemented.",
  "Backup must complete before apply for any mutable tweak.",
  "Verify must prove the target state or explain why verification is impossible.",
  "Rollback must restore the exact previous state when possible.",
  "hardware, driver, display, game, or anti-cheat sensitivity must declare applicability rules",
  "must declare conflicts with related settings",
  "side effects and conflicts",
  "game and anti-cheat must be closed before apply"
]);

const REQUIRED_MATRIX_PHRASES = Object.freeze([
  "Every applied tweak must support detect, precheck, dry-run plan, backup, apply, verify, rollback, do, dont, source links, risk notes, and anti-cheat notes.",
  "Implementation must create either a real `TweakDefinition` or a blocked guardrail for every ID here.",
  "PUBG/BattlEye sources make anti-cheat safety non-negotiable"
]);

const REQUIRED_HARDENING_PHRASES = Object.freeze([
  "It satisfies `tweak-definition-standard.md`.",
  "It has source links and evidence level.",
  "It declares conflicts and side effects.",
  "It has a dry-run plan.",
  "It has backup and rollback.",
  "It has verification.",
  "It has negative tests for unsafe/default behavior."
]);

const PLACEHOLDER_PATTERN = /\b(TBD|TODO|FIXME|placeholder|stub only|unknown)\b/iu;
const KNOWN_MODE_PATTERN = /\b(Safe|Competitive|Lab|Blocked)\b/u;
const TWEAK_ID_PATTERN = /^[a-z0-9][a-z0-9._-]{1,95}$/u;

export async function readDefaultTweakDocumentation(paths = DEFAULT_DOCUMENT_PATHS) {
  const [matrix, standard, hardening] = await Promise.all([
    readFile(paths.v1TweakMatrix, "utf8"),
    readFile(paths.tweakDefinitionStandard, "utf8"),
    readFile(paths.hardeningReview, "utf8")
  ]);

  return {
    hardening,
    matrix,
    standard
  };
}

export function parseTweakMatrix(matrixMarkdown) {
  const rows = [];
  const lines = matrixMarkdown.split(/\r?\n/u);
  let currentSection = "";
  let currentHeaders = undefined;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const heading = line.match(/^##\s+(.+)$/u);

    if (heading) {
      currentSection = heading[1].trim();
      currentHeaders = undefined;
      continue;
    }

    if (!line.trim().startsWith("|")) {
      continue;
    }

    const cells = parseTableRow(line);
    if (cells.length === 0 || isSeparatorRow(cells)) {
      continue;
    }

    const nextLine = lines[index + 1] ?? "";
    const nextCells = nextLine.trim().startsWith("|") ? parseTableRow(nextLine) : [];

    if (nextCells.length === cells.length && isSeparatorRow(nextCells)) {
      currentHeaders = cells;
      continue;
    }

    if (!currentHeaders?.includes("ID")) {
      continue;
    }

    const row = Object.fromEntries(
      currentHeaders.map((header, cellIndex) => [header, cells[cellIndex]?.trim() ?? ""])
    );

    if (row.ID) {
      rows.push({
        line: index + 1,
        section: currentSection,
        values: row
      });
    }
  }

  return rows;
}

export function validateTweakDocumentation(docs) {
  const issues = [];
  const tweakRows = parseTweakMatrix(docs.matrix);

  validateGlobalContract(docs, issues);
  validateMatrixRows(tweakRows, issues);

  return {
    issueCount: issues.length,
    issues,
    sectionCount: new Set(tweakRows.map((row) => row.section)).size,
    tweakCount: tweakRows.length
  };
}

export function formatIssue(issue) {
  const location = issue.line ? `${issue.path}:${issue.line}` : issue.path;
  return `${location} ${issue.code}: ${issue.message}`;
}

function validateGlobalContract(docs, issues) {
  for (const field of REQUIRED_TWEAK_DOCUMENTATION_FIELDS) {
    if (!docs.standard.includes(field)) {
      issues.push({
        code: "missing_required_schema_field",
        message: `Required TweakDefinition field is absent: ${field}`,
        path: "tweak-definition-standard.md"
      });
    }
  }

  for (const phrase of REQUIRED_STANDARD_PHRASES) {
    if (!docs.standard.includes(phrase)) {
      issues.push({
        code: "missing_standard_rule",
        message: `Required standard rule is absent: ${phrase}`,
        path: "tweak-definition-standard.md"
      });
    }
  }

  for (const phrase of REQUIRED_MATRIX_PHRASES) {
    if (!docs.matrix.includes(phrase)) {
      issues.push({
        code: "missing_matrix_rule",
        message: `Required V1 matrix rule is absent: ${phrase}`,
        path: "v1-tweak-matrix.md"
      });
    }
  }

  for (const phrase of REQUIRED_HARDENING_PHRASES) {
    if (!docs.hardening.includes(phrase)) {
      issues.push({
        code: "missing_hardening_gate",
        message: `Required hardening acceptance gate is absent: ${phrase}`,
        path: "tweak-hardening-review.md"
      });
    }
  }
}

function validateMatrixRows(tweakRows, issues) {
  if (tweakRows.length < 100) {
    issues.push({
      code: "matrix_parse_too_small",
      message: `Expected at least 100 tweak rows, found ${tweakRows.length}.`,
      path: "v1-tweak-matrix.md"
    });
  }

  const seen = new Map();

  for (const row of tweakRows) {
    const id = row.values.ID;
    const mode = row.values.Mode ?? "";
    const behavior = row.values["V1 Behavior"] ?? row.values["Blocked Action"] ?? "";
    const rollback = row.values["Precheck and Rollback"] ?? "";
    const defaultPolicy = row.values.Default ?? row.values.Reason ?? "";
    const searchableText = [mode, behavior, rollback, defaultPolicy].join(" ");

    if (!TWEAK_ID_PATTERN.test(id)) {
      addRowIssue(issues, row, "invalid_tweak_id", `Tweak ID is not a safe catalog identifier: ${id}`);
    }

    if (seen.has(id)) {
      addRowIssue(
        issues,
        row,
        "duplicate_tweak_id",
        `Duplicate tweak ID also appears on line ${seen.get(id)}: ${id}`
      );
    }
    seen.set(id, row.line);

    if (!mode && !row.values["Blocked Action"]) {
      addRowIssue(issues, row, "missing_mode", "Tweak row must declare a mode or blocked action.");
    } else if (mode && !KNOWN_MODE_PATTERN.test(mode)) {
      addRowIssue(issues, row, "unknown_mode", `Tweak mode must include Safe, Competitive, Lab, or Blocked: ${mode}`);
    }

    requireFilledCell(row, "V1 Behavior", "Blocked Action", "missing_behavior", issues);
    requireFilledCell(row, "Precheck and Rollback", "Reason", "missing_backup_rollback_or_reason", issues);
    requireFilledCell(row, "Default", "Reason", "missing_default_or_reason", issues);

    if (PLACEHOLDER_PATTERN.test(searchableText)) {
      addRowIssue(issues, row, "placeholder_text", "Tweak row contains placeholder language.");
    }

    if (mode.includes("Blocked") || row.values["Blocked Action"]) {
      validateBlockedRow(row, issues);
    } else {
      validateActionableRow(row, rollback, issues);
    }
  }
}

function validateActionableRow(row, rollback, issues) {
  if (rollback.trim().length < 3) {
    addRowIssue(
      issues,
      row,
      "weak_backup_rollback_documentation",
      "Actionable tweak row must document the Precheck and Rollback column."
    );
  }
}

function validateBlockedRow(row, issues) {
  const behavior = row.values["V1 Behavior"] ?? row.values["Blocked Action"] ?? "";
  const reason = row.values.Reason ?? row.values["Precheck and Rollback"] ?? "";

  if (`${behavior} ${reason}`.trim().length < 8) {
    addRowIssue(
      issues,
      row,
      "weak_blocked_documentation",
      "Blocked tweak row must explain the blocked action and reason."
    );
  }
}

function requireFilledCell(row, primary, fallback, code, issues) {
  const value = row.values[primary] ?? row.values[fallback] ?? "";

  if (!value.trim()) {
    addRowIssue(issues, row, code, `Tweak row must fill ${primary} or ${fallback}.`);
  }
}

function addRowIssue(issues, row, code, message) {
  issues.push({
    code,
    line: row.line,
    message,
    path: "v1-tweak-matrix.md",
    section: row.section,
    tweakId: row.values.ID
  });
}

function parseTableRow(line) {
  return line
    .trim()
    .replace(/^\|/u, "")
    .replace(/\|$/u, "")
    .split("|")
    .map((cell) => cell.trim());
}

function isSeparatorRow(cells) {
  return cells.every((cell) => /^:?-{3,}:?$/u.test(cell));
}

async function main() {
  const docs = await readDefaultTweakDocumentation();
  const result = validateTweakDocumentation(docs);

  if (result.issueCount > 0) {
    console.error(`Tweak documentation gate failed with ${result.issueCount} issue(s):`);
    for (const issue of result.issues) {
      console.error(`- ${formatIssue(issue)}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log(
    `Tweak documentation gate passed for ${result.tweakCount} tweak rows across ${result.sectionCount} sections.`
  );
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await main();
}
