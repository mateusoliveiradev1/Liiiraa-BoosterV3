import { promises as fs } from "node:fs";
import { execFileSync } from "node:child_process";
import path from "node:path";

const root = process.cwd();

const excludedDirectories = new Set([
  ".git",
  ".next",
  ".turbo",
  ".vite",
  "build",
  "coverage",
  "dist",
  "node_modules",
  "out",
  "target",
]);

const excludedExtensions = new Set([
  ".bmp",
  ".gif",
  ".ico",
  ".jpg",
  ".jpeg",
  ".pdf",
  ".png",
  ".webp",
  ".zip",
]);

const secretPatterns = [
  {
    name: "private key block",
    pattern: /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/,
  },
  {
    name: "GitHub token",
    pattern: /\bgh[pousr]_[A-Za-z0-9_]{30,}\b/,
  },
  {
    name: "AWS access key",
    pattern: /\b(?:A3T[A-Z0-9]|AKIA|ASIA)[A-Z0-9]{16}\b/,
  },
  {
    name: "database URL with password",
    pattern: /\bpostgres(?:ql)?:\/\/[^:\s]+:[^@\s]+@[^/\s]+\/[^\s"')]+/i,
  },
  {
    name: "Tauri signing private key",
    pattern: /\bTAURI_SIGNING_PRIVATE_KEY[^\S\r\n]*=[^\S\r\n]*["']?[A-Za-z0-9+/=]{40,}/,
  },
  {
    name: "environment secret assignment",
    pattern:
      /\b(?:API_KEY|AUTH_TOKEN|CLIENT_SECRET|DATABASE_URL|NEON_DATABASE_URL|PASSWORD|PRIVATE_KEY|SECRET|SIGNING_KEY|TOKEN)[^\S\r\n]*=[^\S\r\n]*["']?(?!<|example|changeme|placeholder|replace_me|todo|your-)[^\s"']{16,}/,
  },
];

const findings = [];
let scannedFiles = 0;

function isProbablyBinary(buffer) {
  return buffer.includes(0);
}

function relativePath(filePath) {
  return path.relative(root, filePath).replaceAll(path.sep, "/");
}

function listReleaseFiles() {
  const raw = execFileSync("git", ["ls-files", "-z", "--cached", "--others", "--exclude-standard"], {
    encoding: "buffer",
    stdio: ["ignore", "pipe", "pipe"],
  });

  return raw
    .toString("utf8")
    .split("\0")
    .filter(Boolean)
    .map((filePath) => path.join(root, filePath));
}

function isExcludedPath(filePath) {
  const relative = relativePath(filePath);
  const segments = relative.split("/");
  const extension = path.extname(relative).toLowerCase();

  return segments.some((segment) => excludedDirectories.has(segment)) || excludedExtensions.has(extension);
}

function isKnownPlaceholder(matchText) {
  if (/example|placeholder|changeme|replace_me|dummy|fake/i.test(matchText)) {
    return true;
  }

  const databaseUrl = matchText.match(/postgres(?:ql)?:\/\/[^\s"')]+/i)?.[0];
  if (!databaseUrl) {
    return false;
  }

  try {
    const parsed = new URL(databaseUrl);
    const username = decodeURIComponent(parsed.username).toLowerCase();
    const password = decodeURIComponent(parsed.password).toLowerCase();
    return ["postgres", "user"].includes(username) && ["pass", "password"].includes(password);
  } catch {
    return false;
  }
}

for (const fullPath of listReleaseFiles()) {
  if (isExcludedPath(fullPath)) {
    continue;
  }

  const stat = await fs.stat(fullPath);
  if (stat.size > 1024 * 1024) {
    continue;
  }

  const buffer = await fs.readFile(fullPath);
  if (isProbablyBinary(buffer)) {
    continue;
  }

  const content = buffer.toString("utf8");
  scannedFiles += 1;

  for (const { name, pattern } of secretPatterns) {
    const match = content.match(pattern);
    if (match) {
      if (isKnownPlaceholder(match[0])) {
        continue;
      }

      const beforeMatch = content.slice(0, match.index);
      const line = beforeMatch.split(/\r?\n/).length;
      findings.push({
        file: relativePath(fullPath),
        line,
        name,
      });
    }
  }
}

if (findings.length > 0) {
  console.error("Release secret scan failed:");
  for (const finding of findings) {
    console.error(`- ${finding.file}:${finding.line} matched ${finding.name}`);
  }
  process.exit(1);
}

console.log(`Release secret scan passed across ${scannedFiles} text files.`);
