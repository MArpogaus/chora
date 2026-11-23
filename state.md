# State

## Now (2026-11-23 13:21 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 13:09 PM 4.9 % (11-22) + 4.2 % (11-23), week about 19 % vs 34 % pace
session branch: bc153929 (t115 green 12:46); 69 greens.
burn-down: audit r5 13:12: F1-F4, F6 met; F8 reopened (kit log: no CONTRIBUTING, no D); F5 8 rows (14); F7 open (24 acts unfiled).

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r6 not met (co-48..51), tiling: r5 not met (L1, L2, T6, T7 not reached; I2, T3, T1 hold), tiling-disc r1 |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; FLAKE-WAIT, FLAKE-K1, FLK-K1b, PIX-CAPSULE, TILING-LAYOUT |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | open: RUSTSEC-2026-0592 url 2.5.8 (GUARD-3) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 merged (54c30d16); r2 pr-1..5 |
| 9 | Flakes, window loss | open: sh-33 (leak-r1); sl-9 MAJOR soak rerun on 84c8c5d2; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-series-r6.md (F5, rc 3, 9; 1 %) | haiku high, claude/flake-series-r6 230ea0e5 | session_016iS3kLUGZ6nX4ntqUXGnFw 07:26; run 2 red keys (sh-34) |
| guard-3-r1.md (rc 7, F8, F5; 0.4 %) | sonnet medium, claude/guard-3 bc153929 | starting |
| tiling-disc-r1.md (rc 2; 1.5 %) | sonnet high, claude/tiling-disc 230ea0e5 | done 13:06 8df47cc8 (L1, L2, T7 red-green; foreign reds only); worker archived |
| xp-readable-r1.md (F7; 1.5 %) | sonnet high, claude/xp-readable 5b34d4b1 | session_016iS3kMVx3PinxquBDuK1wo |
| mc-tidy-r2.md (F7; 1.5 %) | sonnet medium, claude/mc-tidy 5b34d4b1 | session_016iS3kP9Js7yxNibyrZocXb; r1 12:52: mc-55, mc-57 close, keys red (sh-34) -> r2 |
| leak-r1.md (rc 9; 2 %) | opus high, claude/leak 5b34d4b1 | session_016iS3kMw4PZJnXirCxCD1cb 12:19 |
| p2-r7-co.md (rc 2; 0.8 %, review) | sonnet medium, claude/p2-r7-co bc153929 | session_016iS3kP1eVZ13YdLAaxeZvg 12:49 |
| review-tiling-disc-r1.md (rc 2; 0.5 %, review) | sonnet medium, claude/review-tiling-disc 8df47cc8 | session_016iS3kLbqihVVewmLEfPwyy 13:07 |
| h-rf-r1.md (rule 8; 1 %) | sonnet high, claude/h-rf bc153929 | session_016iS3kLtmCrsEF9Th9qFLi4 13:10 |
## Queue (% week; 5 % reserve)
1. (rc 7, F8, F5; 0.4 %) GUARD-3 r1 started; sim-long-r2 84c8c5d2 PM review, then soak rerun (sl-9).
2. (rule 8; 1 %) H-RF r1 started 13:10.
3. (rc 8; 0.5 %) DOCS-RC r2 (pr-1..5), sonnet medium.
4. (rc 1, 2; 1 %) rcf-8, rcf-10, co-44, co-45, ti-30, sh-31 once the look has no open MAJOR.
5. (F5, F7; 3.5 %) SH-32 I14 after tiling-disc; TOUR-BASE (I6, rv9 = MAJOR) after xp-readable; FLAKE-SERIES r7 after leak-r1.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
