# Performance Max Plan

The optimizer must be lighter than the problems it fixes.

## Budgets
- cold usable dashboard: target under 2.5s on mid-range Windows gaming PC
- warm dashboard: target under 1.0s
- idle CPU after settle: under 1 percent
- idle memory UI process: target under 250 MB
- no UI task blocks main thread for more than 100 ms
- full scan: progress visible within 250 ms
- benchmark parser: handles large captures off UI thread
- cloud sync: no blocking local optimize/rollback

## Desktop Performance Rules
- Lazy-load NVIDIA/PUBG/benchmark modules.
- Batch Windows reads.
- Cache scan results with invalidation.
- Use progress events for long Rust operations.
- Virtualize long lists.
- Downsample charts.
- Avoid high-frequency React state updates.
- Do not animate during benchmark capture beyond minimal status indicator.
- Do not poll continuously while idle.

## Benchmark Overhead Rules
- Capture only when user starts benchmark or game session tracking.
- Show capture overhead disclaimer.
- Stop capture cleanly.
- Never leave helper processes running silently.
- Record app version and profile version with benchmark results.

## Performance Tests
- startup smoke
- idle CPU sampling
- large tweak list render
- large benchmark chart render
- PresentMon CSV parse throughput
- scan cancellation
- offline mode responsiveness
