# State

## Now (2026-11-23 12:19 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 10:50 PM 4.9 % (11-22) + 3.6 % (11-23), week about 18 % vs 35 % pace
session branch: 5b34d4b1 (t114 green 11:10: record-v9 r3); 68 greens, t115 running (tiling-flake, co-focus, record-v9 r4).
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
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: sim-long r2 recheck (sl-1, sl-2, sl-3; cube-settle, mesh-churn, core-fix merged) |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-series-r6.md (F5, rc 3, 9; 1 %) | haiku high, claude/flake-series-r6 230ea0e5 | session_016iS3kLUGZ6nX4ntqUXGnFw 07:26; run 1 green, run 2 red keys "the Foot row of Programs started no foot" (F5 not 5/5), run 3 at 09:56 |
| co-focus-r3.md (1.5 %) | opus high, claude/co-focus 3a625261 | MERGE-OK 10:04 (rcf-8, rcf-10 MINOR); t115 |
| tiling-flake-r2.md (F5; 1 %) | sonnet medium, claude/tiling-flake 15a5d2fc | MERGE-OK; t115 |
| record-v9-r4.md (F7; 1.5 %) | sonnet medium, claude/record-v9 6949cddb | MERGE-OK 10:56 (rr9-1 MINOR); t115 |
| sim-long-r2.md (rc 9; 1 %) | haiku high, claude/sim-long-r2 230ea0e5 | PARTIAL 12:01: sl-3, sl-1 5/5 green; NEW sh-33 MAJOR leak; wrap-up |
| tiling-disc-r1.md (rc 2; 1.5 %) | sonnet high, claude/tiling-disc 230ea0e5 | session_016iS3kLr3z3YfoMNYBqXya5 10:10 |
| xp-readable-r1.md (F7; 1.5 %) | sonnet high, claude/xp-readable 5b34d4b1 | session_016iS3kMVx3PinxquBDuK1wo |
| mc-tidy-r1.md (F7; 1 %) | sonnet medium, claude/mc-tidy 5b34d4b1 | session_016iS3kP9Js7yxNibyrZocXb |
| leak-r1.md (rc 9; 2 %) | opus high, claude/leak 5b34d4b1 | session_016iS3kMw4PZJnXirCxCD1cb 12:19 |
## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (rc 1, 2, 11) t115: co-focus r3, tiling-flake r2, record-v9 r4; then rcf-8, rcf-10 MINOR; co-44, co-45, ti-30, sh-31 once no MAJOR is open on the look (co-48..51 verify after t115).
5. (rc 9) sh-33: leak-r1 running. (F7) xp-readable r1, mc-tidy r1 running. (F5, rc 3, 9) series 6 on 230ea0e5, sim-long r2; F5 met at 5/5 with tiling-flake r2 merged. DOCS-RC r2 Thu.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
