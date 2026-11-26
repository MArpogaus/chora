# State

## Now (2026-11-26 11:24 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 10:59 PM 4.1 % (11-26 BKK); week about 51 % vs 74 % time; need (100-51-5)/44 h = 1.0 %/h; lanes 4 build
session branch: 3be27e32 (t152 10:00); 97 greens. T153 = 3be27e32 + tour-harness f396ebe6, gate runs 10:02; T154 = co-drop 142260b9 + mc-boil f10be960 + xp-focus e19b470b + tiling-chip 8add6eca after

lanes-short: 4 build of 6 (tour-rows, kit-icons, xp-minor-r5, tiling-focus-r4); 5th, 6th: p2-r14-co, mc pass after t154
burn-down: F1-4, F6, F8 met; F5 met r9, rerun on final head; F7 open (88 open, 10 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp pass r2 met; mc, tiling, co open |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r12 not met: ti-57 half fixed (L3 run 2 holders [] I3) |
| 3 | 3 greens, no flake | met: FLAKE-SERIES r9 5/5 |
| 4, 5 | CI on a PR; packaging | CI PR at release |
| 6 | Review per area | r1 MERGE-OK |
| 7 | deny, machete; updates | GUARD-3 49f65382 (t118) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sl-1, sl-2 open |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train153 | PM, 3be27e32 + 1 | gate 10:02: green through chat-keys 10:56 |
| review-co-drop-r4.md (0.5 %) | opus high | MERGE-OK 142260b9: t154 |
| tour-harness-r2.md (0.3 %) | sonnet med | MERGE-OK f396ebe6: t153 |
| tour-rows-r4.md (1.2 %) | opus high | running s..NWJPZjCXoCT77WnZn |
| mc-boil-r1.md (0.6 %) | sonnet high | MERGE-OK f10be960: t154 |
| kit-icons-r1.md (0.6 %) | sonnet high | running s..QWjVxZbMcVX2vDwX6 |
| xp-focus-r1.md (0.6 %) | sonnet high | MERGE-OK e19b470b: t154 |
| tiling-chip-r1.md (0.3 %) | sonnet med | MERGE-OK 8add6eca: t154 |
| rc1-pass-xp-r2.md (1.2 %) | opus high | done: xp confirms rc 1; xp-49 MINOR |
| xp-minor-r5.md (0.6 %) | sonnet med, 3be27e32 | starts |
| tiling-focus-r4.md (1.0 %) | opus high, 3be27e32 | starts |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 merged t152 (rfbf1-2: r10 watches). TOUR-HARNESS r2 MERGE-OK f396ebe6: t153 alone, next.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r4 MERGE-OK 142260b9: t154 after t153.
3. (rc 2) P2 r11: T7 2/2; P2 r12 NO: ti-57 half fixed (I3 holders []); TILING-FOCUS r4 runs, then P2 r13.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-ROWS r4 (rtr3-2, 44 regr) runs; MC-BOIL r1 MERGE-OK f10be960 (t154).
4a. (F7, rc 11) KIT-ICONS r1 (rv9ka-2 MINOR) runs.
4b. (rc 9) XP-FOCUS r1 MERGE-OK e19b470b (t154); rfc1-1: backlog.
4c. (rc 1b) xp pass r2 met; XP-MINOR r5 (xp-49); tour acts 7, 8 follow D119 (TOUR-ROWS r5); mc pass and P2-R14-CO after t154.
5. (F5, rc 3, 4, 5) FLAKE-SERIES r10 on the final head (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
