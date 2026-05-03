import { execFileSync } from "node:child_process";
import { promises as fs } from "node:fs";
import path from "node:path";

const args = new Map();
let allowMissingTag = false;

for (let index = 2; index < process.argv.length; index += 1) {
  const current = process.argv[index];

  if (current === "--allow-missing-tag") {
    allowMissingTag = true;
    continue;
  }

  if (current.startsWith("--")) {
    const next = process.argv[index + 1];
    if (!next || next.startsWith("--")) {
      throw new Error(`${current} requires a value.`);
    }
    args.set(current.slice(2), next);
    index += 1;
  }
}

const tag = args.get("tag");
const output = args.get("output");

if (!tag) {
  throw new Error("--tag is required.");
}

if (!output) {
  throw new Error("--output is required.");
}

function git(commandArgs, options = {}) {
  return execFileSync("git", commandArgs, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", options.quiet ? "ignore" : "pipe"],
  }).trim();
}

function canGit(commandArgs) {
  try {
    git(commandArgs, { quiet: true });
    return true;
  } catch {
    return false;
  }
}

function gitOrEmpty(commandArgs) {
  try {
    return git(commandArgs, { quiet: true });
  } catch {
    return "";
  }
}

function tagExists(releaseTag) {
  return canGit(["show-ref", "--verify", "--quiet", `refs/tags/${releaseTag}`]);
}

function parseConventionalCommit(subject) {
  const match = subject.match(/^([a-z]+)(?:\(([^)]+)\))?(!)?:\s+(.+)$/);

  if (!match) {
    return {
      section: "Other Changes",
      text: subject,
    };
  }

  const [, type, scope, breaking, description] = match;
  const sectionByType = {
    build: "Build System",
    chore: "Maintenance",
    ci: "Continuous Integration",
    docs: "Documentation",
    feat: "Features",
    fix: "Fixes",
    perf: "Performance",
    refactor: "Refactors",
    security: "Security",
    style: "Style",
    test: "Tests",
  };

  const prefix = scope ? `**${scope}:** ` : "";
  const suffix = breaking ? " (breaking)" : "";

  return {
    section: breaking ? "Breaking Changes" : sectionByType[type] ?? "Other Changes",
    text: `${prefix}${description}${suffix}`,
  };
}

function collectCommits(range) {
  const rawLog = gitOrEmpty(["log", "--format=%H%x1f%s%x1f%b%x1e", range]);
  if (!rawLog) {
    return [];
  }

  return rawLog
    .split("\x1e")
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => {
      const [hash, subject, body = ""] = entry.split("\x1f");
      return {
        hash,
        subject,
        body,
      };
    });
}

function buildSectionMap(commits) {
  const sections = new Map();

  for (const commit of commits) {
    const parsed = parseConventionalCommit(commit.subject);
    const lines = sections.get(parsed.section) ?? [];
    lines.push(`- ${parsed.text} (${commit.hash.slice(0, 7)})`);

    if (/^BREAKING CHANGE:/m.test(commit.body)) {
      const breakingLines = sections.get("Breaking Changes") ?? [];
      breakingLines.push(`- ${commit.subject} (${commit.hash.slice(0, 7)})`);
      sections.set("Breaking Changes", breakingLines);
    }

    sections.set(parsed.section, lines);
  }

  return sections;
}

const exists = tagExists(tag);

if (!exists && !allowMissingTag) {
  throw new Error(`Tag ${tag} does not exist. Use --allow-missing-tag for dry runs.`);
}

const releaseCommit = exists ? git(["rev-list", "-n", "1", tag]) : git(["rev-parse", "HEAD"]);
const previousTag = exists
  ? gitOrEmpty(["describe", "--tags", "--abbrev=0", "--match", "v[0-9]*", `${releaseCommit}^`])
  : gitOrEmpty(["describe", "--tags", "--abbrev=0", "--match", "v[0-9]*", "HEAD"]);
const range = previousTag ? `${previousTag}..${releaseCommit}` : releaseCommit;
const commits = collectCommits(range);
const sections = buildSectionMap(commits);

const orderedSections = [
  "Breaking Changes",
  "Security",
  "Features",
  "Fixes",
  "Performance",
  "Continuous Integration",
  "Build System",
  "Documentation",
  "Tests",
  "Refactors",
  "Maintenance",
  "Style",
  "Other Changes",
];

const lines = [
  `# Liiiraa Booster ${tag}`,
  "",
  `Generated from \`${previousTag ? `${previousTag}..${releaseCommit}` : releaseCommit}\`.`,
  "",
  "## Release Gates",
  "- Signed release tag required for publishing.",
  "- Release secret scan required before metadata upload.",
  "- Artifact attestation required before GitHub release creation.",
  "",
  "## Changes",
];

let wroteSection = false;
for (const section of orderedSections) {
  const entries = sections.get(section);
  if (!entries || entries.length === 0) {
    continue;
  }

  lines.push("", `### ${section}`, ...entries);
  wroteSection = true;
}

if (!wroteSection) {
  lines.push("", "No repository changes were found for this release.");
}

await fs.mkdir(path.dirname(output), { recursive: true });
await fs.writeFile(output, `${lines.join("\n")}\n`, "utf8");

console.log(`Wrote release changelog to ${output}.`);
