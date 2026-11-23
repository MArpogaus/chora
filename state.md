# State

## Now (2026-11-23 21:37 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 21:33 PM 4.9 % (11-22) + 5.2 % (11-23) + 0.7 % (11-24 BKK), week about 23 % vs 39 % pace
session branch: 1ec04694 (t117 green 16:38); 70 greens; train118 h-rf red 18:08 at leak (sh-35 flake; rerun 1 ok).
burn-down: audit r5 13:12: F1-F4, F6 met; F8 reopened (kit log: no CONTRIBUTING, no D); F5 8 rows (14); F7 open (24 acts unfiled).

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r7: co-52, co-54 fixed fe369799 (train119), co-48..50 pass, tiling: r5 not met (L1, L2, T6, T7 open), tiling-disc r3 |
| 3 | 3 greens, no flake | r6 0/5; FLAKE-SERIES r7; mc-58 fix 8dd16c0f |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 (pr-1..5) |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 MINOR; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train121: h-rf, leak-cell, co-reach, xp-readable, docs-rc-r2, guard-3, soak-r3, mc-host, xp-cube-faces | PM | head 8c411416; gate running 21:35 |
| tiling-disc-r4.md (rc 2; 0.5 %) | sonnet medium, claude/tiling-disc 7adb149c | rtd-6 MAJOR (train119 e2e red); starting |
| soak-r4.md (rc 9; 1 %) | haiku high, claude/soak-r4 39283655 | session_016iS3kLADrjFVxyE2AdpCzk 20:23 |
| flake-series-r7.md (rc 3; 1 %) | sonnet high, claude/flake-series-r7 1ec04694 | session_016iS3kLkxQHS3DxoJnqt8Xk 19:00 |
| notify-text-r1.md (F7; 0.5 %) | sonnet medium, claude/notify-text 1ec04694 | session_016iS3kPAjwT5TkEwLEvpXQS 19:58 |
## Queue (% week; 5 % reserve)
1. (rc 3, 7; 1 %) TRAIN121 running (9 branches); train119 red e2e: tiling-disc Edit settings row shifts Zebra (rtd-6), r4.
2. (rc 9) SOAK-R3 39283655 MERGE-OK: sl-9 closed; train120 with docs-rc-r2, guard-3. SOAK-R4 sh-36 settled vs peak RSS.
3. (rc 3, 9; F5) FLAKE-SERIES r7 running: sh-34, stall, xp-42; MC-HOST 8dd16c0f MERGE-OK, train120.
4. (rc 1, 2; 1.3 %) XP-CUBE-FACES d588d573 MERGE-OK; NOTIFY-TEXT r1 sh-31 (F7). After train119: co persona re-run, co-44, co-45.
5. (F5, F7; 3.5 %) TOUR-BASE (I6, rv9 = MAJOR) and tiling persona r6 (L1, L2, T6, T7) after train119; rtd-5 MINOR.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
