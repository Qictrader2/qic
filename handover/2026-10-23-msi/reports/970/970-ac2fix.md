# 970-ac2fix: AC2 control fixes

23 Oct 2026. PR https://github.com/Qictrader2/Frontend/pull/336 into `redesign/906`, branch
`970-ac2-checkbox-radio-amount`, head 00260d94 (two commits plus a merge of redesign/906 at dc179837, no force-push).
Not merged. No Trello writes.

## Files changed
- `src/components/ui/checkbox.tsx`: the customer tick is a lucide `Check`, 16 px, stroke 2.6, path `M20 6 9 17l-5-5`, as
  G0 draws `.check.is-on` (`ic ic-16`, stroke-width 2.6). The Radix tick stays on the default surface (`customer:hidden`).
- `src/components/ui/radio-group.tsx`: `customer:gap-3` on the group through `withCustomerLook`, giving a 34 px pitch
  (G0: 22 px `.tickrow` rows, 12 px column gap). A caller's own gap wins (NetworkRadioRows `gap-2.5`, unchanged).
- `src/components/ui/amount.tsx`: in and out units get `customer:text-money-in` / `customer:text-money-out` at xs, sm,
  md, xl and inline; lg stays muted by default; new `unitTone?: AmountUnitTone` ("figure" | "muted") overrides. Summary,
  held and neutral are unchanged.
- `src/__tests__/design/ac2-controls-fix.test.tsx` (new).

## Board calls
1. Amount unit tone, every directional `.amt` across all v5.2 sources (`v5.2-src/**/*.dc.html`): the unit is text inside
   the toned figure at xs (593), sm (6), md (6), lg (41, A room boards) and unsized (58), e.g. B16 `+0.24 USDT`; G0
   section 7 colours the whole `+120.00 USDT`. The 13 lg figures that draw a separate `.unit.u-lg` keep it muted: B8
   outcomes 1 to 3 (HarcStatusCard, the only directional lg caller) and shared G lg-popover. Under the decision rule the
   exact-screen and shared boards win for lg, so lg defaults to muted. No board draws a directional xl; xl follows the
   toned rule (reviewer LOW 1).
2. Field error icon left as the circle (coordinator ruling).

## Tests and gate
- Gate PASS twice (base 1679d151, then merged dc179837): typecheck, lint, build. Receipts under
  `.scratch/906-redesign/970-ac2fix-root/.scratch/gate/`. `.next` deleted after each gate.
- Slice and caller suites on the merged head: 5 suites, 201 tests pass. Wider run (design, components, navigation,
  reseller, affiliate-tiers, regression): 319 of 324 suites pass; failures are the four known base failures plus
  `navigation/notifications-glass` (time-of-day dependent date math, unrelated).

## Review
opus55-high: APPROVE, no HIGH or MEDIUM. LOW: directional xl tone is unboarded; an axis-only or responsive caller gap
(`gap-y-*`, `md:gap-*`) keeps `customer:gap-3`; jsdom cannot prove Tailwind ordering (precedent: spinner
`hidden customer:inline-block` works live); one test name says "today's classes". Not fixed (no caller affected).

## Notes
- Session restarted mid-run; a worker's uncommitted edits (base bab070d9) were checked, kept and extended (lg muted).
- Budget clock: the file held a stale start; I restarted it at 00:31 UTC, and the restart gap still reads as an overrun
  (3h08 of 1h30).
- I pushed once before noticing redesign/906 had moved to dc179837; I merged it in (no rewrite), re-gated, pushed.
- Root DESIGN.md line 110 ("The unit is Poppins 500 in --muted-foreground") needs a root edit saying in/out units take
  the figure's colour except at lg. Root file, outside this frontend PR; left for the coordinator.
- No backend dependency. No file outside 970's ui ownership edited.
