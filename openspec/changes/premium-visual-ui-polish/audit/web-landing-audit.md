# Web Landing Audit

Date: 2026-05-03

Screenshots audited from `audit/current-screenshots/web/`.

## Overall Findings

- The landing page already has the right product-first skeleton: brand name,
  product visual, performance promise, waitlist CTA, proof section, modules,
  PUBG section, trust section, and reserved future access section.
- The product visual is still a constructed illustration rather than a real app
  screenshot or a high-fidelity product preview. This weakens the first
  viewport signal and makes the page feel less concrete than the reference.
- CTA hierarchy is mostly clear. Desktop first viewport has one primary
  waitlist action and one secondary proof action. Mobile stacks the CTAs cleanly.
  Later sections still need stronger differentiation between waitlist, reserve,
  detail, Discord/community, and placeholder routes.
- Proof is careful and avoids universal claims, which is good. It should stay
  clearly labeled as example/preview content until real benchmark capture data
  is available.
- The section rhythm is deliberate, but the visual language is too uniform:
  dark grid, cyan/green/orange accents, large headings, and repeated cards make
  several sections feel structurally similar.
- Mobile fit is generally stable, but some hero-scale headings are very large
  for compact sections. The final access heading wraps heavily and needs more
  tuned mobile type scale or narrower wording.

## Section Notes

| Section | Strengths | Polish Risks |
| --- | --- | --- |
| First viewport | Strong product name, concise value proposition, visible waitlist CTA, proof CTA, product-preview area, and safeguard chips. | The preview is illustrative and not enough like the actual desktop UI. The hero uses very large type and a split composition that can feel closer to a concept deck than a polished product page. |
| Benchmark proof | Honest methodology copy and cautious metrics. | Needs clearer "example" labeling near the numbers, and a more polished proof module with hardware/context metadata if numbers remain visible. |
| PC-wide optimization | Four separated module cards are scan-friendly. | Still a generic card grid. Future polish should map modules to actual product lanes and use stronger icons/status/proof hooks. |
| PUBG focus | Clear anti-cheat boundary and game-specific modules. | Needs supported-game proof, detection state, and benchmark CTA relation. Current modules are informative but static. |
| Trust | Good emphasis that trust is product-level. | Social preview image is not a live app screenshot and competes with the product UI preview. The reserve CTA needs the same action grammar as the hero CTA. |
| Future access | Honest placeholder framing. | This section is implementation-facing. Public landing polish should hide internal flow reservations or reframe them as waitlist/early access without implying unfinished checkout. |

## Required Follow-Up For Tasks 4.x

- Replace or supplement the constructed app visual with a real current desktop
  screenshot or a high-fidelity generated product preview derived from Liiiraa
  UI only.
- Add separated sections for one-click optimization, optimization modules,
  supported games, benchmark proof, trust/safety, testimonials or preview
  quotes, FAQ, and final CTA.
- Label demo/example benchmark and testimonial content at the point of display.
- Reduce one-note palette risk by broadening neutral surface hierarchy and
  limiting repeated cyan/green/orange accent blocks.
- Tune mobile section headings and final CTA copy to avoid heavy wrapping while
  preserving readable type.
- Keep one primary CTA per decision area and make secondary CTAs visibly quieter
  but still accessible.
