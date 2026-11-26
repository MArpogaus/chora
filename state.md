# State

## Now (2026-11-26 09:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 08:00 PM 3.5 % (11-26 BKK); week about 51 % vs 74 % time; need (100-51-5)/44 h = 1.0 %/h; lanes 5 build
session branch: 613844fc (t151 08:26: xp-blink, mc-minor, tour-rows); 96 greens. T152 = 613844fc + xp-busy 88951ae4 + mc-resume 329a13e5 + tiling-focus d4fd349f + flake-bar db34b699, gate runs 08:26

lanes-short: 5 of 6, tour-harness done 08:50; sixth: tiling persona rerun (haiku) after t152
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
| train152 | PM, 613844fc + 4 | gate runs 08:26 |
| review-co-drop-r4.md (0.5 %) | opus high, 142260b9 | starting |
| tiling-focus-r3.md (1.0 %) | opus high | MERGE-OK d4fd349f (opus r3, 3 nits): t152 |
| flake-bar-full-r1.md (0.8 %) | sonnet high | MERGE-OK db34b699 (PM r1): t152 |
| tour-harness-r2.md (0.3 %) | sonnet med | MERGE-OK f396ebe6 (PM r2): own t153 after t152 |
| tour-rows-r4.md (1.2 %) | opus high, 613844fc | starting |
| mc-resume-r1.md (0.5 %) | sonnet high | MERGE-OK 329a13e5 (PM r1): t152 |
| mc-boil-r1.md (0.6 %) | sonnet high, 329a13e5 | running s..MKTG4kJd4XYsCA9nv |
| xp-busy-r1.md (0.5 %) | sonnet high | MERGE-OK 88951ae4 (PM r1): t152 |
| xp-focus-r1.md (0.6 %) | sonnet high, a93bc75b | running s..Pn2EHnvJVsgX5ZD1n |
| tiling-chip-r1.md (0.3 %) | sonnet med, a93bc75b | running s..MP4o6yQmzjKXuT4ia |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 MERGE-OK db34b699 (t152; rfbf1-2 USR1 open: r10 watches). TOUR-HARNESS r2 MERGE-OK f396ebe6: t153 alone, next.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r4 done 142260b9 (1024 foot 18 px past margin): opus review.
3. (rc 2) P2 r11: T7 2/2; ti-57: TILING-FOCUS r3 MERGE-OK d4fd349f (t152).
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; XP-BUSY r1 MERGE-OK (rxbu1-1 NIT); TOUR-ROWS r4 (rtr3-2, 44 regr) runs; MC-BOIL r1 (rtr3-1) runs.
4b. (rc 9) MC-RESUME r1 MERGE-OK 329a13e5 (t152); XP-FOCUS r1 (rfc1-1) runs.
5. (F5, rc 3, 4, 5) FLAKE-SERIES r10 on the final head (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
