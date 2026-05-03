# Tweak Catalog

This catalog is the first implementation guide. Each tweak must become a `TweakDefinition` before code is written.

V1 scope is locked by [v1-tweak-matrix.md](v1-tweak-matrix.md). If this catalog and the matrix disagree, the matrix wins. Any new tweak must first be added to the matrix with mode, precheck, backup, verify, rollback, source links, and anti-cheat notes.

Modes:
- Safe: allowed in default optimization.
- Competitive: explicit performance-first tradeoff.
- Lab: experimental or high-risk; explicit opt-in only.
- Blocked: never apply automatically.

## Core Windows Tweaks

### win.power.plan.create-liiiraa
- Mode: Safe
- Risk: Low
- Do: duplicate an existing Windows plan, create `Liiiraa Boost - Balanced`, `Liiiraa Boost - Performance`, and `Liiiraa Boost - Competitive`, store previous active scheme, verify active scheme after apply.
- Dont: delete user plans, overwrite unknown plans, apply laptop-aggressive values without warning.
- Verify: `powercfg /getactivescheme` and expected scheme GUID.
- Rollback: restore previous active scheme and optionally remove created Liiiraa plans.
- Sources: Microsoft PC performance docs, Atlas power scheme research.

### win.power.throttling.off
- Mode: Safe on desktop, Competitive on laptop
- Risk: Medium
- Do: disable Windows power throttling in performance plans.
- Dont: apply silently on battery-focused laptops.
- Verify: registry/power setting state and active plan.
- Rollback: restore previous value or remove created override.
- Sources: Atlas DisablePowerSaving research, Microsoft power mode docs.

### win.power.usb-selective-suspend.off
- Mode: Safe
- Risk: Low
- Do: disable USB selective suspend in Liiiraa power plans to reduce device sleep/wake latency.
- Dont: mutate all user power plans.
- Verify: `powercfg /query` for USB selective suspend.
- Rollback: restore original plan settings.
- Sources: Atlas power script research.

### win.power.pcie-link-state.off
- Mode: Safe on desktop, Competitive on laptop
- Risk: Medium
- Do: set PCIe Link State Power Management to off in performance plans.
- Dont: apply globally outside managed plans.
- Verify: `powercfg /query`.
- Rollback: restore backup values.
- Sources: Microsoft power settings, common gaming power tuning.

### win.power.storage-idle.reduce
- Mode: Competitive
- Risk: Medium
- Do: reduce or disable NVMe/storage idle timeout in Competitive plan.
- Dont: force on laptops without heat/power warning.
- Verify: `powercfg /query` storage subgroup.
- Rollback: restore previous values.
- Sources: Atlas power script research.

### win.network.disable-eee-green
- Mode: Competitive
- Risk: Medium
- Do: disable Energy Efficient Ethernet, Green Ethernet, and adapter power saving when adapter exposes supported properties.
- Dont: write unsupported vendor properties blindly.
- Verify: read adapter advanced properties after apply.
- Rollback: restore original advanced property values.
- Sources: Atlas power script research.

### win.gaming.game-dvr-capture.off
- Mode: Safe
- Risk: Low
- Do: disable Game DVR/background recording/capture features that consume resources.
- Dont: remove Xbox app components or break Game Bar globally when user uses it.
- Verify: registry policy and user setting state.
- Rollback: restore previous values.
- Sources: Atlas disable-game-bar research, Microsoft gaming docs.

### win.apps.startup-review
- Mode: Safe
- Risk: Low
- Do: list startup apps and recommend disabling high-impact nonessential entries.
- Dont: disable unknown security, driver, audio, anti-cheat, or peripheral software automatically.
- Verify: startup entry state.
- Rollback: restore entry state.
- Sources: Microsoft PC performance docs.

### win.apps.background-permissions
- Mode: Safe
- Risk: Low
- Do: identify apps with background permissions and recommend safe reductions.
- Dont: break notifications or user-selected sync apps without consent.
- Verify: app background permission state.
- Rollback: restore previous values.
- Sources: Microsoft PC performance docs.

### win.tasks.safe-pause
- Mode: Safe
- Risk: Medium
- Do: pause approved scheduled tasks during gaming sessions when they are known background noise.
- Dont: disable Windows Update, Defender, or system maintenance permanently by default.
- Verify: scheduled task state.
- Rollback: restore task state after session.
- Sources: ReviOS final task research; Microsoft safety posture.

### win.ntfs.last-access.disable
- Mode: Safe
- Risk: Low
- Do: disable last access timestamp updates.
- Dont: apply if enterprise/compliance software depends on access timestamps without warning.
- Verify: `fsutil behavior query disablelastaccess`.
- Rollback: restore previous fsutil setting.
- Sources: Atlas/ReviOS NTFS research, Microsoft fsutil docs.

### win.ntfs.8dot3.disable
- Mode: Safe
- Risk: Low/Medium
- Do: disable 8.3 name creation for future files when compatibility permits.
- Dont: apply if legacy software compatibility is detected or user opts out.
- Verify: `fsutil 8dot3name query`.
- Rollback: restore previous behavior setting.
- Sources: Atlas/ReviOS NTFS research, Microsoft fsutil docs.

### win.mmcss.system-responsiveness
- Mode: Competitive
- Risk: Medium
- Do: offer `SystemResponsiveness=10` as a benchmarked competitive tweak.
- Dont: assume universal FPS gain.
- Verify: registry value.
- Rollback: restore previous value.
- Sources: Atlas MMCSS research, Microsoft MMCSS docs.

### win.scheduler.foreground-boost
- Mode: Competitive
- Risk: Medium
- Do: offer `Win32PrioritySeparation=38` as a benchmarked foreground app tweak.
- Dont: apply in Safe mode by default.
- Verify: registry value.
- Rollback: restore previous value.
- Sources: Atlas/ReviOS scheduler research.

### win.security.vbs-hvci-performance-tradeoff
- Mode: Competitive
- Risk: High
- Do: detect VBS/HVCI/Memory Integrity and offer a clear performance-vs-security choice.
- Dont: silently disable security features.
- Verify: Win32_DeviceGuard and registry state after reboot.
- Rollback: restore previous settings and prompt reboot.
- Sources: Microsoft Memory Integrity docs, ReviOS VBS research.

### win.security.virtual-machine-platform-tradeoff
- Mode: Competitive
- Risk: High
- Do: detect VMP/Hyper-V related components and explain conflicts with WSL/VMs/security.
- Dont: disable if user uses WSL, Hyper-V, emulators, or development virtualization without explicit consent.
- Verify: Windows optional feature state.
- Rollback: restore feature state and prompt reboot.
- Sources: Microsoft gaming performance option research.

### win.timer-resolution.session
- Mode: Lab
- Risk: Medium
- Do: support a per-session timer resolution helper only while a game/profile is active.
- Dont: install permanent timer hacks or global scheduled tasks by default.
- Verify: helper process/session state and capture frametime delta.
- Rollback: stop helper and restore previous timer behavior.
- Sources: Atlas timer resolution research.

### win.memory-compression.toggle
- Mode: Lab
- Risk: Medium
- Do: expose memory compression state as a Lab experiment with benchmark evidence.
- Dont: disable blindly on low-RAM systems.
- Verify: `Get-MMAgent`.
- Rollback: restore previous MMAgent state.
- Sources: ReviOS Disable-MMAgent research.

### win.search-indexing.session-reduce
- Mode: Competitive
- Risk: Medium
- Do: pause or reduce indexing during game sessions if indexing is active and causing disk/CPU load.
- Dont: rename SearchApp or break Windows Search binaries.
- Verify: service/session state.
- Rollback: restore service state.
- Sources: imribiy Search Toggle caution; Microsoft performance docs.

### win.sysmain.conditional
- Mode: Lab
- Risk: Medium
- Do: detect HDD vs SSD and offer SysMain changes only conditionally.
- Dont: disable automatically on HDD or low-memory systems.
- Verify: service state and boot/game load benchmark.
- Rollback: restore service startup state.
- Sources: imribiy service script warnings.

## NVIDIA Tweaks

### nvidia.global.performance-profile
- Mode: Safe/Competitive
- Risk: Medium
- Do: create/apply `Liiiraa Boost - Global Performance` with documented settings where supported.
- Dont: import random `.nip` dumps or hidden flags as default.
- Verify: Driver Settings API/NPI readback.
- Rollback: restore backed-up customized profiles.
- Sources: NVIDIA Driver Settings API, NVIDIA Control Panel docs.

### nvidia.power.prefer-maximum-performance
- Mode: Competitive
- Risk: Medium
- Do: set power management to Prefer Maximum Performance for selected profile(s).
- Dont: force globally for laptop battery mode.
- Verify: profile setting readback.
- Rollback: restore previous setting.
- Sources: NVIDIA power management support article.

### nvidia.low-latency.on
- Mode: Competitive
- Risk: Medium
- Do: set Low Latency Mode to On where useful, especially DX11/non-Reflex paths.
- Dont: default to Ultra globally; do not apply while BattlEye/PUBG is running.
- Verify: profile setting readback and frametime benchmark.
- Rollback: restore previous setting.
- Sources: NVIDIA Manage 3D Settings, BattlEye FAQ note, NVIDIA Reflex docs.

### nvidia.max-frame-rate.vrr-cap
- Mode: Competitive
- Risk: Low
- Do: recommend caps below refresh rate for VRR/G-SYNC profiles.
- Dont: force cap globally; keep per-game/per-profile.
- Verify: profile setting and benchmark.
- Rollback: restore previous cap/off state.
- Sources: NVIDIA Control Panel docs, Blur Busters VRR guidance.

### nvidia.shader-cache.size
- Mode: Safe
- Risk: Low
- Do: keep shader cache enabled and use a reasonable cache size when driver exposes it.
- Dont: clear shader cache before every launch.
- Verify: setting readback.
- Rollback: restore previous value.
- Sources: NVIDIA profile settings research.

### nvidia.texture-filtering.performance
- Mode: Competitive
- Risk: Low
- Do: use Performance or High Performance profile for competitive presets.
- Dont: force on users who choose quality profile.
- Verify: setting readback.
- Rollback: restore previous value.
- Sources: NVIDIA Control Panel docs.

## PUBG Tweaks

### pubg.detect-installation
- Mode: Safe
- Risk: Low
- Do: detect Steam/Epic install and config paths.
- Dont: scan unrelated personal folders broadly.
- Verify: executable and config existence.
- Rollback: no mutation.
- Sources: PUBG support paths and Steam/Epic conventions.

### pubg.launch-options.remove-legacy
- Mode: Safe
- Risk: Low
- Do: detect and recommend removing old Steam launch options.
- Dont: add unsupported launch flags.
- Verify: launcher setting/manual confirmation.
- Rollback: store previous launch options if app modifies them.
- Sources: PUBG official performance guide.

### pubg.dx-mode.benchmark
- Mode: Safe
- Risk: Low
- Do: benchmark DX11 Enhanced vs DX11 and recommend per machine.
- Dont: force DX12 as default.
- Verify: config state and benchmark result.
- Rollback: restore previous DX mode.
- Sources: PUBG update 13.2/25.2 notes and support guide.

### pubg.graphics.competitive-checklist
- Mode: Safe
- Risk: Low
- Do: provide recommendations for shadows, effects, foliage, motion blur, V-Sync, render scale, fullscreen/borderless, and FPS cap.
- Dont: modify unsupported files or promise fixed gains.
- Verify: config readback where supported or manual confirmation.
- Rollback: restore previous config snapshot if modified.
- Sources: PUBG official guide, community settings research.

### pubg.fullscreen-optimization.troubleshoot
- Mode: Competitive
- Risk: Medium
- Do: offer fullscreen optimization toggle as troubleshooting/benchmark option.
- Dont: universally disable it without measuring on Windows 11.
- Verify: executable compatibility property and benchmark.
- Rollback: restore previous compatibility setting.
- Sources: PUBG official performance guide, Microsoft windowed optimization docs.

### pubg.verify-files-guidance
- Mode: Safe
- Risk: Low
- Do: guide user to verify game files via launcher if corruption is suspected.
- Dont: manipulate PUBG binaries directly.
- Verify: user confirmation/launcher state where possible.
- Rollback: no mutation.
- Sources: PUBG official performance guide.

## Cleanup, Drivers, and Health

### health.gpu-driver-state
- Mode: Safe
- Risk: Low
- Do: detect GPU driver version, age, vendor, and known mismatch state.
- Dont: auto-install drivers without user consent.
- Verify: driver version before/after.
- Rollback: recommend restore point/DDU manual path when needed.
- Sources: PUBG official performance guide, NVIDIA/AMD driver guidance.

### health.thermal-throttling-detect
- Mode: Safe
- Risk: Low
- Do: detect temperatures, clocks, power limits, and throttling indicators where APIs allow.
- Dont: change BIOS or overclock settings.
- Verify: telemetry capture.
- Rollback: no mutation.
- Sources: general hardware performance diagnostics.

### cleanup.temp-cache-safe
- Mode: Safe
- Risk: Low
- Do: clean temp files, app caches, and selected safe junk folders.
- Dont: delete shader caches blindly before every game session or delete user files.
- Verify: file count/space freed.
- Rollback: not guaranteed; only delete from approved temporary paths.
- Sources: Microsoft PC performance docs.

### local.rollback.snapshot
- Mode: Safe
- Risk: Low
- Do: create local snapshots before applying any tweak.
- Dont: apply mutable changes without snapshot when rollback is required.
- Verify: snapshot exists and includes all planned mutable changes.
- Rollback: restore snapshot.
- Sources: product safety requirement.

## Blocked Defaults

### blocked.disable-defender-global
- Mode: Blocked
- Reason: security risk and anti-cheat/OS trust risk.
- Allowed alternative: avoid scans during gameplay where supported and user-approved; use narrow exclusions only with clear warning.

### blocked.disable-windows-update-global
- Mode: Blocked
- Reason: security/stability risk.
- Allowed alternative: prevent unexpected restart during active gaming session.

### blocked.disable-uac
- Mode: Blocked
- Reason: security risk.
- Allowed alternative: use a signed elevated helper with least privilege.

### blocked.disable-pagefile
- Mode: Blocked
- Reason: crash/stability risk and poor behavior on memory pressure.
- Allowed alternative: inspect commit pressure and recommend managed/system pagefile.

### blocked.rename-system-files
- Mode: Blocked
- Reason: breaks Windows servicing and stability.
- Allowed alternative: supported service/task/session control only.

### blocked.kernel-or-anticheat-bypass
- Mode: Blocked
- Reason: violates product safety and anti-cheat compatibility.
- Allowed alternative: none.
