# State

## Now (2026-11-24 00:05 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 23:01 PM 4.9 % (11-22) + 5.2 % (11-23) + 0.9 % (11-24 BKK), week about 23 % vs 40 % pace
session branch: 8c411416 (t121 green 23:50); 71 greens; train122 running 23:55 (notify-text, tiling-disc, xp-front-bar-r2).
burn-down: audit r5 13:12: F1-F4, F6 met; F8 reopened (kit log: no CONTRIBUTING, no D); F5 8 rows (14); F7 open (24 acts unfiled).

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r7 not met (reach), co-52, co-54 fixed t121: P2-R8-CO; tiling r5 not met (L1, L2, T6, T7), tiling-disc train122 |
| 3 | 3 greens, no flake | r6 0/5; FLAKE-SERIES r7; mc-58 fixed t121 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 (pr-1..5) |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train122: notify-text, tiling-disc, xp-front-bar-r2 | PM | on 8c411416; gate 23:55 |
| soak-r4 (rc 9) | MERGE-OK f0c45fcf, sh-36 closed | train123 |
| p2-r8-co.md (rc 2; 0.8 %) | sonnet medium, claude/p2-r8-co 8c411416 | session_016iS3kL5XZy4sE2bqwnfLXK 23:54 |
| tour-base-r1.md (F7, rc 11; 1.5 %) | sonnet high, claude/tour-base 8c411416 | session_016iS3kLwgpLsqzbCZvy458S 23:54 |
| flake-series-r7.md (rc 3; 1 %) | sonnet high, claude/flake-series-r7 1ec04694 | session_016iS3kLkxQHS3DxoJnqt8Xk 19:00 |
## Queue (% week; 5 % reserve)
1. (rc 1, 2, 3; 1 %) TRAIN122 running: notify-text 60a38634, tiling-disc ca165f7b, xp-front-bar-r2 da8b8fa9.
2. (rc 2) P2-R8-CO: co-51, co-52, co-54 verified; then TILING persona r6 (L1, L2, T6, T7) after train122.
3. (rc 3, 9; F5) FLAKE-SERIES r7 running (sh-34, stall, xp-42); soak-r4 MERGE-OK f0c45fcf (sh-36 closed) joins train123.
4. (F7, rc 11; 1.5 %) TOUR-BASE: I6 baseline, 0 unexplained acts.
5. (rc 1; 1 %) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
