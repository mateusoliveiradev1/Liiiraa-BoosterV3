# Desktop Route Audit

Date: 2026-05-03

Screenshots audited from `audit/current-screenshots/desktop/1440x900/`.

## Overall Findings

- Visual density is the main desktop issue. Most routes are information-rich but
  carry too many equal-weight cards, status blocks, icon actions, and repeated
  summaries in the same viewport.
- The top command bar has a useful next-action model, but route bodies often
  introduce more primary-looking buttons or icon clusters, diluting the next
  action.
- Category separation exists in content names, but the product does not yet read
  as distinct premium lanes for Game Mode, System, Network, GPU, Power,
  Startup/Services, Benchmarks, Rollback, and Settings.
- Existing visual tests appear to cover horizontal overflow and text clipping,
  but several visible labels rely on truncation or dense wrapping. The polish
  pass should reduce the need for ellipsis in navigation, top summaries, and
  compact cards.
- Button styling is generally consistent, but action semantics are not always
  obvious from icon-only controls. Each icon cluster needs stable labels,
  tooltips, and clearer grouping by apply, review, benchmark, rollback, export,
  and advanced intent.

## Route Notes

| Route | Strengths | Polish Risks |
| --- | --- | --- |
| Dashboard | Strong command-center concept, visible readiness score, trust and rollback rails, one clear scan action. | Repeats readiness, scan progress, rollback, benchmark, trust, and next-action summaries across many panels. The bottom snapshot and system readiness areas feel like audit ledgers rather than a premium first screen. |
| Scan | Clear read-only boundary, selected checks, progress state, no-write label, and findings grouped by impact/risk. | Scan scope, progress ladder, next action, and findings all compete. The route needs a simpler default scan cockpit with detailed findings moved into expandable or ledger treatment. |
| Optimize | Good safety-gated policy, mode segmentation, backup/apply/verify/benchmark/rollback sequence, and risk buckets. | This is the densest route. Diff preview, workflow state, multiple tables, mode controls, icon cluster, and policy explanations create a heavy technical document feel. One-click safe apply is visible but not visually dominant enough in the body. |
| Power | Scoped power-plan route and visible policy cards. | Needs to become a Power lane with one primary action and clearer separation between current plan, recommended plan, laptop/desktop policy, rollback, and advanced plan details. |
| NVIDIA | Driver/profile policy is product-relevant and distinct. | Needs more premium GPU lane treatment: driver state, profile state, monitor/VRR/refresh context, benchmark prompt, rollback, and blocked/lab controls should scan as separate decisions. |
| PUBG | Strong anti-cheat boundary, game detection, config snapshot, NVIDIA link, DX benchmark state, and launch flag ledger. | The checklist is credible but reads as a long compliance screen. Supported-game status, profile state, launch/config state, GPU link, and benchmark path should be separated into a game-mode flow with fewer same-weight panels. |
| Benchmarks | Good native-frame policy, before/after metrics, metadata, variance, charts, sample ledger, and privacy gate. | The page is long and data-heavy. The first viewport should show a proof summary, confidence state, and one capture/compare action, then push samples and metadata into a ledger or inspector. |
| Rollback | Recovery timeline is concrete and rollback-focused. | Restore-all is clear, but individual restore icon buttons are visually subtle and unlabeled. The session list needs stronger rollback/destructive styling and clearer before/after grouping. |
| Settings | Strong trust boundary, signed release lane, consent gates, update integrity, local data controls, and advanced gates. | The route has multiple unrelated decision areas at once. Privacy, updates, release trust, local data, and lab gates should be separated with clearer primary actions and fewer button styles in one viewport. |

## Required Follow-Up For Tasks 2.x And 3.x

- Establish a single primary action pattern per route body and per decision
  section.
- Convert repeated status cards into stable category lanes and compact proof
  primitives.
- Move dense technical copy into ledgers, drawers, inspectors, or expandable rows.
- Add tooltips and accessible labels to icon-only actions before widening their
  use.
- Define destructive and rollback spacing rules so restore/reset/delete actions
  never sit visually beside normal optimization actions without separation.
- Keep benchmark and proof modules honest: numeric proof needs metadata or an
  explicit example/preview label.
