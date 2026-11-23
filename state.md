# State

## Now (2026-11-23 18:09 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 16:39 PM 4.9 % (11-22) + 5.1 % (11-23), week about 21 % vs 36 % pace
session branch: 1ec04694 (t117 green 16:38); 70 greens; train118 h-rf red 18:08 at leak (sh-35 cell flake: one +7 MB step).
burn-down: audit r5 13:12: F1-F4, F6 met; F8 reopened (kit log: no CONTRIBUTING, no D); F5 8 rows (14); F7 open (24 acts unfiled).

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r7 not met (co-51, co-52, co-54, co-57: CO-REACH), co-48..50 pass, tiling: r5 not met (L1, L2, T6, T7 open), tiling-disc r3 |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; FLAKE-WAIT, FLAKE-K1, FLK-K1b, PIX-CAPSULE, TILING-LAYOUT, xp-42 |
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
| xp-readable-r3.md (F7; 0.6 %) | sonnet medium, e85413d2 | rxr-6 MAJOR; session_016iS3kQYgq6js1BArE4dk5K 17:30
| co-reach-r1.md (rc 1, 2; 2 %) | opus high, claude/co-reach bc153929 | session_016iS3kNey3bZFXVjkQbFTGV 13:59 |
| soak-r3.md (rc 9; 1 %) | haiku high, claude/soak-r3 1ec04694 | session_016iS3kPTaTUWz9CUtmLJXzF 16:40 |
| h-rf-r2.md (rc 3; 0.6 %) | sonnet high | c9d8d9f0 MERGE-OK; t118 red leak (sh-35); train119
| leak-cell-r1.md (rc 3; 0.5 %) | sonnet medium, claude/leak-cell 1ec04694 | starting 18:15
## Queue (% week; 5 % reserve)
1. (rc 9; 1 %) SOAK-R3 running (sl-9 MAJOR; end_probes each round); verdict closes or keeps sl-9.
2. (rc 3, 7) LEAK-CELL r1 (sh-35); train119 h-rf + leak-cell (harness), then train120 docs-rc-r2, guard-3.
3. (rc 3, 9; F5) fs-r6: 0/5 green (keys sh-34 x4, acts x1 stalls): FLAKE-SERIES r7 after train118.
4. (rc 1, 2; 2 %) CO-REACH r1 starting (co-52 raised MAJOR, co-54 lost window, co-51, co-57). (rc 1, 2; 1 %) rcf-8, rcf-10, co-44, co-45, ti-30, sh-31 once the look has no open MAJOR.
5. (F5, F7; 3.5 %) SH-32 I14 after tiling-disc; TOUR-BASE (I6, rv9 = MAJOR) after xp-readable; need_fuzzel guard xp-run, e2e with FLAKE-SERIES r7.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
