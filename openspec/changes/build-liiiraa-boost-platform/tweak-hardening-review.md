# Tweak Hardening Review

This is the final pre-implementation pass for V1 tweak quality. It explains the extra guardrails added after deeper research.

## Decisions
- Treat every tweak as a typed product feature, not a script line.
- Add applicability rules for Windows build, hardware vendor, driver support, power source, display state, game state, and anti-cheat state.
- Add conflict validation before apply. Many "FPS tweaks" are only safe when they do not conflict with another latency, sync, overlay, color, or driver setting.
- Keep one-click Safe mode conservative. Competitive and Lab exist for real tradeoffs.
- Make blocked guardrails first-class tests. Blocking bad tweak folklore is part of the product.

## New Hardening Areas

### Display and Present Path
- Detect refresh rate, VRR, HDR, ICC/color profiles, overlays, and capture state before touching GameDVR, FSO, HAGS, or driver sync settings.
- Warn users that GameDVR/FSO changes can affect color-management behavior in exclusive fullscreen on some systems.
- Benchmark fullscreen vs borderless vs windowed optimizations instead of forcing one universal path.

### CPU Intel and AMD
- Detect Intel hybrid topology, Thread Director readiness, Intel APO/DTT readiness, AMD chipset driver status, AMD CPPC/preferred-core hints, and AMD X3D scheduling readiness.
- Keep automatic OC/undervolt, SMT disable, E-core disable, CPU mitigation disable, realtime priority, and hard affinity out of default optimization.
- Use Windows Processor Power Management only inside Liiiraa-owned power plans, with backup and laptop/power-source rules.

### GPU NVIDIA and AMD
- Validate conflicts between Reflex, Low Latency Mode, frame caps, VRR/G-SYNC, V-SYNC, frame generation, and in-game limiters.
- Validate conflicts between HYPR-RX, Anti-Lag, Boost, Chill, Enhanced Sync, FreeSync, AFMF, FRTC, RSR/RIS, and in-game caps.
- Treat ReBAR/SAM as detect/recommend/benchmark. Do not force hidden ReBAR or SAM behavior globally.
- Treat NVIDIA App auto tuning, Ryzen Master, PBO, Curve Optimizer, and any voltage/frequency tuning as Lab/advisory.

### Network
- Keep RSS/default offloads conservative.
- Make RSC, interrupt moderation, buffers, and offload changes Lab-only, adapter-specific, and benchmarked.
- Block Jumbo Frames for internet gaming.
- Do not sell DNS or TCP viral packs as FPS optimization.

### Storage
- Use Storage Sense, TRIM/Optimize-Volume, DirectStorage readiness, NTFS last access, and 8.3 behavior carefully.
- Block unsupported NVMe/server-driver registry hacks.
- Never delete game content folders as an optimization. Prefer official verify/repair.

### PUBG and Anti-Cheat
- Keep PUBG official guidance: remove legacy Steam launch options, verify files, keep drivers updated, collect crash reports, benchmark DX11 vs DX11 Enhanced.
- Keep BattlEye guardrails strict: no kernel debug/test-signing/driver-signature bypasses, no service/file tampering, no memory edits.

## V1 Acceptance Gate
Before a tweak can ship:
- It exists in `v1-tweak-matrix.md`.
- It satisfies `tweak-definition-standard.md`.
- It has source links and evidence level.
- It declares conflicts and side effects.
- It has a dry-run plan.
- It has backup and rollback.
- It has verification.
- It has negative tests for unsafe/default behavior.
- It has UI disclosure.
- It has benchmark criteria if the expected impact is performance-sensitive.
