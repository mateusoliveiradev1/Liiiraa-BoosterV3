# V1 Tweak Matrix

This is the V1 scope lock for Liiiraa Booster. Implementation must create either a real `TweakDefinition` or a blocked guardrail for every ID here.

V1 default behavior:
- Safe tweaks can appear in the default optimize flow.
- Competitive tweaks require explicit user consent and clear tradeoff copy.
- Lab tweaks require advanced opt-in, restore point/backups, and benchmark framing.
- Blocked items are implemented as denial rules so the app refuses unsafe scripts, catalog entries, or future AI-generated shortcuts.

Every applied tweak must support detect, precheck, dry-run plan, backup, apply, verify, rollback, do, dont, source links, risk notes, and anti-cheat notes.

## Source Synthesis
- Microsoft official docs support conservative use of `powercfg`, Game Mode/windowed optimizations, Defender exclusions/scheduling, Storage Sense, fsutil, RSS/RSC/network tuning, Delivery Optimization, VBS/HVCI, and Windows Graphics settings.
- NVIDIA official docs support profile-level power management and Reflex/Low Latency tradeoffs. NVIDIA Profile Inspector is allowed only as validated import/export compatibility, not as a random hidden-flag dump.
- AMD official docs support HYPR-RX, Anti-Lag, Radeon Boost, Radeon Chill, Enhanced Sync, and per-game profile planning where the installed driver exposes support.
- PUBG/BattlEye sources make anti-cheat safety non-negotiable: no memory edits, no BattlEye file changes, no test-signing/kernel-debug bypasses, no game binary patching.
- Atlas/ReviOS/XOS/HoneCtrl/WinUtil research is treated as candidate input only. Useful ideas: reversible power plans, GameDVR controls, VBS visibility, rollback playbooks, explicit options. Rejected as defaults: disabling Defender/Windows Update, removing OS components blindly, renaming system files, aggressive BCD timer/security changes, permanent timer services.

## Baseline and Health

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| sys.scan.inventory | Safe | Read OS build, edition, uptime, pending reboot, CPU, RAM, GPU, driver versions, disks, network adapters, active power plan, VBS/HVCI/VMP, Defender state, startup load. | Read-only; no rollback. | On |
| sys.scan.restore-point | Safe | Offer restore point creation before multi-tweak apply where System Restore is enabled. | Detect support and disk space; no silent enable. | Prompt |
| sys.scan.pending-reboot | Safe | Block high-risk apply when reboot is already pending. | Read pending reboot markers; no mutation. | On |
| sys.scan.thermal-power | Safe | Detect laptop/desktop, battery state, power source, CPU/GPU temperature sensors when available. | Read-only; no rollback. | On |
| sys.scan.driver-age | Safe | Flag old GPU/chipset/network/storage drivers and link to vendor update flow. | Recommendation only. | On |
| sys.scan.xmp-expo | Safe | Detect likely RAM underclock using SMBIOS/WMI where possible and recommend BIOS check. | Recommendation only; never change BIOS. | On |
| sys.scan.directx-gpu | Safe | Detect DX version, GPU feature level, VRR/HDR/HAGS visibility, DirectStorage readiness. | Read-only. | On |
| sys.scan.display-pipeline | Safe | Detect active refresh rate, HDR, VRR, MPO-relevant symptoms, ICC/color profile usage, and capture/overlay state before graphics tweaks. | Read-only. | On |
| sys.scan.disk-health | Safe | Detect free space, media type, TRIM status, basic SMART availability. | Read-only. | On |
| sys.scan.game-process | Safe | Detect running games/anti-cheat and delay sensitive changes. | No apply while PUBG/BattlEye is active for profile/system-risk changes. | On |
| sys.scan.baseline-score | Safe | Generate baseline health score from measurable bottlenecks, not generic "FPS promises". | No mutation. | On |

## CPU Intel and AMD

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| cpu.detect.vendor-topology | Safe | Detect Intel/AMD vendor, model, generation, P-core/E-core or CCD topology, SMT/HT, cache hints, and supported Windows CPU APIs. | Read-only. | On |
| cpu.detect.throttle | Safe | Detect thermal throttling, power-limit throttling symptoms, low clocks, and cooling/power bottlenecks where sensors expose data. | Read-only; recommend cooling/power fixes. | On |
| cpu.detect.chipset-driver | Safe | Detect chipset driver age/presence for Intel/AMD where possible. | Recommendation only. | On |
| cpu.power.ppm-audit | Safe | Audit Windows Processor Power Management settings for the active/Liiiraa plan. | Read-only until power task applies scoped plan values. | On |
| cpu.power.boost-mode-profile | Competitive | Tune processor boost mode only inside Liiiraa plans and only with thermal/power evidence. | Backup plan value; laptop warning. | Off |
| cpu.app.self-ecoqos | Safe | Make Liiiraa Booster background scans use cancellation/concurrency limits/low-impact scheduling while games run. | App-scoped; no OS rollback needed. | On |
| cpu.intel.thread-director.detect | Safe | Detect hybrid Intel CPU and Windows 11 readiness; explain Thread Director-friendly scheduling. | Read-only. | On |
| cpu.intel.apo.detect | Safe | Detect Intel APO/DTT/platform package readiness and show official setup path. | Recommendation only; no forced affinity. | Prompt |
| cpu.intel.apo.advanced-mode | Lab | Treat APO Advanced Mode for unverified games as benchmark experiment. | User applies through official Intel app; store benchmark result only. | Off |
| cpu.intel.epp-performance | Competitive | Apply EPP/performance bias only inside Liiiraa plans where supported. | Backup power plan value. | Off |
| cpu.intel.xtu-advisory | Lab | Provide Intel XTU/BIOS tuning checklist for unlocked platforms. | Advisory only; no automatic OC/undervolt. | Off |
| cpu.intel.disable-e-cores | Blocked | Do not disable E-cores as a default gaming tweak. | Guardrail; benchmarked BIOS-only advisory can be future Lab. | Block |
| cpu.amd.chipset-driver.detect | Safe | Detect AMD chipset driver needs, especially for Ryzen/X3D systems. | Recommendation only. | On |
| cpu.amd.cppc-preferred-cores | Safe | Detect CPPC/preferred core readiness where visible and recommend BIOS/chipset repair if missing. | Recommendation only. | Prompt |
| cpu.amd.x3d-scheduler.detect | Safe | For multi-CCD X3D CPUs, verify AMD PPM/3D V-Cache scheduling components and Game Mode dependency. | Recommendation only. | On |
| cpu.amd.pbo-curve-advisory | Lab | Provide PBO/Curve Optimizer/Ryzen Master checklist with warranty/stability warning. | Advisory only; no automatic tuning. | Off |
| cpu.amd.ryzen-power-plan-legacy | Safe | Detect legacy Ryzen Balanced plan only where relevant; do not force old plan on modern Windows. | Recommendation only. | Prompt |
| cpu.smt-disable | Blocked | Do not disable SMT/Hyper-Threading globally. | Guardrail. | Block |
| cpu.security-mitigations-disable | Blocked | Do not disable CPU security mitigations as a performance tweak. | Guardrail. | Block |
| cpu.priority.realtime-game | Blocked | Do not force game realtime priority. | Guardrail. | Block |
| cpu.hard-affinity.force | Lab | Hard affinity only as explicit benchmark experiment for a known title/profile. | Session-scoped and rollbackable if ever implemented. | Off |

## Power and Latency

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| power.plan.liiiraa-balanced | Safe | Duplicate current/Balanced plan into `Liiiraa Boost - Balanced`. | Store previous active scheme and created GUID. | On |
| power.plan.liiiraa-performance | Safe desktop, Competitive laptop | Create `Liiiraa Boost - Performance` from High/Ultimate-style settings but scoped to Liiiraa plan. | Laptop heat/battery warning; restore old active plan. | On desktop |
| power.plan.liiiraa-competitive | Competitive | Create `Liiiraa Boost - Competitive` with more aggressive AC-only values. | Explicit consent; restore old plan and settings. | Off |
| power.throttling.off | Safe desktop, Competitive laptop | Set `PowerThrottlingOff` only for performance profile. | Backup registry value; warn on laptop. | Desktop On |
| power.usb.selective-suspend.off | Safe | Disable USB selective suspend in Liiiraa plans to reduce device sleep/wake latency. | Backup plan value. | On |
| power.pcie.link-state.off | Safe desktop, Competitive laptop | Disable PCIe Link State in Liiiraa performance plans. | Backup plan value; warn laptop. | Desktop On |
| power.nvme.idle.reduce | Competitive | Reduce NVMe idle timeout for AC gaming profile. | Backup plan value; heat/power warning. | Off |
| power.disk.timeout.tune | Safe | Prevent aggressive disk sleep in Liiiraa performance plan on AC. | Backup plan value. | On |
| power.processor.epp.performance | Competitive | Lower EPP / bias CPU toward performance where supported. | Detect CPU/HWP support; backup value. | Off |
| power.processor.minmax.guard | Competitive | Adjust min/max processor state only inside Liiiraa plans. | Never set universal 100% on laptops by default; backup. | Off |
| power.processor.boost.thermal | Competitive | Provide Performance, Efficient, and Cooler boost profiles for users with thermal throttling. | Backup plan value; benchmark thermals and 1% lows. | Off |
| power.core-parking.lab | Lab | Expose core parking changes only as benchmark experiment. | Hardware-specific; restore plan values. | Off |
| power.display.sleep.session | Safe | Prevent display/system sleep during benchmark/game session only. | Session-scoped; restore automatically. | On |

## Windows Gaming Surface

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| game.mode.verify | Safe | Detect Game Mode state and recommend enabled for gaming. | Backup user setting if changed. | On |
| game.capture.background.off | Safe | Disable background recording/captures when user does not use them. | Backup HKCU/HKLM values. | On |
| game.capture.color-pipeline-warning | Safe | Warn before GameDVR/FSO changes when ICC profiles, HDR, or exclusive fullscreen color workflows are detected. | Plan-only warning; no mutation. | On |
| game.bar.overlay.optional | Safe | Offer Game Bar overlay disable only if user does not use widgets/recording. | Backup setting; do not remove Xbox packages. | Prompt |
| game.notifications.focus | Safe | Enable/plan focus mode during gaming sessions. | Session-scoped; restore. | Prompt |
| game.graphics.preference.pubg | Safe | Set PUBG executable to High Performance GPU in Windows Graphics settings when supported. | Backup app preference. | On |
| game.windowed.optimizations | Safe | Detect and recommend Optimizations for windowed games for DX10/DX11 borderless paths. | Per-game toggle where possible; benchmark if uncertain. | Prompt |
| game.vrr.detect-plan | Safe | Detect VRR support and current state; recommend only with compatible display. | Backup setting if changed. | Prompt |
| game.hags.benchmark | Competitive | Offer HAGS toggle only as before/after benchmark, not universal. | Requires reboot/sign-out as needed; backup. | Off |
| game.fso.per-game | Competitive | Fullscreen optimization compatibility toggle per executable only for troubleshooting/benchmarking. | Backup file compatibility flags. | Off |
| game.auto-hdr.detect | Safe | Detect Auto HDR state; recommend based on visual preference/performance notes. | Backup per-game setting where available. | Prompt |
| game.overlays.third-party | Safe | Detect Steam/Discord/GeForce/AMD overlays and recommend user-chosen reductions. | Recommendation or app-supported toggle only. | Prompt |
| game.present-path.benchmark | Safe/Competitive | Benchmark fullscreen, borderless, VRR, HAGS, and overlay combinations instead of assuming one path is best. | Session benchmark; no persistent change unless user chooses. | Prompt |

## Security Tradeoffs

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| security.vbs.detect | Safe | Show VBS/HVCI/Credential Guard/VMP status and performance/security explanation. | Read-only. | On |
| security.hvci.tradeoff | Competitive | Allow Memory Integrity off/on plan only with explicit security warning. | Detect incompatible drivers and previous state; reboot; restore. | Off |
| security.vmp.tradeoff | Competitive | Allow Virtual Machine Platform plan only when user does not need WSL/VM/emulators. | Detect WSL/Hyper-V features; reboot; restore. | Off |
| security.hyperv.tradeoff | Competitive | Plan Hyper-V stack changes only with compatibility warnings. | Backup feature states; restore. | Off |
| security.defender.exclusion.narrow | Safe/Competitive | Add narrow game/library exclusions only when path is verified and user accepts risk. | Backup exclusions; never wildcard system folders. | Prompt |
| security.defender.tamper-detect | Safe | Detect Tamper Protection/admin limitations before Defender-related planning. | Read-only; explain why some changes cannot be automated. | On |
| security.defender.schedule | Safe | Help schedule scans outside gaming hours. | Backup policy/task values. | Prompt |
| security.smart-app-control.detect | Safe | Detect Smart App Control state and warn that disabling it is a security tradeoff, not a performance default. | Read-only. | On |
| security.defender.disable-global | Blocked | Deny global Defender disable as optimizer default. | Guardrail only. | Block |
| security.uac.disable | Blocked | Deny UAC disable/lowering secure desktop as performance tweak. | Guardrail only. | Block |

## Background Work, Services, and Updates

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| bg.startup.review | Safe | List startup apps and high-impact entries; apply only known noncritical choices. | Backup startup entry state. | On |
| bg.background-apps.review | Safe | Detect noisy background apps and recommend reductions. | Backup app permission when changed. | Prompt |
| bg.scheduled.session-pause | Safe | Pause approved nonsecurity scheduled tasks during game/benchmark session. | Session-scoped restore. | Prompt |
| bg.search.indexer.pause-session | Competitive | Pause/reduce indexing only while gaming if active load is observed. | Store service state; restore. | Off |
| bg.search.system-file-rename | Blocked | Deny SearchApp/system binary rename/delete. | Guardrail only. | Block |
| bg.sysmain.conditional | Lab | SysMain changes only after HDD/SSD/RAM/load analysis and benchmark. | Backup service startup; restore. | Off |
| bg.printing.bluetooth.disable | Blocked by default | Do not disable broad daily-use services for FPS claims. | Only future custom profiles with user ownership. | Block |
| update.delivery-optimization.limit | Safe | Limit Delivery Optimization bandwidth/upload sharing when it competes with games. | Backup policy values. | Prompt |
| update.auto-restart.guard | Safe | Prevent surprise restarts during active gaming hours where Windows supports it. | Backup active hours/policy. | Prompt |
| update.driver-source-policy | Competitive | Offer a policy to reduce Windows Update GPU driver replacement only when the user commits to vendor driver maintenance. | Backup policy; never block security updates. | Off |
| update.disable-global | Blocked | Deny global Windows Update disable. | Guardrail only. | Block |

## Storage and Filesystem

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| storage.temp.cleanup | Safe | Clean temp/shader-adjacent junk only with preview and excludes. | Show files/size; no blind deletes. | Prompt |
| storage.sense.configure | Safe | Configure Storage Sense for safe cleanup cadence. | Backup settings. | Prompt |
| storage.trim.verify | Safe | Verify TRIM/Optimize Drives status and offer Windows-supported optimize. | Read media type; no SSD defrag forcing. | Prompt |
| storage.directstorage.check | Safe | Check DirectStorage readiness: NVMe, GPU feature, OS support, game location. | Read-only. | On |
| storage.nvme.driver-hack | Blocked | Deny consumer registry hacks that attempt to force server/native NVMe behavior. | Guardrail. | Block |
| storage.ntfs.last-access | Safe | Disable last access updates only when compatible. | Backup fsutil value; warn backup/compliance tools. | Prompt |
| storage.ntfs.8dot3 | Safe/Competitive | Disable 8.3 name creation for future files if legacy risk is low. | Backup fsutil value; compatibility warning. | Prompt |
| storage.shader-cache.inspect | Safe | Inspect GPU shader cache state and size. | Read-only unless user clears stale cache. | On |
| storage.shader-cache.clear-session | Competitive | Clear shader cache only for corruption/stutter troubleshooting, not every launch. | Warn first-run stutter; no schedule. | Off |
| storage.pagefile.disable | Blocked | Deny global pagefile disable. | Guardrail only. | Block |
| storage.component-removal | Blocked | Deny blind Appx/component removals as performance tweaks. | Guardrail only. | Block |

## Network

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| net.adapter.power-saving.off | Safe desktop, Competitive laptop | Disable adapter power saving where supported. | Backup adapter values. | Desktop On |
| net.eee.green.off | Competitive | Disable EEE/Green Ethernet/Energy Detect where adapter exposes exact property. | Adapter-specific backup; no wildcard writes. | Off |
| net.rss.ensure | Safe | Ensure RSS is enabled when supported and currently disabled without reason. | Backup global/adapter state. | Prompt |
| net.rsc.profile | Lab | RSC toggles only with benchmark and adapter-specific support. | Backup state. | Off |
| net.rsc.vpn-diagnosis | Lab | RSC/offload changes may be tested when VPN, capture tools, or adapter drivers show latency/throughput issues. | Adapter-specific backup; adapter restart warning. | Off |
| net.offloads.keep-default | Safe | Prefer checksum/LSO/offloads defaults unless evidence shows adapter issue. | Read-only/recommendation. | On |
| net.interrupt-moderation.lab | Lab | Tune interrupt moderation only by adapter model and latency benchmark. | Backup property. | Off |
| net.jumbo-frame.lan-only | Blocked by default | Do not enable Jumbo Frames for internet gaming; only LAN/NAS workflows can request it later. | Guardrail. | Block |
| net.dns.profile | Safe | Offer DNS profile for reliability/privacy, not FPS claims. | Backup adapter DNS. | Prompt |
| net.delivery.upload-limit | Safe | Limit update peer upload during gaming hours. | Backup Delivery Optimization. | Prompt |
| net.tcp-autotuning-myths | Blocked by default | Do not apply viral TCP packs globally. | Guardrail unless user imports explicit lab profile. | Block |
| net.reset.repair | Safe | Offer Windows network reset/repair only for troubleshooting, not optimization. | User-confirmed repair path. | Prompt |

## NVIDIA

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| nvidia.detect | Safe | Detect GPU, driver, NVAPI support, NPI availability, profiles, display refresh/VRR. | Read-only. | On |
| nvidia.driver.update-clean | Safe | Recommend latest Game Ready/Studio driver or clean reinstall when driver age/crashes indicate it. | Recommendation only. | Prompt |
| nvidia.backup.profiles | Safe | Export/backup global and app profiles before any mutation. | Required before apply. | On |
| nvidia.global.profile | Safe/Competitive | Create `Liiiraa Boost - Global Performance` with conservative settings only. | Readback verify; rollback backup. | Prompt |
| nvidia.pubg.profile | Competitive | Create/update PUBG profile for `TslGame.exe`. | Refuse while PUBG/BattlEye running; rollback backup. | Prompt |
| nvidia.power.max-perf | Competitive | Prefer Maximum Performance per-game/profile, not battery global. | Warn power/heat/fan; backup. | Off |
| nvidia.low-latency.on | Competitive | Use Low Latency On for DX11/non-Reflex paths; avoid Ultra as default. | Prefer in-game Reflex when available. | Off |
| nvidia.reflex.guidance | Safe | Detect/ref guide user to in-game Reflex where supported. | Recommendation only. | On |
| nvidia.reflex-vs-llm.policy | Safe | Prevent contradictory Reflex/driver low-latency stacking as a default. | Plan validation. | On |
| nvidia.profile.conflict-validator | Safe | Validate Reflex, Low Latency Mode, Max Frame Rate, V-SYNC, G-SYNC, frame generation, and in-game cap interactions. | Plan validation before apply. | On |
| nvidia.max-frame-rate.vrr | Competitive | Recommend cap based on refresh/VRR/user target. | Backup cap/off state. | Prompt |
| nvidia.gsync.vrr.profile | Competitive | Build G-SYNC/VRR profile recommendations with FPS cap and V-SYNC policy explained. | Backup profile settings. | Prompt |
| nvidia.shader-cache.size | Safe | Keep shader cache enabled; adjust size only if supported and useful. | Backup setting. | Prompt |
| nvidia.texture-filtering.performance | Competitive | Offer performance texture filtering for competitive profile. | Backup setting. | Off |
| nvidia.preferred-refresh.highest | Safe | Set highest refresh per PUBG profile where supported. | Backup setting. | Prompt |
| nvidia.rebar.detect | Safe | Detect Resizable BAR support/status through driver/system signals. | Read-only; recommend BIOS/VBIOS/driver path. | On |
| nvidia.rebar.hidden-override | Lab | Hidden ReBAR per-game override through NPI only as benchmark experiment. | Backup profile; benchmark before/after; no global force. | Off |
| nvidia.framegen.competitive-policy | Safe | Recommend frame generation off for competitive latency profiles and optional for visual/single-player profiles. | Recommendation only. | On |
| nvidia.auto-tuning-advisory | Lab | NVIDIA App automatic tuning/OC is advisory only. | No silent voltage/firmware changes. | Off |
| nvidia.hidden-flags.bulk | Blocked | Deny hidden undocumented bulk `.nip` dumps. | Guardrail. | Block |

## AMD and Intel Graphics

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| amd.detect | Safe | Detect Radeon driver, feature support, game profile availability. | Read-only. | On |
| amd.driver.update-clean | Safe | Recommend official Adrenalin/chipset driver update or clean reinstall when driver age/crashes indicate it. | Recommendation only. | Prompt |
| amd.hypr-rx.plan | Competitive | Recommend HYPR-RX where supported, explaining AFMF/RSR tradeoffs for competitive latency. | User applies via AMD or supported API only. | Prompt |
| amd.profile.conflict-validator | Safe | Validate HYPR-RX, Anti-Lag, Boost, Chill, Enhanced Sync, FreeSync, AFMF, FRTC, and in-game cap conflicts. | Plan validation before apply. | On |
| amd.anti-lag | Competitive | Offer Anti-Lag for GPU-limited cases where supported. | Backup profile when API available. | Prompt |
| amd.anti-lag2.supported-only | Safe | Treat Anti-Lag 2 as game-integrated/support-dependent; do not inject or fake support. | Recommendation only. | On |
| amd.radeon-boost | Competitive | Explain dynamic resolution tradeoff; not default for visibility-critical PUBG. | Backup profile. | Off |
| amd.chill | Safe/Competitive | Use Chill only for thermal/power cap goals, not max FPS. | Backup profile. | Off |
| amd.frtc.frame-cap | Safe/Competitive | Recommend driver/game frame cap for VRR, thermals, or latency consistency. | Backup profile where possible. | Prompt |
| amd.enhanced-sync | Competitive | Offer as tearing/latency alternative with stutter warning. | Backup profile. | Prompt |
| amd.freesync.vrr.profile | Competitive | Detect FreeSync/VRR and recommend cap/sync policy. | Backup setting where possible. | Prompt |
| amd.sam.detect | Safe | Detect Smart Access Memory/Resizable BAR availability and state. | Read-only; recommend BIOS/driver path. | On |
| amd.sam.enable-guide | Safe | Guide user to official SAM/ReBAR requirements; no firmware flashing. | Recommendation only. | Prompt |
| amd.afmf.framegen-policy | Competitive/Lab | Treat AFMF/frame generation as optional for visual/general games, not PUBG competitive default. | User consent and benchmark. | Off |
| amd.ris-rsr.profile | Safe/Competitive | Recommend Radeon Image Sharpening/RSR only when user accepts visual tradeoffs. | Backup profile where possible. | Prompt |
| amd.ulps | Lab | ULPS changes only for multi-GPU/known issue and benchmark. | Backup registry values. | Off |
| amd.mpo-fix | Lab | MPO `OverlayTestMode` only for known flicker/stutter issue with rollback. | Backup DWM value. | Off |
| amd.anti-lag-plus-deprecated | Blocked | Deny deprecated/injection-like Anti-Lag+ style behavior and never fake game-integrated support. | Guardrail. | Block |
| amd.crash-defender-disable | Blocked | Deny disabling AMD Crash Defender services by default. | Guardrail. | Block |
| intel.detect | Safe | Detect Intel/Arc driver and known PUBG issues; recommend driver updates. | Read-only. | On |
| intel.presentmon.gpubusy | Safe | Use PresentMon/GPU Busy metrics where available. | Read-only. | On |

## PUBG

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| pubg.detect.install | Safe | Detect Steam/Epic install, `TslGame.exe`, config folders, crash folders, BattlEye. | Read-only. | On |
| pubg.config.snapshot | Safe | Snapshot config files before any recommendation. | Backup required. | On |
| pubg.config.safe-read | Safe | Parse supported user config keys only. | No game binary writes. | On |
| pubg.launch-options.legacy | Safe | Detect old/viral launch flags and recommend cleanup. | Backup current launch options where accessible. | Prompt |
| pubg.dx11-vs-dx11e | Competitive | Benchmark DX11 vs DX11 Enhanced per machine; no universal force. | Store results and rollback chosen mode. | Prompt |
| pubg.dx12 | Lab | Treat DX12 as lab/benchmark only if game exposes it. | Restore chosen mode. | Off |
| pubg.fullscreen-mode | Competitive | Benchmark fullscreen/borderless/windowed based on current Windows graphics path. | Backup setting. | Prompt |
| pubg.settings.visibility | Safe/Competitive | Provide visibility/performance checklist, user-confirmed apply only for supported config keys. | Backup config. | Prompt |
| pubg.shader-cache.repair | Competitive | Repair/clear shader cache only for troubleshooting. | Warn first-run stutter. | Off |
| pubg.files.verify | Safe | Recommend Steam/Epic verify files when corruption/stutter symptoms appear. | Launch store-supported flow only. | Prompt |
| pubg.network.jitter-check | Safe | Measure ping/jitter/packet loss to help separate PC vs network issues. | Read-only. | On |
| pubg.crash-folder.collect | Safe | Help collect PUBG crash reports from official local crash path for support/debugging. | Read-only unless user exports zip. | Prompt |
| pubg.process-priority | Blocked by default | Do not force realtime/high priority blindly. | Guardrail unless future benchmarked lab. | Block |
| pubg.delete-game-content | Blocked | Do not delete movies, pak files, shaders, or game content folders as an optimization. | Use official verify/repair instead. | Block |
| pubg.memory-edit | Blocked | Deny memory edits, recoil/macro/scripting, file patching. | Guardrail. | Block |
| pubg.battleye-files | Blocked | Deny modifying BattlEye service/files/permissions. | Guardrail. | Block |

## Benchmarking and Proof

| ID | Mode | V1 Behavior | Precheck and Rollback | Default |
| --- | --- | --- | --- | --- |
| bench.presentmon.capture | Safe | Capture frametime metrics with low overhead. | User consent for capture. | Prompt |
| bench.before-after | Safe | Compare before/after with same metadata: map/session, driver, Windows build, power plan, catalog version. | No mutation. | On |
| bench.variance-warning | Safe | Warn when change is inside noise/variance. | No mutation. | On |
| bench.cpu-gpu-bound | Safe | Use GPU Busy/CPU indicators when available to classify bottleneck. | Read-only. | On |
| bench.thermal-correlation | Safe | Correlate clocks/temperature/power when available. | Read-only. | Prompt |
| bench.cloud-sync | Safe | Sync only after explicit consent and redaction. | Local-first; delete/export support. | Off |
| bench.native-vs-generated | Safe | Separate native frames from generated/interpolated frames where tools expose it. | Mark confidence if tool cannot separate them. | On |
| bench.latency-proxy | Safe | Track render present latency/GPU busy where available instead of claiming true click-to-photon latency. | Label metric source and limits. | On |

## Blocked Guardrails

| ID | Blocked Action | Reason |
| --- | --- | --- |
| blocked.defender.disable | Disable Microsoft Defender globally. | Security regression; use narrow exclusions/scheduling only. |
| blocked.windows-update.disable | Disable Windows Update globally/permanently. | Security and reliability regression. |
| blocked.uac.disable | Disable UAC or secure desktop for FPS claims. | Privilege boundary regression. |
| blocked.pagefile.disable | Disable pagefile globally. | Stability/crash dump risk. |
| blocked.driver-signing | Disable driver signature enforcement, enable testsigning, kernel debugging, or unsafe BCD anti-cheat bypasses. | Anti-cheat/security risk. |
| blocked.system-file-rename | Rename/delete `SearchApp.exe`, `RuntimeBroker.exe`, or other Windows binaries. | OS integrity risk. |
| blocked.component-removal | Blindly remove Windows packages/apps/components. | Breakage and updater risk. |
| blocked.trustedinstaller-takeover | Use TrustedInstaller/NSudo-style ownership to force OS mutation. | Too invasive for commercial optimizer. |
| blocked.bcd-timer-pack | Apply viral BCD timer/APIC/TSCSync packs by default. | Hardware-specific, high regression risk. |
| blocked.timer-service-permanent | Install permanent timer resolution services by default. | Power/compatibility risk. |
| blocked.bulk-reg-pack | Apply imported `.reg` packs without typed definitions. | No safety/rollback/source integrity. |
| blocked.hidden-gpu-dump | Import hidden NPI/AMD flags as a bulk preset. | Driver-version risk. |
| blocked.anticheat-tamper | Modify BattlEye/PUBG process, files, memory, handles, or services. | Ban/security risk. |
| blocked.firewall-disable | Disable firewall globally. | Security regression. |
| blocked.exclusions-wildcard | Add broad Defender exclusions like drive root, user profile root, downloads, temp, or system folders. | Malware bypass risk. |
| blocked.services-bulk-disable | Disable large service lists without hardware/user precheck. | Breaks OS features. |
| blocked.force-realtime-priority | Force realtime process priority for games. | System starvation/stutter risk. |
| blocked.disable-crash-protection | Disable AMD/NVIDIA/Windows crash protection services by default. | Stability risk. |
| blocked.cpu-mitigations-disable | Disable Spectre/Meltdown/class CPU mitigations for FPS claims. | Security regression. |
| blocked.e-core-disable-default | Disable Intel E-cores as a default optimization. | OS/Thread Director regression risk. |
| blocked.smt-disable-default | Disable SMT/Hyper-Threading globally. | Workload-specific regression risk. |
| blocked.software-overclock-auto | Automatically overclock or undervolt CPU/GPU. | Heat, stability, warranty, and support risk. |
| blocked.firmware-flash | Flash motherboard BIOS, GPU VBIOS, or firmware. | Bricking/security risk. |
| blocked.force-rebar-global | Force ReBAR/SAM hidden flags globally. | Per-game regression risk. |
| blocked.nvme-driver-reg-hack | Force unsupported NVMe/server storage driver behavior through registry hacks. | Unsupported and recently unstable target. |
| blocked.jumbo-frame-internet | Enable Jumbo Frames for internet gaming. | Requires end-to-end LAN support and can break connectivity. |
| blocked.color-breaking-gamedvr | Apply capture/FSO/GameDVR changes without warning when color profile/HDR workflows are detected. | Visual correctness regression. |
| blocked.no-rollback | Any apply action without backup and verify. | Violates product safety model. |
