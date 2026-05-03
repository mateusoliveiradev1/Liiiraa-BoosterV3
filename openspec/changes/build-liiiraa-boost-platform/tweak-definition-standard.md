# Tweak Definition Standard

Every tweak must be implemented like a product feature, not like a script.

## Required Schema

```ts
type TweakMode = "safe" | "competitive" | "lab" | "blocked";
type TweakRisk = "low" | "medium" | "high" | "critical";
type RebootPolicy = "none" | "recommended" | "required";
type SessionScope = "persistent" | "profile-scoped" | "session-only" | "recommendation-only" | "blocked";
type RollbackKind = "exact-value" | "delete-created-value" | "restore-backup-file" | "restore-profile-export" | "manual-instructions" | "not-needed-readonly";

type TweakDefinition = {
  id: string;
  title: string;
  summary: string;
  category: string;
  mode: TweakMode;
  risk: TweakRisk;
  defaultEnabled: boolean;
  sessionScope: SessionScope;
  rollbackKind: RollbackKind;
  requiresAdmin: boolean;
  reboot: RebootPolicy;
  supportedOs: string[];
  supportedHardware: CompatibilityRule[];
  supportedDrivers: CompatibilityRule[];
  unsupportedWhen: CompatibilityRule[];
  conflictsWith: string[];
  laptopPolicy: "same-as-desktop" | "warn" | "different-defaults" | "blocked-on-battery";
  powerSourcePolicy: "any" | "ac-only" | "battery-safe-only";
  sourceLinks: SourceLink[];
  evidenceLevel: "official" | "community-tested" | "experimental" | "internal-benchmark-required";
  measurementPlan: MeasurementPlan;
  expectedImpact: ExpectedImpact;
  knownSideEffects: string[];
  antiCheatNotes: string[];
  gameClosedRequired: boolean;
  userDisclosure: string;
  do: string[];
  dont: string[];
  detect: TweakStep;
  precheck: TweakStep;
  plan: TweakStep;
  backup: TweakStep;
  apply: TweakStep;
  verify: TweakStep;
  rollback: TweakStep;
  tests: TweakTestPlan;
};
```

## Required Behavior
- Detect never mutates the system.
- Plan never mutates the system.
- Backup must complete before apply for any mutable tweak.
- Verify must prove the target state or explain why verification is impossible.
- Rollback must restore the exact previous state when possible.
- A failed verify must mark the tweak `rollback_required`.
- A tweak with no rollback cannot be Safe.
- A tweak with no source cannot be implemented.
- A tweak with only community evidence must require benchmark validation before becoming default.
- A tweak with hardware, driver, display, game, or anti-cheat sensitivity must declare applicability rules.
- A tweak that changes latency, scheduling, graphics, or networking must declare conflicts with related settings.
- A persistent tweak must document whether it is global, per-profile, or per-user.
- A game-related tweak must state whether the game and anti-cheat must be closed before apply.

## Applicability and Conflict Rules
Every tweak must answer:
- Which Windows builds and editions are supported?
- Which CPU/GPU/storage/network vendors are supported?
- Which driver versions or capability flags are required?
- Does laptop battery mode change the default?
- Does the tweak conflict with another tweak?
- Does it require a reboot, sign out, service restart, game restart, adapter restart, or shader rebuild?
- Does it change color management, HDR, VRR, overlay, capture, security, or anti-cheat behavior?

Examples:
- NVIDIA Reflex-capable games should prefer in-game Reflex over forcing driver Ultra Low Latency.
- AMD Radeon Chill must not be planned together with Anti-Lag or Radeon Boost when the driver states they do not interoperate.
- GameDVR/FSO changes must warn users who rely on ICC profiles, HDR workflows, or Windows color management in exclusive fullscreen.
- NIC RSC/offload/interrupt changes must be adapter-specific and benchmarked, not a global internet latency promise.
- CPU E-core, SMT, security mitigation, and automatic OC/undervolt changes are blocked by default.

## Evidence Levels
- Official: vendor/OS/game documentation supports the behavior.
- Community-tested: reputable community research exists but official docs are incomplete.
- Experimental: plausible but hardware/driver/version sensitive.
- Internal-benchmark-required: must be proven by our app before recommendation.

## Required Tests Per Tweak
- parses definition
- Safe mode eligibility
- Competitive/Lab opt-in behavior
- blocked OS/hardware behavior
- unsupported driver/capability behavior
- laptop/battery behavior
- conflict detection
- dry-run output
- backup metadata
- apply command generation
- verify success
- verify failure
- rollback command generation
- anti-cheat boundary if game-related
- game/process closed requirement if game-related
- no shell injection if arguments/paths are involved
- rollback restores exact previous state or documents manual recovery

## Required UI Per Tweak
Every tweak shown to the user must display:
- name
- category
- mode
- risk
- expected impact
- exact changes
- source confidence
- rollback status
- reboot requirement
- why it is recommended for this PC
- side effects and conflicts
- whether it is global, per-profile, session-only, recommendation-only, or blocked
- whether a benchmark is required before/after

## Promotion Rules
Safe default requires:
- low or medium risk
- rollback exists
- no security feature disabled silently
- no anti-cheat concern
- official or strong evidence
- tests pass

Competitive requires:
- explicit user opt-in
- tradeoff explanation
- rollback exists
- benchmark recommended

Lab requires:
- explicit per-tweak opt-in
- restore point/snapshot when possible
- clear instability warning
- never included in one-click default optimize

Blocked means:
- never apply automatically
- may only appear as educational "we do not do this" documentation
