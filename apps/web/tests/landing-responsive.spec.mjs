import { dirname, resolve } from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const { expect, test } = loadPlaywrightTest();

const testDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(testDir, "..");
const landingUrl = pathToFileURL(resolve(webRoot, "index.html")).href;
const waitlistUrl = pathToFileURL(resolve(webRoot, "waitlist", "index.html")).href;

for (const viewport of [
  { name: "desktop", width: 1440, height: 1000 },
  { name: "mobile", width: 390, height: 844 }
]) {
  test(`landing page keeps product-first proof layout stable on ${viewport.name}`, async ({ page }) => {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.goto(landingUrl);

    await expect(page.getByRole("heading", { level: 1, name: "Liiiraa Booster" })).toBeVisible();
    await expect(page.locator(".product-preview")).toBeVisible();
    await expect(page.getByAltText(/Liiiraa Booster dashboard/)).toBeVisible();
    const heroStats = page.locator(".hero__stats");
    if (viewport.name === "desktop") {
      await expect(heroStats.getByText("Rollback first", { exact: true })).toBeVisible();
      await expect(heroStats.getByText("Example data", { exact: true })).toBeVisible();
      await expect(heroStats.getByText("PUBG preview", { exact: true })).toBeVisible();
    } else {
      await expect(heroStats.getByText("Rollback first", { exact: true })).toBeHidden();
      await expect(heroStats.getByText("Example data", { exact: true })).toBeHidden();
      await expect(heroStats.getByText("PUBG preview", { exact: true })).toBeHidden();
    }
    await expect(page.getByRole("heading", { name: "Separated lanes replace the old dense feature grid." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "PUBG is the first visible profile, with support labels kept honest." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Performance proof is labeled as example data until real capture ships." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Safety details are product surfaces, not footer fine print." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Clear answers before download is available." })).toBeVisible();
    await expect(page.getByRole("link", { name: "Join waitlist" }).first()).toHaveAttribute("href", "./waitlist/");
    await expect(page.getByText("Windows download pending")).toBeVisible();

    await expectNoHorizontalOverflow(page);
    await expectNoVisibleTextClipping(page);
  });
}

test("waitlist placeholder is a real reserved route instead of fake checkout", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(waitlistUrl);

  await expect(page.getByRole("heading", { level: 1, name: "Waitlist placeholder" })).toBeVisible();
  await expect(page.getByText("checkout, and account flows are intentionally reserved")).toBeVisible();
  await expect(page.getByRole("link", { name: "Back to landing" })).toHaveAttribute("href", "../");
  await expect(page.getByLabel("Email for launch notice")).toBeVisible();
  await expect(page.getByText("Download pending")).toBeVisible();

  await expectNoHorizontalOverflow(page);
  await expectNoVisibleTextClipping(page);
});

async function expectNoHorizontalOverflow(page) {
  const overflow = await page.evaluate(() => {
    const viewportWidth = document.documentElement.clientWidth;
    const scrollWidth = Math.max(document.documentElement.scrollWidth, document.body.scrollWidth);
    const offenders = [...document.body.querySelectorAll("*")]
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          rect.width > 0 &&
          rect.height > 0 &&
          (rect.left < -1 || rect.right > viewportWidth + 1)
        );
      })
      .slice(0, 5)
      .map((element) => {
        const rect = element.getBoundingClientRect();
        return {
          tag: element.tagName.toLowerCase(),
          className: element.className,
          text: element.textContent.trim().slice(0, 80),
          left: Math.round(rect.left),
          right: Math.round(rect.right),
          viewportWidth
        };
      });

    return {
      overflowX: scrollWidth - viewportWidth,
      offenders
    };
  });

  expect(overflow.overflowX).toBeLessThanOrEqual(1);
  expect(overflow.offenders).toEqual([]);
}

async function expectNoVisibleTextClipping(page) {
  const clipped = await page.evaluate(() =>
    [...document.body.querySelectorAll("h1,h2,h3,p,a,span,strong,dt,dd")]
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        const intentionallyClipped = style.overflow === "hidden" || style.textOverflow === "ellipsis";

        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          !intentionallyClipped &&
          rect.width > 0 &&
          rect.height > 0 &&
          element.scrollWidth > element.clientWidth + 1
        );
      })
      .slice(0, 5)
      .map((element) => ({
        tag: element.tagName.toLowerCase(),
        className: element.className,
        text: element.textContent.trim().slice(0, 80),
        scrollWidth: element.scrollWidth,
        clientWidth: element.clientWidth
      }))
  );

  expect(clipped).toEqual([]);
}

function loadPlaywrightTest() {
  const requireFromSpec = createRequire(import.meta.url);

  try {
    return requireFromSpec("@playwright/test");
  } catch (specError) {
    const cliEntry = process.argv[1];

    try {
      return createRequire(cliEntry)("@playwright/test");
    } catch (cliError) {
      cliError.message = `${cliError.message}\nSpec resolution failed first: ${specError.message}`;
      throw cliError;
    }
  }
}
