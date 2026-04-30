import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));

const [tokens, snapshot, themeCss] = await Promise.all([
  readJson(resolve(packageDir, "tokens/liiiraa.tokens.json")),
  readJson(resolve(packageDir, "tests/token.snapshot.json")),
  readFile(resolve(packageDir, "styles/theme.css"), "utf8")
]);

const fail = (message) => {
  throw new Error(message);
};

const get = (object, path) =>
  path.split(".").reduce((value, key) => (value == null ? value : value[key]), object);

const requiredPaths = [
  "meta.product",
  "meta.signature",
  "colors.background.app",
  "colors.surface.panel",
  "colors.border.focus",
  "colors.text.primary",
  "colors.text.secondary",
  "colors.accent.telemetry",
  "colors.accent.performance",
  "colors.status.warning",
  "colors.status.danger",
  "colors.risk.low",
  "colors.risk.medium",
  "colors.risk.high",
  "colors.risk.critical",
  "colors.mode.safe",
  "colors.mode.competitive",
  "colors.mode.lab",
  "colors.mode.blocked",
  "typography.fontFamily.ui",
  "typography.fontFamily.metric",
  "typography.letterSpacing.default",
  "components.statusStrip.height",
  "components.actionBar.height"
];

for (const path of requiredPaths) {
  if (get(tokens, path) == null) {
    fail(`Missing required token path: ${path}`);
  }
}

const snapshotShape = {
  meta: tokens.meta,
  colors: {
    background: tokens.colors.background,
    surface: tokens.colors.surface,
    text: tokens.colors.text,
    accent: tokens.colors.accent,
    status: tokens.colors.status,
    risk: tokens.colors.risk,
    mode: tokens.colors.mode
  },
  typography: {
    fontFamily: tokens.typography.fontFamily,
    fontSize: tokens.typography.fontSize,
    lineHeight: tokens.typography.lineHeight,
    letterSpacing: tokens.typography.letterSpacing
  },
  radius: tokens.radius,
  components: tokens.components
};

const stable = (value) => JSON.stringify(value, Object.keys(value).sort(), 2);

if (JSON.stringify(snapshotShape, null, 2) !== JSON.stringify(snapshot, null, 2)) {
  fail("Token snapshot is stale. Update tests/token.snapshot.json with the intentional token contract.");
}

if (!themeCss.includes("@theme")) {
  fail("styles/theme.css must expose Tailwind v4 @theme variables.");
}

const requiredCss = [
  "--color-liiiraa-bg-app: #0b0f14;",
  "--color-liiiraa-surface-panel: #151d26;",
  "--color-liiiraa-text-primary: #f5f8fb;",
  "--color-liiiraa-telemetry: #27d7ff;",
  "--color-liiiraa-performance: #3af28f;",
  "--color-liiiraa-warning: #ffbd5a;",
  "--color-liiiraa-danger: #ff5a67;",
  "--radius-liiiraa-card: 0.5rem;"
];

for (const cssToken of requiredCss) {
  if (!themeCss.includes(cssToken)) {
    fail(`Missing required CSS token: ${cssToken}`);
  }
}

const parseRem = (value) => {
  if (value.endsWith("rem")) return Number.parseFloat(value) * 16;
  if (value.endsWith("px")) return Number.parseFloat(value);
  return Number.parseFloat(value);
};

if (parseRem(tokens.radius.card) > 8) {
  fail("Card radius must stay at or below 8px.");
}

for (const [name, value] of Object.entries(tokens.typography.letterSpacing)) {
  if (Number.parseFloat(value) < 0) {
    fail(`Letter spacing must not be negative: ${name}`);
  }
}

const hexToRgb = (hex) => {
  const value = hex.replace("#", "");
  return [0, 2, 4].map((start) => Number.parseInt(value.slice(start, start + 2), 16) / 255);
};

const linearize = (channel) =>
  channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;

const luminance = (hex) => {
  const [red, green, blue] = hexToRgb(hex).map(linearize);
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
};

const contrast = (foreground, background) => {
  const light = Math.max(luminance(foreground), luminance(background));
  const dark = Math.min(luminance(foreground), luminance(background));
  return (light + 0.05) / (dark + 0.05);
};

const contrastPairs = [
  ["text.primary", tokens.colors.text.primary, tokens.colors.background.app],
  ["text.secondary", tokens.colors.text.secondary, tokens.colors.surface.panel],
  ["status.success", tokens.colors.status.success, tokens.colors.background.app],
  ["status.warning", tokens.colors.status.warning, tokens.colors.background.app],
  ["status.danger", tokens.colors.status.danger, tokens.colors.background.app]
];

for (const [name, foreground, background] of contrastPairs) {
  const ratio = contrast(foreground, background);
  if (ratio < 4.5) {
    fail(`${name} contrast is ${ratio.toFixed(2)} and must be at least 4.5.`);
  }
}

console.log(`Liiiraa token snapshot passed (${stable(tokens.meta).replace(/\n/g, " ")}).`);
