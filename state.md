# State

## Now (2026-11-26 15:20 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 14:59 PM 5.4 % (11-26 BKK); week est. 51 % (unmeasured, owner reads) vs 76 % time; need (100-51-5)/40 h = 1.1 %/h; lanes 6 build
session branch: 0efebc69 (t155 15:15); 99 greens. t153 red sh-37; t154, t155 xp-run flake sh-44 (green on rerun); t156 (xp-minor-r5, tiling-focus-r4) gate runs

lanes: 6 build of 6 (tour-fix-r1, h-rf2-r2, kit-swallow-r1, co-reach-r6, mc-minor-r1, flake-series-r10)
burn-down: F1-4, F6, F8 met; F5 open (reds since r9), r10 runs; F7 open (88 open, 10 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp, mc pass r2 met; tiling, co open |
| 2 | Persona r4 | xp, mc meet; co r14 not met (co-79 Terminal-row pane path 0/5); tiling: ti-57 fixed 4/4 (pass r2); rpt2-1 open |
| 3 | 3 greens, no flake | open: reds t141, t142, t148, t153 (sh-37); FLAKE-SERIES r10 reruns |
| 4, 5 | CI on a PR; packaging | CI PR at release |
| 6 | Review per area | r1 MERGE-OK (shas: reports/review-*.md) |
| 7 | deny, machete; updates | GUARD-3 49f65382 (t118) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sl-1, sl-2 open |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| tour-fix-r1.md (Q 4, 1.2 %) | opus high, 972f9415 | running s..PCS93YNQQZy5cpfop |
| h-rf2-r2.md (Q 4b, 0.5 %) | sonnet high, a2b080ab | starting |
| kit-swallow-r1.md (Q 3, 0.6 %) | sonnet high, 4d8bbe36 | running s..Pqu9765urJR3qGoNQ |
| co-reach-r6.md (Q 2, 1.0 %) | opus high, 4d8bbe36 | running s..Lj635mZvdmt7myT7J |
| mc-minor-r1.md (Q 4, 0.8 %) | sonnet high, 4d8bbe36 | running s..Pg5YnDS8SDtMQ5L2i |
| flake-series-r10.md (Q 5, 0.6 %) | haiku high, 4d8bbe36 | running s..MrWJ1FZ4X7WVbaJa6 |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 t152, TOUR-HARNESS r2 t153 merged.
2. (rc 2) construct r14 not met: co-75, co-76 hold 5/5; co-79 pane path (Terminal row) 0/5: CO-REACH r6 runs; then rc1 pass co r2.
3. (rc 2) P2 r12 NO: ti-57; TILING-FOCUS r4 7ef97ab8 in t156; pass r2: ti-57 fixed 4/4, but rpt2-1 MAJOR (Super+Alt chords reach clients): KIT-SWALLOW r1 runs; rpt2-2 MINOR backlog.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-ROWS r4 review FIX (rtr4r-1 MAJOR c5d rows; rtr4r-2..4 MINOR): TOUR-FIX r1 runs (rtr4r-1, rtr4-3/4); take r3 after; MC-BOIL r1 merged t154.
4a. (F7, rc 11) KIT-ICONS r1 merged t155.
4b. (rc 3) H-RF2 review r1 FIX (rhrf2-1 MAJOR mixed commit blocked): H-RF2 r2 runs; rc1 pass co after co-reach r6.
4c. (rc 1b) XP-MINOR r5 7614fd24 in t156 ( rpx2-5 backlog); mc pass r2 met (mc-62 MINOR: MC-MINOR r1 runs; rpm2-6 backlog); tour take r3 next, no tour tooling before it.
5. (F5, rc 3) FLAKE-SERIES r10 runs on 4d8bbe36 (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
