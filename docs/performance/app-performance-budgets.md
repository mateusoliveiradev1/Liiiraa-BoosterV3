# App Performance Budgets

Liiiraa Booster must stay lighter than the work it performs. These budgets make the OpenSpec performance plan executable for startup, idle resource use, scan responsiveness, UI responsiveness, and benchmark overhead.

## Budget Table

| Area | Budget | Smoke behavior |
| --- | --- | --- |
| Cold startup | Dashboard usable in 2500 ms or less | Reads desktop shell entry files as the current startup proxy until a packaged Tauri launch measurement exists. |
| Warm startup | Dashboard usable in 1000 ms or less | Repeats the shell entry read as a warm-cache proxy. |
| Idle CPU | Under 1 percent after settle | Samples the smoke process after startup work settles and reports a warning if it exceeds the target. |
| Idle memory | UI process under 250 MB where practical | Uses smoke process RSS as a conservative local proxy and reports a warning if it exceeds the target. |
| UI responsiveness | No main-thread task over 100 ms | Measures event-loop delay while benchmark parsing runs in a worker. |
| Scan time | First scan progress within 250 ms | Verifies simulated scan work emits progress on the next event-loop turn. |
| Scan cancellation | Cancellation acknowledged within 250 ms | Verifies cancellable scan work stops promptly. |
| Benchmark overhead | Large capture parsing stays off the UI thread | Parses 25000 synthetic PresentMon rows in a worker, requires at least 10000 rows/s, and keeps main-thread delay under 100 ms. |

## Running The Smoke

Run the smoke directly from the repository root:

```powershell
node scripts/check-performance-budgets.mjs
```

Use JSON output for CI log ingestion:

```powershell
node scripts/check-performance-budgets.mjs --json
```

The default mode fails deterministic budget violations and reports warnings for host-sensitive idle CPU or memory measurements. `--strict` upgrades warnings to failures when a controlled performance runner is available.

## Replacement Path

The script is intentionally dependency-free so T113 can run before a complete desktop packaging pipeline exists. As soon as packaged Tauri startup telemetry is available, replace the startup proxy with real cold and warm launch timings while keeping the same budget keys:

- `coldDashboardUsableMs`
- `warmDashboardUsableMs`
- `idleCpuPercent`
- `idleMemoryMb`
- `uiLongTaskMs`
- `scanProgressVisibleMs`
- `scanCancellationMs`
- `benchmarkParserMs`
- `benchmarkRowsPerSecond`
- `benchmarkMainThreadDelayMs`
