# State

## Now (2026-11-26 10:12 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 10:12 PM 3.9 % (11-26 BKK); week about 51 % vs 74 % time; need (100-51-5)/44 h = 1.0 %/h; lanes 5 build
session branch: 3be27e32 (t152 10:00: xp-busy, mc-resume, tiling-focus, flake-bar); 97 greens. T153 = 3be27e32 + tour-harness f396ebe6, gate runs 10:02; T154 = co-drop 142260b9 + mc-boil f10be960 after

lanes-short: 4 build of 6 (tour-rows, xp-focus, tiling-chip, kit-icons); 5th: p2-r12-tiling starts; 6th after t154
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
| train153 | PM, 3be27e32 + 1 | gate runs 10:02 |
| review-co-drop-r4.md (0.5 %) | opus high, 142260b9 | MERGE-OK 142260b9 (3 MINOR, 1 NIT): t154 |
| tour-harness-r2.md (0.3 %) | sonnet med | MERGE-OK f396ebe6 (PM r2): t153 runs |
| tour-rows-r4.md (1.2 %) | opus high, 613844fc | running s..NWJPZjCXoCT77WnZn |
| mc-boil-r1.md (0.6 %) | sonnet high, 329a13e5 | MERGE-OK f10be960 (PM r1): t154 |
| kit-icons-r1.md (0.6 %) | sonnet high, 613844fc | running s..QWjVxZbMcVX2vDwX6 |
| xp-focus-r1.md (0.6 %) | sonnet high, a93bc75b | running s..Pn2EHnvJVsgX5ZD1n |
| tiling-chip-r1.md (0.3 %) | sonnet med, a93bc75b | running s..MP4o6yQmzjKXuT4ia |
| p2-r12-tiling.md (0.8 %) | sonnet med + haiku, 3be27e32 | starts 10:12 |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 MERGE-OK db34b699 (t152; rfbf1-2 USR1 open: r10 watches). TOUR-HARNESS r2 MERGE-OK f396ebe6: t153 alone, next.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r4 MERGE-OK 142260b9 (3 MINOR: known issues): t154 after t153.
3. (rc 2) P2 r11: T7 2/2; ti-57: TILING-FOCUS r3 merged t152; P2-R12-TILING starts.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-ROWS r4 (rtr3-2, 44 regr) runs; MC-BOIL r1 MERGE-OK f10be960 (t154).
4a. (F7, rc 11) KIT-ICONS r1 (rv9ka-2 MINOR, launcher icons) starts.
4b. (rc 9) MC-RESUME r1 merged t152; XP-FOCUS r1 (rfc1-1) runs.
5. (F5, rc 3, 4, 5) FLAKE-SERIES r10 on the final head (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
