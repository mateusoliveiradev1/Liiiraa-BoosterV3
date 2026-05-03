import { access, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const root = new URL("..", import.meta.url);

const requiredFiles = [
  "index.html",
  "theme.generated.css",
  "styles.css",
  "waitlist/index.html",
  "public/favicon.svg",
  "public/logo.svg",
  "public/logo-mark.svg",
  "public/product-dashboard.png",
  "public/social-preview.png"
];

await Promise.all(requiredFiles.map((file) => access(new URL(file, root))));

const indexHtml = await readFile(new URL("index.html", root), "utf8");
const waitlistHtml = await readFile(new URL("waitlist/index.html", root), "utf8");
const themeCss = await readFile(new URL("theme.generated.css", root), "utf8");
const sharedTokens = JSON.parse(await readFile(new URL("../../packages/ui/tokens/liiiraa.tokens.json", root), "utf8"));

const requiredIndexMarkers = [
  "Liiiraa Booster",
  "product-dashboard.png",
  "Optimization modules",
  "Supported games",
  "benchmark",
  "rollback",
  "PUBG",
  "Windows download pending",
  "./waitlist/"
];

const missingMarkers = requiredIndexMarkers.filter((marker) => !indexHtml.includes(marker));

if (missingMarkers.length > 0) {
  throw new Error(`Missing landing page markers: ${missingMarkers.join(", ")}`);
}

if (!waitlistHtml.includes("../styles.css")) {
  throw new Error("Waitlist placeholder must reuse the landing page stylesheet.");
}

if (!indexHtml.includes("./theme.generated.css") || !waitlistHtml.includes("../theme.generated.css")) {
  throw new Error("Landing and waitlist pages must load the generated shared theme before app CSS.");
}

const requiredThemeTokens = [
  ["background app", `--liiiraa-color-background-app: ${sharedTokens.colors.background.app};`],
  ["surface premium", `--liiiraa-color-surface-premium: ${sharedTokens.colors.surface.premium};`],
  ["active state", `--liiiraa-color-active: ${sharedTokens.colors.status.active};`],
  ["success state", `--liiiraa-color-success: ${sharedTokens.colors.status.success};`],
  ["rollback state", `--liiiraa-color-rollback: ${sharedTokens.colors.status.rollback};`],
  ["button height", `--liiiraa-action-height-md: ${sharedTokens.components.button.height.md};`],
  ["card radius", `--liiiraa-radius-card: ${sharedTokens.radius.card};`]
];

const missingThemeTokens = requiredThemeTokens
  .filter(([, marker]) => !themeCss.includes(marker))
  .map(([name]) => name);

if (missingThemeTokens.length > 0) {
  throw new Error(`Generated web theme is out of sync with shared tokens: ${missingThemeTokens.join(", ")}`);
}

console.log(`Static web check passed for ${fileURLToPath(new URL("index.html", root))}`);
