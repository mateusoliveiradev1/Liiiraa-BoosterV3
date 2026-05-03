import { createRequire } from "node:module";
import { optimizationWorkflow } from "../../../packages/ui/src/optimizationWorkflow.js";

const { expect, test } = loadPlaywrightTest();

test("covers scan -> plan -> apply simulation -> verify -> rollback simulation", async ({ page }) => {
  await page.setContent(renderWorkflowHarness(optimizationWorkflow), { waitUntil: "domcontentloaded" });

  await expect(page.getByRole("heading", { exact: true, name: "Smart Scan" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Open Smart Boost" })).toBeDisabled();
  await expect(page.getByText("Ready - Smart Scan is ready to check this PC")).toBeVisible();

  await page.getByRole("button", { name: "Start Smart Scan" }).click();
  await expect(page.getByRole("progressbar", { name: "Smart Scan progress" })).toHaveAttribute(
    "aria-valuenow",
    "100"
  );
  await expect(page.getByText("Complete - Smart Boost can open")).toBeVisible();

  await page.getByRole("button", { name: "Open Smart Boost" }).click();
  await expect(page.getByRole("heading", { name: "Smart Boost plan" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Safe lane" })).toContainText("Ready to apply");
  await expect(page.getByRole("region", { name: "Competitive lane" })).toContainText("Review required");
  await expect(page.getByRole("region", { name: "Lab lane" })).toContainText("Review required");
  await expect(page.getByRole("region", { name: "Blocked lane" })).toContainText("Blocked from apply");

  await page.getByRole("button", { name: "Apply Safe Boost" }).click();
  await expect(page.getByRole("heading", { name: "Safe Boost simulation" })).toBeVisible();
  await expect(page.getByTestId("step-backup")).toContainText("Backup");
  await expect(page.getByTestId("step-apply")).toContainText("Apply");
  await expect(page.getByTestId("step-verify")).toContainText("Verify");
  await expect(page.getByTestId("step-benchmark")).toContainText("Benchmark prompt");
  await expect(page.getByTestId("step-rollback")).toContainText("Rollback if needed");

  await page.getByRole("button", { name: "Run apply simulation" }).click();
  await expect(page.getByTestId("step-backup")).toContainText("complete");
  await expect(page.getByTestId("step-apply")).toContainText("complete");
  await expect(page.getByRole("button", { name: "Verify simulated changes" })).toBeEnabled();

  await page.getByRole("button", { name: "Verify simulated changes" }).click();
  await expect(page.getByText("Read-back verified for safe changes")).toBeVisible();
  await expect(page.getByRole("button", { name: "Open Recovery" })).toBeEnabled();

  await page.getByRole("button", { name: "Open Recovery" }).click();
  await expect(page.getByRole("heading", { name: "Recovery simulation" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Recovery timeline" })).toContainText("Safe gaming baseline");
  await expect(page.getByRole("region", { name: "Recovery timeline" })).toContainText("Restore Balanced");

  await page.getByRole("button", { name: /Restore all for Safe gaming baseline/i }).click();
  await expect(page.getByText("Rollback simulation complete")).toBeVisible();
  await expect(page.getByText("All changed values restored for the simulated safe session.")).toBeVisible();
});

function renderWorkflowHarness(workflow) {
  const safeGroup = workflow.optimize.groups.find((group) => group.id === "safe");
  const firstRollbackSession = workflow.rollback.sessions[0];

  return `<!doctype html>
  <html lang="en">
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <title>Liiiraa Booster E2E Harness</title>
      <style>
        * { box-sizing: border-box; }
        body {
          margin: 0;
          min-width: 320px;
          background: #0b0f14;
          color: #f5f8fb;
          font-family: Inter, "Segoe UI", system-ui, sans-serif;
        }
        main {
          display: grid;
          gap: 14px;
          width: min(100%, 1120px);
          margin: 0 auto;
          padding: 20px;
        }
        h1, h2, p { margin: 0; letter-spacing: 0; }
        h1 { font-size: 30px; line-height: 1.1; }
        button {
          min-height: 38px;
          padding: 0 12px;
          border: 1px solid #344252;
          border-radius: 8px;
          background: #202b37;
          color: #f5f8fb;
          font: inherit;
          font-weight: 750;
        }
        button.primary {
          background: #27d7ff;
          border-color: #27d7ff;
          color: #071015;
        }
        button:disabled { color: #7d8a99; opacity: 0.65; }
        .panel {
          display: grid;
          gap: 12px;
          padding: 16px;
          border: 1px solid #2a3541;
          border-radius: 8px;
          background: #151d26;
        }
        .actions, .steps { display: flex; flex-wrap: wrap; gap: 8px; }
        .groups {
          display: grid;
          gap: 10px;
          grid-template-columns: repeat(2, minmax(0, 1fr));
        }
        .group {
          display: grid;
          gap: 8px;
          padding: 12px;
          border: 1px solid #344252;
          border-top: 3px solid var(--tone, #9aa8b8);
          border-radius: 8px;
          background: #111820;
        }
        .group[data-tone="success"] { --tone: #3af28f; }
        .group[data-tone="warning"] { --tone: #ffbd5a; }
        .group[data-tone="lab"] { --tone: #9b7cff; }
        .group[data-tone="danger"] { --tone: #ff5a67; }
        .step, .row {
          display: grid;
          gap: 8px;
          grid-template-columns: minmax(180px, 1fr) minmax(140px, 0.7fr) minmax(180px, 1fr);
          padding: 10px 0;
          border-top: 1px solid #2a3541;
        }
        .step:first-child, .row:first-child { border-top: 0; }
        .muted { color: #b6c2cf; }
        progress {
          width: 100%;
          height: 12px;
          accent-color: #27d7ff;
        }
        [hidden] { display: none !important; }
        @media (max-width: 760px) {
          main { padding: 14px; }
          .groups, .step, .row { grid-template-columns: 1fr; }
        }
      </style>
    </head>
    <body data-stage="scan">
      <main>
        <header class="panel">
          <p class="muted">Liiiraa Booster critical workflow</p>
          <h1>Smart Scan, Smart Boost, verify, recover</h1>
          <nav class="actions" aria-label="Workflow simulation stages">
            <button type="button" data-stage-button="scan" aria-current="step">Scan</button>
            <button type="button" data-stage-button="plan">Smart Boost</button>
            <button type="button" data-stage-button="apply">Apply</button>
            <button type="button" data-stage-button="rollback">Recovery</button>
          </nav>
        </header>

        <section class="panel" data-panel="scan" aria-labelledby="scan-title">
          <h2 id="scan-title">Smart Scan</h2>
          <p id="scan-state">${escapeHtml(workflow.scan.states[0].label)} - ${escapeHtml(workflow.scan.states[0].detail)}</p>
          <progress aria-label="Smart Scan progress" id="scan-progress" max="100" value="0"></progress>
          <div class="actions">
            <button class="primary" id="start-scan" type="button">Start Smart Scan</button>
            <button id="generate-plan" type="button" disabled>Open Smart Boost</button>
          </div>
          ${renderFindings(workflow.scan.findings)}
        </section>

        <section class="panel" data-panel="plan" aria-labelledby="plan-title" hidden>
          <h2 id="plan-title">Smart Boost plan</h2>
          <p class="muted">Safe changes are the only default apply path; Competitive, Lab, and Blocked rows stay review-only.</p>
          <div class="groups">
            ${workflow.optimize.groups.map(renderPlanGroup).join("")}
          </div>
          <div class="actions">
            <button class="primary" id="apply-safe" type="button">${escapeHtml(workflow.optimize.actions[0].label)}</button>
            <button type="button" disabled>${escapeHtml(workflow.optimize.actions[1].label)}</button>
            <button type="button" disabled>${escapeHtml(workflow.optimize.actions[2].label)}</button>
          </div>
        </section>

        <section class="panel" data-panel="apply" aria-labelledby="apply-title" hidden>
          <h2 id="apply-title">Safe Boost simulation</h2>
          <p id="apply-status" class="muted">Ready to simulate ${escapeHtml(safeGroup.tweaks.length.toString())} safe changes with backup first.</p>
          <ol class="steps" aria-label="Apply safety steps">
            ${workflow.optimize.applySteps.map(renderApplyStep).join("")}
          </ol>
          <div class="actions">
            <button class="primary" id="run-apply" type="button">Run apply simulation</button>
            <button id="verify-changes" type="button" disabled>Verify simulated changes</button>
            <button id="open-rollback" type="button" disabled>Open Recovery</button>
          </div>
        </section>

        <section class="panel" data-panel="rollback" aria-labelledby="rollback-title" hidden>
          <h2 id="rollback-title">Recovery simulation</h2>
          <section aria-label="Recovery timeline">
            <h3>${escapeHtml(firstRollbackSession.label)}</h3>
            <p class="muted">${escapeHtml(firstRollbackSession.summary)}</p>
            ${firstRollbackSession.items.map(renderRollbackItem).join("")}
          </section>
          <button class="primary" id="restore-session" type="button">Restore all for ${escapeHtml(firstRollbackSession.label)}</button>
          <p id="rollback-status" class="muted">Awaiting rollback simulation.</p>
        </section>
      </main>
      <script>
        const panels = [...document.querySelectorAll("[data-panel]")];
        const stageButtons = [...document.querySelectorAll("[data-stage-button]")];
        const showStage = (stage) => {
          document.body.dataset.stage = stage;
          panels.forEach((panel) => { panel.hidden = panel.dataset.panel !== stage; });
          stageButtons.forEach((button) => {
            if (button.dataset.stageButton === stage) {
              button.setAttribute("aria-current", "step");
            } else {
              button.removeAttribute("aria-current");
            }
          });
        };

        document.querySelector("#start-scan").addEventListener("click", () => {
          document.querySelector("#scan-progress").value = 100;
          document.querySelector("#scan-progress").setAttribute("aria-valuenow", "100");
          document.querySelector("#scan-state").textContent = "Complete - Smart Boost can open";
          document.querySelector("#generate-plan").disabled = false;
        });

        document.querySelector("#generate-plan").addEventListener("click", () => showStage("plan"));
        document.querySelector("#apply-safe").addEventListener("click", () => showStage("apply"));
        document.querySelector("#run-apply").addEventListener("click", () => {
          document.querySelector('[data-testid="step-backup"] [data-step-state]').textContent = "complete";
          document.querySelector('[data-testid="step-apply"] [data-step-state]').textContent = "complete";
          document.querySelector('[data-testid="step-verify"] [data-step-state]').textContent = "ready";
          document.querySelector("#apply-status").textContent = "Backup complete; safe changes applied in simulation.";
          document.querySelector("#verify-changes").disabled = false;
        });

        document.querySelector("#verify-changes").addEventListener("click", () => {
          document.querySelector('[data-testid="step-verify"] [data-step-state]').textContent = "complete";
          document.querySelector('[data-testid="step-benchmark"] [data-step-state]').textContent = "prompted";
          document.querySelector('[data-testid="step-rollback"] [data-step-state]').textContent = "available";
          document.querySelector("#apply-status").textContent = "Read-back verified for safe changes";
          document.querySelector("#open-rollback").disabled = false;
        });

        document.querySelector("#open-rollback").addEventListener("click", () => showStage("rollback"));
        document.querySelector("#restore-session").addEventListener("click", () => {
          document.querySelector("#rollback-status").textContent =
            "Rollback simulation complete. All changed values restored for the simulated safe session.";
        });
      </script>
    </body>
  </html>`;
}

function renderFindings(findings) {
  return findings
    .map(
      (finding) => `
        <div class="row">
          <strong>${escapeHtml(finding.title)}</strong>
          <span>${escapeHtml(finding.group)}</span>
          <span>${escapeHtml(finding.risk)} risk - ${escapeHtml(finding.detail)}</span>
        </div>`
    )
    .join("");
}

function renderPlanGroup(group) {
  const applyState = group.applyEnabled ? "Ready to apply" : group.id === "blocked" ? "Blocked from apply" : "Review required";

  return `
    <section class="group" aria-label="${escapeHtml(group.label)} lane" data-tone="${escapeHtml(group.tone)}">
      <h3>${escapeHtml(group.label)}</h3>
      <p>${escapeHtml(group.summary)}</p>
      <strong>${escapeHtml(applyState)}</strong>
      ${group.tweaks
        .map(
          (tweak) => `
            <div class="row">
              <strong>${escapeHtml(tweak.change)}</strong>
              <span>${escapeHtml(tweak.risk)} risk</span>
              <span>${escapeHtml(tweak.rollback)}</span>
            </div>`
        )
        .join("")}
    </section>`;
}

function renderApplyStep(step) {
  return `
    <li class="step" data-testid="step-${escapeHtml(step.id)}">
      <strong>${escapeHtml(step.label)}</strong>
      <span data-step-state>${escapeHtml(step.state)}</span>
      <span>${escapeHtml(step.detail)}</span>
    </li>`;
}

function renderRollbackItem(item) {
  return `
    <div class="row">
      <strong>${escapeHtml(item.label)}</strong>
      <span>${escapeHtml(item.before)} -> ${escapeHtml(item.after)}</span>
      <span>${escapeHtml(item.rollback)}</span>
    </div>`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
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
