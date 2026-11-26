# State

## Now (2026-11-26 07:03 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 06:52 PM 3.1 % (11-26 BKK); week about 49 % vs 73 % time; need (100-49-5)/45 h = 1.02 %/h; lanes 6 build
session branch: a93bc75b (t149 05:55); 95 greens. T150 RED (launch.py REPO hardcoded /home/user/chora: rth2-1); T151 = a93bc75b + xp-blink + mc-minor + tour-rows, TOUR_REPO set, gate runs 06:47; T152 xp-busy 88951ae4

burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79, rv9ka-1 fixed |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r11 not met: T7 2/2 green, ti-57 focus after T7 (L3 red) |
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
| train151 | PM, a93bc75b + 3fd3a3bb + 1ab2e6a7 + f36a229d | gate runs 06:47 |
| tour-rows-r3.md (0.6 %) | sonnet med | MERGE-OK f36a229d (PM r3): t151; rtr3-1, rtr3-2 MAJOR filed |
| co-drop-r3.md (0.6 %) | sonnet high | FIX (rcdr3-1 MAJOR key_row cut at 1024x768): r4 |
| co-drop-r4.md (0.5 %) | sonnet high, 369cf96f | running s..Q293gSQavooQkk1iE |
| tiling-focus-r3.md (1.0 %) | opus high | DONE d4fd349f: review r3 |
| review-tiling-focus-r3.md (0.5 %) | opus high, d4fd349f | running s..Q4q1hXSzEAEcFfqeg |
| flake-bar-full-r1.md (0.8 %) | sonnet high, 6daaaab8 | running s..MRoVkQ1EbwsGcjcGg |
| tour-harness-r2.md (0.3 %) | sonnet med, 06c3f7cc + a93bc75b | running s..NZ7KUNNYNp5CsRuEe |
| mc-resume-r1.md (0.5 %) | sonnet high, 3e3b1b2d + 1ab2e6a7 | running s..MaFjpYTR7g5JAHQ1J |
| xp-busy-r1.md (0.5 %) | sonnet high | MERGE-OK 88951ae4 (PM r1): t152 |
| xp-pager-r1.md (0.4 %) | sonnet med, a93bc75b | starting |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 runs. T150 RED: TOUR-HARNESS r2 (rth2-1 tour reads /home/user/chora), own train at once; T151 runs with TOUR_REPO; T152 xp-busy.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r3 FIX (rcdr3-1 MAJOR): r4.
3. (rc 2) P2 r11: T7 2/2; ti-57: TILING-FOCUS r3 DONE d4fd349f: opus review r3.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; XP-BUSY r1 MERGE-OK (rxbu1-1 NIT); XP-PAGER r1 (rv9j-1 MINOR, xp free of MAJOR); TOUR-ROWS r3 MERGE-OK, r4 for rtr3-2 (44 regr without intended row); MC-BOIL r1 (rtr3-1 side panel 29 of 56 px) after MC-RESUME.
4b. (rc 9) rfs2-1 mc host restart keeps old config, rmm5-2: MC-RESUME r1.
5. (F5, rc 3, 4, 5) FLAKE-SERIES r10 on the final head (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
