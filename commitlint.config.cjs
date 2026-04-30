const allowedTypes = [
  "feat",
  "fix",
  "perf",
  "refactor",
  "test",
  "docs",
  "build",
  "ci",
  "chore",
  "style",
  "security",
  "revert",
];

const recommendedScopes = [
  "openspec",
  "repo",
  "desktop",
  "api",
  "web",
  "db",
  "ui",
  "security",
  "performance",
  "optimizer",
  "windows",
  "nvidia",
  "pubg",
  "benchmark",
  "release",
];

module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "type-enum": [2, "always", allowedTypes],
    "scope-enum": [1, "always", recommendedScopes],
  },
};
