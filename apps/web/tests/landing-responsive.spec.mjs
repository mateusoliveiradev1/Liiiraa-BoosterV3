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
    await expect(page.locator(".app-visual")).toBeVisible();
    if (viewport.name === "desktop") {
      await expect(page.getByText("Backup first")).toBeVisible();
      await expect(page.getByText("Benchmarked")).toBeVisible();
      await expect(page.getByText("PUBG ready")).toBeVisible();
    } else {
      await expect(page.getByText("Backup first")).toBeHidden();
      await expect(page.getByText("Benchmarked")).toBeHidden();
      await expect(page.getByText("PUBG ready")).toBeHidden();
    }
    await expect(page.getByRole("heading", { name: "Claims stay tied to measurement." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Competitive planning without anti-cheat shortcuts." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Trust surfaces are part of the product, not a footer afterthought." })).toBeVisible();
    await expect(page.getByRole("link", { name: "Join waitlist" })).toHaveAttribute("href", "./waitlist/");

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
