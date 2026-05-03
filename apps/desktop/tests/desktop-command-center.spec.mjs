import { expect, test } from "@playwright/test";

const routeStates = [
  {
    id: "dashboard",
    label: "Dashboard",
    expectations: [/Live system profile/i, /Current bottleneck/i, /Next action/i, /Rollback/i, /Trust/i]
  },
  {
    id: "scan",
    label: "Smart Scan",
    expectations: [/Safe PC checkup/i, /Choose checks/i, /Graphics check/i, /Recommendations/i, /Open Smart Boost/i]
  },
  {
    id: "optimize",
    label: "Smart Boost",
    expectations: [/Safe Boost control/i, /^Safe$/i, /^Competitive$/i, /^Lab$/i, /^Blocked$/i, /Recovery availability/i]
  },
  {
    id: "power",
    label: "Power",
    expectations: [/Liiiraa power plan control/i, /Plan ladder/i, /Desktop and laptop policy/i, /Liiiraa Boost - Balanced/i]
  },
  {
    id: "nvidia",
    label: "GPU Control",
    expectations: [/Profile safety and PUBG readiness/i, /Profile states/i, /Refresh and cap logic/i, /Safety policy/i]
  },
  {
    id: "pubg",
    label: "Game Mode",
    expectations: [/Competitive checklist and anti-cheat boundary/i, /Detection/i, /BattlEye/i, /DirectX benchmark choice/i]
  },
  {
    id: "benchmarks",
    label: "Performance",
    expectations: [/Before and after proof/i, /Average and low comparison/i, /0\.1% low/i, /p95 frame/i, /Run metadata/i]
  },
  {
    id: "rollback",
    label: "Recovery",
    expectations: [/Session recovery timeline/i, /Safe gaming baseline/i, /Restore Balanced/i, /GPU profile rollback/i]
  },
  {
    id: "settings",
    label: "Settings",
    expectations: [
      /Privacy, updates, and trust/i,
      /Signed by Liiiraa/i,
      /Privacy and telemetry/i,
      /Live resource monitor/i,
      /Signing and update trust/i
    ]
  }
];

const localeVisualCases = [
  {
    actions: [/Iniciar Smart Scan/i, /Cancelar Smart Scan/i],
    button: /Aplicar Safe Boost/i,
    locale: "pt-BR",
    nav: /Smart Boost/i,
    status: /Atualizador/i,
    tableHeaders: [/Alteracao/i, /Impacto/i, /Confianca/i, /Risco/i, /Reversao/i]
  },
  {
    actions: [/Iniciar Smart Scan/i, /Cancelar Smart Scan/i],
    button: /Aplicar Safe Boost/i,
    locale: "es-ES",
    nav: /Smart Boost/i,
    status: /Actualizador/i,
    tableHeaders: [/Cambio/i, /Impacto/i, /Confianza/i, /Riesgo/i, /Reversion/i]
  }
];

const screenshotViewports = [
  { name: "tauri-default", width: 1280, height: 800 },
  { name: "desktop-wide", width: 1440, height: 900 },
  { name: "desktop-minimum", width: 1024, height: 680 }
];

test("desktop command center visual smoke covers navigation and critical optimizer states", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");

  await expect(page.getByLabel("Primary")).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Desktop sections" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Runtime status" })).toContainText("Updater");
  await expect(page.getByRole("region", { name: "Runtime status" })).toContainText("Backups");
  await expect(page.locator("body")).not.toContainText("Join waitlist");
  await expect(page.locator(".app-shell")).toHaveAttribute("data-sidebar-collapsed", "true");
  await expect(page.locator(".brand__mark")).toBeVisible();

  await page.getByLabel("Primary").hover();
  await expect(page.locator(".app-shell")).toHaveAttribute("data-sidebar-collapsed", "false");
  await expect(page.locator(".brand__logo")).toBeVisible();

  await page.mouse.move(560, 240);
  await expect(page.locator(".app-shell")).toHaveAttribute("data-sidebar-collapsed", "true");

  await page.getByRole("button", { name: "Help" }).click();
  const infoDialog = page.getByRole("dialog", { name: "Desktop information" });
  await expect(infoDialog).toBeVisible();
  await expect(infoDialog).toContainText("Help and live data");
  await expect(infoDialog).toContainText("Browser preview");
  await expect(infoDialog).not.toContainText(/RTX 4070|7800X3D|NVIDIA 551\.86|\+11\.8%/);
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "dashboard");
  await page.getByRole("button", { name: "Close information" }).click();
  await expect(page.getByRole("dialog", { name: "Desktop information" })).toBeHidden();

  for (const route of routeStates) {
    await openRoute(page, route.label, route.id);
    await expectRouteOptimizerState(page, route);
  }

  await openRoute(page, "Smart Boost", "optimize");
  await expect(page.getByRole("heading", { name: "Safe Boost control" })).toBeVisible();
  await expect(page.getByRole("heading", { exact: true, name: "Safe" })).toBeVisible();
  await expect(page.getByRole("heading", { exact: true, name: "Competitive" })).toBeVisible();
  await expect(page.getByRole("heading", { exact: true, name: "Lab" })).toBeVisible();
  await expect(page.getByRole("heading", { exact: true, name: "Blocked" })).toBeVisible();
  await expect(page.getByText("Ready to apply").first()).toBeVisible();
  await expect(page.getByText("Review required").first()).toBeVisible();
  await expect(page.getByText("Blocked from apply").first()).toBeVisible();
  await expect(page.getByText("Recovery availability")).toBeVisible();

  await openRoute(page, "Recovery", "rollback");
  await expect(page.getByRole("heading", { name: "Session recovery timeline" })).toBeVisible();
  await expect(page.getByText("Safe gaming baseline").first()).toBeVisible();
  await expect(page.getByText("Restore Balanced")).toBeVisible();
  await expect(page.getByRole("button", { name: "Restore GPU profiles" }).first()).toBeVisible();

  await openRoute(page, "Settings", "settings");
  await expect(page.getByRole("heading", { name: "Privacy, updates, and trust" })).toBeVisible();
  await expect(page.getByText("Signed by Liiiraa").first()).toBeVisible();
  await expect(page.getByRole("heading", { name: "Privacy and telemetry" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Consent gates" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Signing and update trust" })).toBeVisible();
  await expect(page.getByText("Live resource monitor").first()).toBeVisible();
  await expect(page.getByRole("button", { name: "Turn off" })).toBeVisible();
  await page.getByRole("button", { name: "Turn off" }).click();
  await expect(page.getByRole("button", { name: "Turn on" })).toBeVisible();
  await expect(page.getByText("Performance telemetry").first()).toBeVisible();
  await expect(page.getByText("Signature verified").first()).toBeVisible();
});

test("dashboard actions open active tweak workflows", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#dashboard");

  await expect(page.getByRole("heading", { name: "All available tweaks on the dashboard" })).toBeVisible();
  await expect(page.locator(".dashboard-tweak-matrix")).toContainText("Pause unused background recording");
  await expect(page.locator(".dashboard-tweak-matrix")).toContainText("Test adapter-specific RSC");
  await expect(page.locator(".dashboard-tweak-matrix")).toContainText("Keep Defender protection on");

  await page.getByRole("button", { name: "Run Smart Boost" }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "optimize");
  await expect(page.getByRole("status")).toContainText(/Safe tweaks|Safe boost|Smart Boost/i);

  await page.goto("/#dashboard");
  await page.getByRole("button", { name: "Review tweak plan" }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "optimize");

  await page.goto("/#dashboard");
  await page.getByRole("button", { name: /Continue smart scan/i }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "scan");

  await page.goto("/#dashboard");
  await page.getByRole("button", { name: /Open recovery/i }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "rollback");

  await page.goto("/#dashboard");
  await page.getByRole("button", { name: /^Benchmark:/i }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "benchmarks");
});

test("smart scan completes and unlocks the tweak plan", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#scan");

  const scanWorkflow = page.getByLabel("Scan workflow");

  await expect(page.locator("main.command-center")).toHaveAttribute("id", "scan");
  await expect(scanWorkflow.getByRole("heading", { name: "Safe PC checkup" })).toBeVisible();
  await expect(scanWorkflow.getByRole("heading", { name: "Smart Boost preview" })).toBeVisible();
  await expect(scanWorkflow.getByRole("button", { name: "Open Smart Boost" })).toBeDisabled();

  await scanWorkflow.getByRole("button", { name: "Start Smart Scan" }).click();
  await expect(scanWorkflow.getByRole("button", { name: "Cancel Smart Scan" })).toBeEnabled();
  await expect(scanWorkflow.locator(".smart-scan-ring")).toHaveAttribute("aria-valuenow", "100", { timeout: 8000 });
  await expect(scanWorkflow.getByRole("button", { name: "Open Smart Boost" })).toBeEnabled();

  await scanWorkflow.getByRole("button", { name: "Open Smart Boost" }).click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", "optimize");
  await expect(page.getByRole("heading", { name: "Safe Boost control" })).toBeVisible();
});

for (const viewport of screenshotViewports) {
  test(`desktop screenshot gate reports overflow and optimizer state at ${viewport.name}`, async ({
    page
  }, testInfo) => {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });

    for (const route of routeStates) {
      await page.goto(`/#${route.id}`);
      await expect(page.locator("main.command-center")).toHaveAttribute("id", route.id);
      await expect(page.locator(".app-shell")).toBeVisible();
      await expect(page.locator("body")).not.toContainText("Join waitlist");

      await expectRouteOptimizerState(page, route);
      await expectNonBlankPrimarySurface(page);
      await expectNoMarketingHeroRegression(page);
      await expectNoHorizontalOverflow(page);
      await expectNoVisibleTextClipping(page);
      await expectNoPrimaryControlOverflow(page);
      await expectNoSignificantTextOverlap(page);

      await page.screenshot({
        animations: "disabled",
        fullPage: true,
        path: testInfo.outputPath(`${viewport.name}-${route.id}.png`)
      });
    }
  });
}

for (const visualCase of localeVisualCases) {
  test(`desktop locale visual fit holds compact controls for ${visualCase.locale}`, async ({
    page
  }, testInfo) => {
    await page.setViewportSize({ width: 1024, height: 680 });

    await page.goto(`/?locale=${visualCase.locale}#scan`);
    await expect(page.locator(".app-shell")).toHaveAttribute("data-locale", visualCase.locale);
    await expect(page.locator("html")).toHaveAttribute("lang", visualCase.locale);
    await expect(page.locator(".nav-list")).toContainText(visualCase.nav);
    await expect(page.locator(".status-strip")).toContainText(visualCase.status);

    for (const action of visualCase.actions) {
      await expect(page.getByRole("button", { name: action }).first()).toBeVisible();
    }

    await expectNoHorizontalOverflow(page);
    await expectNoVisibleTextClipping(page);
    await expectNoPrimaryControlOverflow(page);
    await expectNoSignificantTextOverlap(page);

    await page.goto(`/?locale=${visualCase.locale}#optimize`);
    await expect(page.locator(".app-shell")).toHaveAttribute("data-locale", visualCase.locale);
    await expect(page.locator(".nav-list")).toContainText(visualCase.nav);
    await expect(page.getByRole("button", { name: visualCase.button }).first()).toBeVisible();

    for (const header of visualCase.tableHeaders) {
      await expect(page.locator(".tweak-ledger").first()).toContainText(header);
    }

    await expect(page.locator(".risk-badge").first()).toBeVisible();
    await expect(page.locator(".mode-segmented")).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await expectNoVisibleTextClipping(page);
    await expectNoPrimaryControlOverflow(page);
    await expectNoSignificantTextOverlap(page);

    await page.screenshot({
      animations: "disabled",
      fullPage: true,
      path: testInfo.outputPath(`${visualCase.locale}-optimize-locale-fit.png`)
    });
  });
}

async function openRoute(page, label, expectedId) {
  await page
    .getByRole("navigation", { name: "Desktop sections" })
    .getByRole("button", { name: new RegExp(`\\b${escapeRegExp(label)}\\b`) })
    .click();
  await expect(page.locator("main.command-center")).toHaveAttribute("id", expectedId);
}

async function expectRouteOptimizerState(page, route) {
  const workspace = page.locator("main.command-center");

  for (const expectedText of route.expectations) {
    await expect(workspace.getByText(expectedText).first()).toBeVisible();
  }
}

async function expectNonBlankPrimarySurface(page) {
  const surface = await page.evaluate(() => {
    const main = document.querySelector("main.command-center");

    if (!main) {
      return null;
    }

    const rect = main.getBoundingClientRect();
    const visibleBlocks = [...main.querySelectorAll("section,article,button,[role='table'],[role='img']")].filter(
      (element) => {
        const blockRect = element.getBoundingClientRect();
        const style = getComputedStyle(element);

        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          blockRect.width > 10 &&
          blockRect.height > 10
        );
      }
    );

    return {
      height: Math.round(rect.height),
      textLength: main.textContent.trim().length,
      visibleBlocks: visibleBlocks.length,
      width: Math.round(rect.width)
    };
  });

  expect(surface).not.toBeNull();
  expect(surface.width).toBeGreaterThan(300);
  expect(surface.height).toBeGreaterThan(240);
  expect(surface.textLength).toBeGreaterThan(160);
  expect(surface.visibleBlocks).toBeGreaterThan(3);
}

async function expectNoMarketingHeroRegression(page) {
  const main = page.locator("main.command-center");

  await expect(main).not.toContainText(/join waitlist|welcome to|ready to boost/i);

  const heroLike = await page.evaluate(() =>
    Boolean(document.querySelector("main.command-center .hero, main.command-center [class*='hero']"))
  );

  expect(heroLike).toBe(false);
}

async function expectNoHorizontalOverflow(page) {
  const overflow = await page.evaluate(() => {
    const clipToVisibleRect = (element, rect) => {
      const clipped = {
        bottom: rect.bottom,
        left: rect.left,
        right: rect.right,
        top: rect.top
      };
      let parent = element.parentElement;

      while (parent && parent !== document.documentElement) {
        const style = getComputedStyle(parent);
        const clipsOverflow = [style.overflow, style.overflowX, style.overflowY].some((value) =>
          /auto|clip|hidden|scroll/.test(value)
        );

        if (clipsOverflow) {
          const parentRect = parent.getBoundingClientRect();

          clipped.left = Math.max(clipped.left, parentRect.left);
          clipped.right = Math.min(clipped.right, parentRect.right);
          clipped.top = Math.max(clipped.top, parentRect.top);
          clipped.bottom = Math.min(clipped.bottom, parentRect.bottom);
        }

        parent = parent.parentElement;
      }

      return {
        bottom: clipped.bottom,
        height: Math.max(0, clipped.bottom - clipped.top),
        left: clipped.left,
        right: clipped.right,
        top: clipped.top,
        width: Math.max(0, clipped.right - clipped.left)
      };
    };
    const describe = (element) => {
      const rect = element.getBoundingClientRect();
      const clipped = clipToVisibleRect(element, rect);

      return {
        className:
          typeof element.className === "string"
            ? element.className
            : element.getAttribute("class") ?? "",
        tag: element.tagName.toLowerCase(),
        text: element.textContent.trim().slice(0, 80),
        height: Math.round(clipped.height),
        left: Math.round(clipped.left),
        top: Math.round(clipped.top),
        width: Math.round(clipped.width)
      };
    };
    const viewportWidth = document.documentElement.clientWidth;
    const scrollWidth = Math.max(document.documentElement.scrollWidth, document.body.scrollWidth);
    const offenders = [...document.body.querySelectorAll("*")]
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        const clipped = clipToVisibleRect(element, rect);

        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          clipped.width > 0 &&
          clipped.height > 0 &&
          (clipped.left < -1 || clipped.right > viewportWidth + 1)
        );
      })
      .slice(0, 5)
      .map(describe);

    return {
      offenders,
      overflowX: scrollWidth - viewportWidth
    };
  });

  expect(overflow.overflowX).toBeLessThanOrEqual(1);
  expect(overflow.offenders).toEqual([]);
}

async function expectNoVisibleTextClipping(page) {
  const clipped = await page.evaluate(() => {
    const describe = (element) => {
      const rect = element.getBoundingClientRect();

      return {
        className:
          typeof element.className === "string"
            ? element.className
            : element.getAttribute("class") ?? "",
        tag: element.tagName.toLowerCase(),
        text: element.textContent.trim().slice(0, 80),
        height: Math.round(rect.height),
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        width: Math.round(rect.width)
      };
    };

    return [...document.body.querySelectorAll("h1,h2,h3,p,a,button,span,strong,dt,dd,small,label")]
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
          element.textContent.trim().length > 0 &&
          element.scrollWidth > element.clientWidth + 1
        );
      })
      .slice(0, 5)
      .map(describe);
  });

  expect(clipped).toEqual([]);
}

async function expectNoPrimaryControlOverflow(page) {
  const overflowingControls = await page.evaluate(() => {
    const describe = (element) => {
      const rect = element.getBoundingClientRect();

      return {
        className:
          typeof element.className === "string"
            ? element.className
            : element.getAttribute("class") ?? "",
        tag: element.tagName.toLowerCase(),
        text: element.textContent.trim().slice(0, 80),
        height: Math.round(rect.height),
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        width: Math.round(rect.width)
      };
    };

    return [...document.body.querySelectorAll("button,.pill,.status-item,.metric-tile")]
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();

        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          rect.width > 0 &&
          rect.height > 0 &&
          (element.scrollWidth > element.clientWidth + 1 || element.scrollHeight > element.clientHeight + 1)
        );
      })
      .slice(0, 5)
      .map(describe);
  });

  expect(overflowingControls).toEqual([]);
}

async function expectNoSignificantTextOverlap(page) {
  const overlaps = await page.evaluate(() => {
    const clipToVisibleRect = (element, rect) => {
      const clipped = {
        bottom: rect.bottom,
        left: rect.left,
        right: rect.right,
        top: rect.top
      };
      let parent = element.parentElement;

      while (parent && parent !== document.documentElement) {
        const style = getComputedStyle(parent);
        const clipsOverflow = [style.overflow, style.overflowX, style.overflowY].some((value) =>
          /auto|clip|hidden|scroll/.test(value)
        );

        if (clipsOverflow) {
          const parentRect = parent.getBoundingClientRect();

          clipped.left = Math.max(clipped.left, parentRect.left);
          clipped.right = Math.min(clipped.right, parentRect.right);
          clipped.top = Math.max(clipped.top, parentRect.top);
          clipped.bottom = Math.min(clipped.bottom, parentRect.bottom);
        }

        parent = parent.parentElement;
      }

      clipped.left = Math.max(clipped.left, 0);
      clipped.right = Math.min(clipped.right, document.documentElement.clientWidth);
      clipped.top = Math.max(clipped.top, 0);
      clipped.bottom = Math.min(clipped.bottom, document.documentElement.scrollHeight);

      return {
        bottom: clipped.bottom,
        height: Math.max(0, clipped.bottom - clipped.top),
        left: clipped.left,
        right: clipped.right,
        top: clipped.top,
        width: Math.max(0, clipped.right - clipped.left)
      };
    };
    const describe = (element) => {
      const rect = element.getBoundingClientRect();
      const clipped = clipToVisibleRect(element, rect);

      return {
        className:
          typeof element.className === "string"
            ? element.className
            : element.getAttribute("class") ?? "",
        height: Math.round(clipped.height),
        left: Math.round(clipped.left),
        tag: element.tagName.toLowerCase(),
        text: element.textContent.trim().slice(0, 80),
        top: Math.round(clipped.top),
        width: Math.round(clipped.width)
      };
    };
    const elements = [...document.body.querySelectorAll("h1,h2,h3,p,a,button,span,strong,dt,dd,small,label")]
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        const clipped = clipToVisibleRect(element, rect);

        return (
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          clipped.width > 1 &&
          clipped.height > 1 &&
          element.textContent.trim().length > 0
        );
      })
      .map((element) => ({
        element,
        rect: clipToVisibleRect(element, element.getBoundingClientRect())
      }));
    const collisions = [];

    for (let index = 0; index < elements.length; index += 1) {
      for (let otherIndex = index + 1; otherIndex < elements.length; otherIndex += 1) {
        const first = elements[index];
        const second = elements[otherIndex];

        if (
          first.element.contains(second.element) ||
          second.element.contains(first.element) ||
          first.element.parentElement === second.element.parentElement
        ) {
          continue;
        }

        const width = Math.min(first.rect.right, second.rect.right) - Math.max(first.rect.left, second.rect.left);
        const height = Math.min(first.rect.bottom, second.rect.bottom) - Math.max(first.rect.top, second.rect.top);

        if (width > 6 && height > 6) {
          collisions.push({
            first: describe(first.element),
            second: describe(second.element)
          });

          if (collisions.length >= 5) {
            return collisions;
          }
        }
      }
    }

    return collisions;
  });

  expect(overlaps).toEqual([]);
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
