# State

## Now (2026-11-23 19:07 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 18:59 PM 4.9 % (11-22) + 5.2 % (11-23) + 0.2 % (11-24 BKK), week about 22 % vs 39 % pace
session branch: 1ec04694 (t117 green 16:38); 70 greens; train118 h-rf red 18:08 at leak (sh-35 flake; rerun 1 ok).
burn-down: audit r5 13:12: F1-F4, F6 met; F8 reopened (kit log: no CONTRIBUTING, no D); F5 8 rows (14); F7 open (24 acts unfiled).

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r7: co-52, co-54 fixed fe369799 (train119), co-48..50 pass, tiling: r5 not met (L1, L2, T6, T7 open), tiling-disc r3 |
| 3 | 3 greens, no flake | r6 0/5; FLAKE-SERIES r7, MC-HOST (mc-58) |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 (pr-1..5) |
| 9 | Flakes, window loss | sh-33 fixed (leak merged 1ec04694); sl-9 MAJOR soak rerun on 84c8c5d2; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| docs-rc-r2.md (rc 8; 0.5 %) | sonnet medium, claude/docs-rc-r2 bc153929 | 37730187 MERGE-OK ; train120 |
| guard-3-r1.md (rc 7; 0.4 %) | sonnet medium | 49f65382 MERGE-OK ; train120 |
| tiling-disc-r3.md (rc 2; 0.6 %) | sonnet medium, 3b32b9b4 | rtd-3 acts; session_016iS3kMu2bg6HBDwKc5RruR 16:47
| xp-readable-r3.md (F7; 0.6 %) | sonnet medium | 51cd0bc0 MERGE-OK; train119 |
| co-reach-r1.md (rc 1, 2; 2 %) | opus high | fe369799 MERGE-OK; train119 |
| soak-r3.md (rc 9; 1 %) | haiku high, claude/soak-r3 1ec04694 | session_016iS3kPTaTUWz9CUtmLJXzF 16:40 |
| h-rf-r2.md (rc 3; 0.6 %) | sonnet high | c9d8d9f0 MERGE-OK; t118 red leak (sh-35); train119
| leak-cell-r1.md (rc 3; 0.5 %) | sonnet medium, claude/leak-cell 1ec04694 | session_016iS3kNzHeMKkwNC3gXhwFy 18:10
| flake-series-r7.md (rc 3; 1 %) | sonnet high, claude/flake-series-r7 1ec04694 | session_016iS3kLkxQHS3DxoJnqt8Xk 19:00 |
| mc-host-r1.md (rc 3; 0.8 %) | sonnet high, claude/mc-host 1ec04694 | starting |
## Queue (% week; 5 % reserve)
1. (rc 9; 1 %) SOAK-R3 running (sl-9 MAJOR; end_probes each round); verdict closes or keeps sl-9.
2. (rc 3, 7) LEAK-CELL r1 (sh-35); train119 h-rf + leak-cell, then train120 docs-rc-r2, guard-3.
3. (rc 3, 9; F5) FLAKE-SERIES r7 running: sh-34, stall, xp-42.
4. (rc 1, 2) CO-REACH fe369799 MERGE-OK, train119; then co persona re-run. rcf-8, rcf-10, co-44, co-45, ti-30, sh-31 (1 %) after.
5. (F5, F7; 3.5 %) SH-32 I14 after tiling-disc; TOUR-BASE (I6, rv9 = MAJOR) after train119; MC-HOST r1 mc-58 running.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
