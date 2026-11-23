# State

## Now (2026-11-23 12:53 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 12:38 PM 4.9 % (11-22) + 3.9 % (11-23), week about 18 % vs 34 % pace
session branch: bc153929 (t115 green 12:46: tiling-flake r2, co-focus r3, record-v9 r4); 69 greens.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open (flake lanes, series); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r6 not met (co-48..51), tiling: r5 not met (L1, L2, T6, T7 not reached; I2, T3, T1 hold), tiling-disc r1 |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; FLAKE-WAIT, FLAKE-K1, FLK-K1b, PIX-CAPSULE, TILING-LAYOUT |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 merged (54c30d16); r2 pr-1..5 |
| 9 | Flakes, window loss | open: sim-long r2 recheck (sl-1, sl-2, sl-3; cube-settle, mesh-churn, core-fix merged) |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-series-r6.md (F5, rc 3, 9; 1 %) | haiku high, claude/flake-series-r6 230ea0e5 | session_016iS3kLUGZ6nX4ntqUXGnFw 07:26; run 1 green, run 2 red keys "the Foot row of Programs started no foot" (F5 not 5/5), run 3 at 09:56 |
| sim-long-r2.md (rc 9; 1 %) | haiku high, claude/sim-long-r2 230ea0e5 | PARTIAL 12:01: sl-3, sl-1 5/5 green; NEW sh-33 MAJOR leak; wrap-up |
| tiling-disc-r1.md (rc 2; 1.5 %) | sonnet high, claude/tiling-disc 230ea0e5 | session_016iS3kLr3z3YfoMNYBqXya5 10:10 |
| xp-readable-r1.md (F7; 1.5 %) | sonnet high, claude/xp-readable 5b34d4b1 | session_016iS3kMVx3PinxquBDuK1wo |
| mc-tidy-r2.md (F7; 1.5 %) | sonnet medium, claude/mc-tidy 5b34d4b1 | session_016iS3kP9Js7yxNibyrZocXb; r1 12:52: mc-55, mc-57 close, keys red (sh-34) -> r2 |
| leak-r1.md (rc 9; 2 %) | opus high, claude/leak 5b34d4b1 | session_016iS3kMw4PZJnXirCxCD1cb 12:19 |
| p2-r7-co.md (rc 2; 0.8 %, review) | sonnet medium, claude/p2-r7-co bc153929 | session_016iS3kP1eVZ13YdLAaxeZvg 12:49 |
| audit-f-r5.md (F1-F8; 0.8 %, review) | opus high, claude/audit-f-r5 bc153929 | session_016iS3kLW1UjAiKQkZNZ2fC5 12:49 |
## Queue (% week; 5 % reserve)
1. (rc 1, 2, 11; 1.6 %) p2-r7-co (co-48..51 verify), audit-f r5 started 12:49.
2. (rule 8; 1 %) H-RF lane at the next free slot: red-first per finding id.
3. (rc 8; 0.5 %) DOCS-RC r2 (pr-1..5), sonnet medium.
4. (rc 1, 2; 1 %) rcf-8, rcf-10, co-44, co-45, ti-30, sh-31 once the look has no open MAJOR.
5. (F5, rc 3, 9; 3 %) leak-r1 (sh-33) running; F5 met at 5/5 with sh-32..34 closed. (F7) xp-readable, mc-tidy running.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
