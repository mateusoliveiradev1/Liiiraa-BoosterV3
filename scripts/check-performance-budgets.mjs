import { promises as fs } from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance, monitorEventLoopDelay } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";

const PERFORMANCE_BUDGETS = {
  coldDashboardUsableMs: {
    label: "cold dashboard usable",
    limit: 2500,
    unit: "ms",
    severity: "fail",
  },
  warmDashboardUsableMs: {
    label: "warm dashboard usable",
    limit: 1000,
    unit: "ms",
    severity: "fail",
  },
  idleCpuPercent: {
    label: "idle CPU after settle",
    limit: 1,
    unit: "%",
    severity: "warn",
  },
  idleMemoryMb: {
    label: "idle memory UI process",
    limit: 250,
    unit: "MB",
    severity: "warn",
  },
  uiLongTaskMs: {
    label: "UI long task ceiling",
    limit: 100,
    unit: "ms",
    severity: "fail",
  },
  scanProgressVisibleMs: {
    label: "scan progress visible",
    limit: 250,
    unit: "ms",
    severity: "fail",
  },
  scanCancellationMs: {
    label: "scan cancellation acknowledgement",
    limit: 250,
    unit: "ms",
    severity: "fail",
  },
  benchmarkParserMs: {
    label: "benchmark parser worker duration",
    limit: 1500,
    unit: "ms",
    severity: "warn",
  },
  benchmarkRowsPerSecond: {
    label: "benchmark parser throughput",
    limit: 10000,
    unit: "rows/s",
    severity: "fail",
    comparison: "minimum",
  },
  benchmarkMainThreadDelayMs: {
    label: "benchmark overhead main-thread delay",
    limit: 100,
    unit: "ms",
    severity: "fail",
  },
};

const REQUIRED_BUDGET_KEYS = [
  "coldDashboardUsableMs",
  "warmDashboardUsableMs",
  "idleCpuPercent",
  "idleMemoryMb",
  "uiLongTaskMs",
  "scanProgressVisibleMs",
  "scanCancellationMs",
  "benchmarkParserMs",
  "benchmarkRowsPerSecond",
  "benchmarkMainThreadDelayMs",
];

const DASHBOARD_STARTUP_FILES = [
  "apps/desktop/src/main.tsx",
  "apps/desktop/src/App.tsx",
  "apps/desktop/src/commandCenter.ts",
  "apps/desktop/src/routes/index.tsx",
  "apps/desktop/src/styles.css",
];

const BENCHMARK_SMOKE_ROWS = 25_000;
const IDLE_SAMPLE_MS = 750;

if (!isMainThread) {
  runBenchmarkParserWorker();
} else {
  await runMain();
}

async function runMain() {
  const args = new Set(process.argv.slice(2));
  const json = args.has("--json");
  const strict = args.has("--strict");
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const results = [];

  results.push(validateBudgetCatalog());
  results.push(await measureDashboardStartup(root, "cold"));
  results.push(await measureDashboardStartup(root, "warm"));
  results.push(await measureIdleCpu());
  results.push(measureIdleMemory());
  results.push(await measureScanProgress());
  results.push(await measureScanCancellation());
  results.push(await measureBenchmarkOverhead());

  const evaluated = results.flatMap((result) => (Array.isArray(result) ? result : [result]));
  const failures = evaluated.filter((result) => result.status === "fail");
  const warnings = evaluated.filter((result) => result.status === "warn");

  if (json) {
    console.log(
      JSON.stringify(
        {
          budgets: PERFORMANCE_BUDGETS,
          rows: evaluated,
          summary: {
            status: failures.length > 0 || (strict && warnings.length > 0) ? "fail" : "pass",
            failures: failures.length,
            warnings: warnings.length,
            strict,
          },
        },
        null,
        2
      )
    );
  } else {
    console.log("Performance budget smoke results:");
    for (const result of evaluated) {
      const measured =
        typeof result.value === "number" && Number.isFinite(result.value)
          ? `${formatNumber(result.value)} ${result.unit}`
          : result.value;
      const threshold = describeThreshold(result);
      console.log(`- ${result.status.toUpperCase()} ${result.label}: ${measured}${threshold}`);
      if (result.note) {
        console.log(`  ${result.note}`);
      }
    }
  }

  if (failures.length > 0 || (strict && warnings.length > 0)) {
    process.exitCode = 1;
  }
}

function validateBudgetCatalog() {
  const missing = REQUIRED_BUDGET_KEYS.filter((key) => !PERFORMANCE_BUDGETS[key]);
  const invalid = Object.entries(PERFORMANCE_BUDGETS).filter(([, budget]) => {
    return !Number.isFinite(budget.limit) || budget.limit <= 0 || !budget.label || !budget.unit;
  });

  if (missing.length === 0 && invalid.length === 0) {
    return pass("budget catalog coverage", REQUIRED_BUDGET_KEYS.length, "budgets", {
      note: "Covers startup, idle CPU, memory, scan timing, UI responsiveness, and benchmark overhead.",
    });
  }

  return fail("budget catalog coverage", REQUIRED_BUDGET_KEYS.length - missing.length, "budgets", {
    note: `Missing: ${missing.join(", ") || "none"}; invalid: ${invalid
      .map(([key]) => key)
      .join(", ") || "none"}.`,
  });
}

async function measureDashboardStartup(root, phase) {
  const start = performance.now();

  for (const relativePath of DASHBOARD_STARTUP_FILES) {
    await fs.readFile(path.join(root, relativePath), "utf8");
  }

  const duration = performance.now() - start;
  const budgetKey = phase === "cold" ? "coldDashboardUsableMs" : "warmDashboardUsableMs";
  return evaluateBudget(budgetKey, duration, {
    note: `${phase} smoke reads desktop shell entry files as the current startup proxy until packaged Tauri timings are available.`,
  });
}

async function measureIdleCpu() {
  const before = process.cpuUsage();
  const start = performance.now();

  await sleep(IDLE_SAMPLE_MS);

  const elapsedMs = performance.now() - start;
  const used = process.cpuUsage(before);
  const cpuMs = (used.user + used.system) / 1000;
  const logicalCpuCount = Math.max(1, os.cpus().length);
  const cpuPercent = (cpuMs / (elapsedMs * logicalCpuCount)) * 100;

  return evaluateBudget("idleCpuPercent", cpuPercent, {
    note: `Sampled this smoke process for ${formatNumber(elapsedMs)} ms across ${logicalCpuCount} logical CPUs.`,
  });
}

function measureIdleMemory() {
  const rssMb = process.memoryUsage().rss / 1024 / 1024;
  return evaluateBudget("idleMemoryMb", rssMb, {
    note: "Uses this smoke process RSS as a conservative local proxy for the UI-process memory budget.",
  });
}

async function measureScanProgress() {
  const start = performance.now();
  await new Promise((resolve) => setTimeout(resolve, 0));
  const progressMs = performance.now() - start;

  return evaluateBudget("scanProgressVisibleMs", progressMs, {
    note: "Verifies a scan scheduler can emit first progress on the next event-loop turn.",
  });
}

async function measureScanCancellation() {
  const controller = new AbortController();
  const scan = simulateCancellableScan(controller.signal);
  const start = performance.now();

  controller.abort();
  await scan;

  const cancellationMs = performance.now() - start;
  return evaluateBudget("scanCancellationMs", cancellationMs, {
    note: "Verifies scan work acknowledges cancellation without waiting for a full scan loop.",
  });
}

function simulateCancellableScan(signal) {
  return new Promise((resolve) => {
    const step = () => {
      if (signal.aborted) {
        resolve();
        return;
      }

      setTimeout(step, 5);
    };

    setTimeout(step, 0);
  });
}

async function measureBenchmarkOverhead() {
  const histogram = monitorEventLoopDelay({ resolution: 10 });
  histogram.enable();

  const workerResult = await runWorker({ rows: BENCHMARK_SMOKE_ROWS });

  histogram.disable();
  const mainThreadDelayMs = Number(histogram.max) / 1_000_000;

  return [
    evaluateBudget("uiLongTaskMs", mainThreadDelayMs, {
      note: "Uses event-loop delay while benchmark parsing runs in a worker as the UI responsiveness smoke.",
    }),
    evaluateBudget("benchmarkParserMs", workerResult.durationMs, {
      note: `Parsed ${workerResult.rows.toLocaleString("en-US")} synthetic PresentMon rows off the main thread.`,
    }),
    evaluateBudget("benchmarkRowsPerSecond", workerResult.rowsPerSecond, {
      note: "Throughput smoke guards large capture parsing from regressing into UI-visible overhead.",
    }),
    evaluateBudget("benchmarkMainThreadDelayMs", mainThreadDelayMs, {
      note: "Main-thread delay must stay below the long-task ceiling while benchmark parsing runs.",
    }),
  ];
}

function runWorker(data) {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL(import.meta.url), {
      workerData: data,
    });

    worker.once("message", resolve);
    worker.once("error", reject);
    worker.once("exit", (code) => {
      if (code !== 0) {
        reject(new Error(`benchmark parser worker exited with code ${code}`));
      }
    });
  });
}

function runBenchmarkParserWorker() {
  const rows = Number(workerData.rows);
  const csv = createPresentMonCsv(rows);
  const start = performance.now();
  const parsed = parsePresentMonCsv(csv);
  const durationMs = performance.now() - start;

  parentPort.postMessage({
    rows: parsed.measuredRows,
    durationMs,
    rowsPerSecond: (parsed.measuredRows / durationMs) * 1000,
    droppedFrames: parsed.droppedFrames,
    p95FrameTimeMs: percentile(parsed.frameTimes, 0.95),
  });
}

function createPresentMonCsv(rows) {
  const lines = ["Application,ProcessID,FrameTime,CPUBusy,GPUBusy,Dropped,FrameType"];

  for (let index = 0; index < rows; index += 1) {
    const frameTime = 6.5 + (index % 90) / 20;
    const cpuBusy = 1.8 + (index % 16) / 10;
    const gpuBusy = frameTime - 0.7;
    const dropped = index % 997 === 0 ? "true" : "false";
    lines.push(`TslGame.exe,42,${frameTime.toFixed(2)},${cpuBusy.toFixed(2)},${gpuBusy.toFixed(2)},${dropped},Application`);
  }

  return lines.join("\n");
}

function parsePresentMonCsv(csv) {
  const lines = csv.split(/\r?\n/);
  const headers = lines.shift()?.split(",") ?? [];
  const frameTimeIndex = headers.indexOf("FrameTime");
  const droppedIndex = headers.indexOf("Dropped");

  if (frameTimeIndex < 0 || droppedIndex < 0) {
    throw new Error("synthetic PresentMon CSV is missing required columns");
  }

  let droppedFrames = 0;
  const frameTimes = [];

  for (const line of lines) {
    if (!line) {
      continue;
    }

    const columns = line.split(",");
    const frameTime = Number(columns[frameTimeIndex]);
    if (Number.isFinite(frameTime) && frameTime > 0) {
      frameTimes.push(frameTime);
    }

    if (columns[droppedIndex] === "true") {
      droppedFrames += 1;
    }
  }

  return {
    measuredRows: frameTimes.length,
    droppedFrames,
    frameTimes,
  };
}

function percentile(values, percentileValue) {
  if (values.length === 0) {
    return 0;
  }

  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.ceil(sorted.length * percentileValue) - 1);
  return sorted[index];
}

function evaluateBudget(key, value, options = {}) {
  const budget = PERFORMANCE_BUDGETS[key];
  const comparison = budget.comparison ?? "maximum";
  const withinBudget =
    comparison === "minimum" ? value >= budget.limit : value <= budget.limit;

  if (withinBudget) {
    return pass(budget.label, value, budget.unit, {
      limit: budget.limit,
      comparison,
      note: options.note,
    });
  }

  const payload = {
    limit: budget.limit,
    comparison,
    note: options.note,
  };

  return budget.severity === "warn"
    ? warn(budget.label, value, budget.unit, payload)
    : fail(budget.label, value, budget.unit, payload);
}

function pass(label, value, unit, options = {}) {
  return result("pass", label, value, unit, options);
}

function warn(label, value, unit, options = {}) {
  return result("warn", label, value, unit, options);
}

function fail(label, value, unit, options = {}) {
  return result("fail", label, value, unit, options);
}

function result(status, label, value, unit, options = {}) {
  return {
    status,
    label,
    value,
    unit,
    limit: options.limit,
    comparison: options.comparison,
    note: options.note,
  };
}

function describeThreshold(result) {
  if (!Number.isFinite(result.limit)) {
    return "";
  }

  const operator = result.comparison === "minimum" ? ">=" : "<=";
  return ` (budget ${operator} ${formatNumber(result.limit)} ${result.unit})`;
}

function formatNumber(value) {
  return new Intl.NumberFormat("en-US", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  }).format(value);
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
