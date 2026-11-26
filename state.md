# State

## Now (2026-11-26 13:16 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 12:52 PM 4.6 % (11-26 BKK); week est. 51 % (unmeasured, owner reads) vs 76 % time; need (100-51-5)/40 h = 1.1 %/h; lanes 6 build
session branch: 3be27e32 (t152 10:00); 97 greens. t153 red mc-scale (sh-37); t153+154 (4d8bbe36) gate green to watchdog-starved, xp-run red 13s then green on rerun (flake), tail runs; t155: abdd0158

lanes: 6 build of 6 (tour-rows, xp-minor-r5, tiling-focus-r4, co-reach-r6, rc1-pass-mc-r2, flake-series-r10)
burn-down: F1-4, F6, F8 met; F5 open (reds since r9), r10 runs; F7 open (88 open, 10 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp pass r2 met; mc, tiling, co open |
| 2 | Persona r4 | xp, mc meet; co r14 not met (co-79 Terminal-row pane path 0/5); tiling r12 not met: ti-57 half fixed (L3 run 2 holders [] I3) |
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
| tour-rows-r4.md (Q 4, 1.2 %) | opus high | running s..NWJPZjCXoCT77WnZn |
| xp-minor-r5.md (Q 4, 0.6 %) | sonnet med, 3be27e32 | running s..MoBcvY4KQx7Rg4Tfm |
| tiling-focus-r4.md (Q 3, 1.0 %) | opus high, 3be27e32 | running s..LXp8QBhegY1oEWiDc |
| co-reach-r6.md (Q 2, 1.0 %) | opus high, 4d8bbe36 | starting |
| rc1-pass-mc-r2.md (Q 4, 1.2 %) | opus high, 4d8bbe36 | running s..PVpuUFAbj4pjA97GK |
| flake-series-r10.md (Q 5, 0.6 %) | haiku high, 4d8bbe36 | running s..MrWJ1FZ4X7WVbaJa6 |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 merged t152 (rfbf1-2: r10 watches). TOUR-HARNESS r2 MERGE-OK f396ebe6: t153 alone, next.
2. (rc 2) construct r14 not met: co-75, co-76 hold 5/5; co-79 pane path (Terminal row) 0/5: CO-REACH r6 runs; then rc1 pass co r2.
3. (rc 2) P2 r12 NO: ti-57 half fixed; TILING-FOCUS r4 is the last round (else owner); no MINOR tiling lane before.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-ROWS r4 (rtr3-2, 44 regr) runs; MC-BOIL r1 MERGE-OK f10be960 (t154).
4a. (F7, rc 11) KIT-ICONS r1 MERGE-OK abdd0158 (t155).
4b. (rc 1) next: rc1 passes tiling and co before any xp round; H-RF2 before t156.
4c. (rc 1b) XP-MINOR r5 (xp-49); mc pass r2 runs; tour take r3 next, no tour tooling before it.
5. (F5, rc 3) FLAKE-SERIES r10 runs on 4d8bbe36 (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
