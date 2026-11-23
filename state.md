# State

## Now (2026-11-23 09:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day, no day cap; tokens.py 08:16 PM 4.9 % (11-22) + 3.1 % (11-23), week about 17 % vs 35 % pace; under: 6 lanes
session branch: 230ea0e5 (t113 green 07:24: MC-ROUND, eprintln ratchet 65 to 47); 67 greens.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open (closes on the flake lanes + haiku series 5 at 2 jobs); F7 open.

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r6 not met (co-48..51), tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | series r5 5/5 on 3b14c825; open: FLAKE-WAIT, FLAKE-K1, FLK-K1b, PIX-CAPSULE, TILING-LAYOUT |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r1 MERGE-OK (next train); r2 pr-1..5 |
| 9 | Flakes, window loss | open: cube-settle, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| flake-series-r6.md (F5, rc 3, 9; 1 %) | haiku high, claude/flake-series-r6 230ea0e5 | session_016iS3kLUGZ6nX4ntqUXGnFw 07:26, run 1 of 5 at 08:10 (22 steps, no FAIL) |
| co-focus-r3.md (rc 1, 2; 1.5 %) | opus high, claude/co-focus 8a599917 | review r2 FIX 08:32 (rcf-9 MAJOR: turn takes another client's keys; rcf-1..4 hold); r3 session_016iS3kLRpDadJrSqse98AFN 08:33 |
| tiling-flake-r2.md (F5, rc 3; 1 %) | sonnet medium, claude/tiling-flake 15a5d2fc | r2 DONE 09:24 (test-side causes, 10/10 under load, gate EXIT 0); PM review MERGE-OK; train t115 after t114 |
| record-v9-r4.md (F7, rc 11; 1.5 %) | sonnet medium, claude/record-v9 088d36de | r3 MERGE-OK (PM review), train t114 07:41, restarted 09:31 with CARGO_BUILD_JOBS=2 (container died twice in its build step, 08:22 and ~09:24); r4 session_016iS3kP9SomnoWqm2LHMfr4 07:42 |

## Queue (% week; 5 % reserve)
3. (rc 8) DOCS-RC r2 (pr-1..5) on the Thu head.
4. (rc 1, 2) co-focus r3 rcf-9, rcf-7 (MAJOR); rcf-8 MINOR after. (F7, rc 11) record-v9 r3 in t114 (108 regressions); r4 pins takes and names causes; its finding lines open fix lanes per look.
5. (rc 3, 9; F5) haiku series 6 running on 230ea0e5; F5 met again when it is 5/5 and tiling-flake r2 merges. Then fill lanes to 6 (DOCS-RC r2 Thu; fix lanes per look after record-v9 r3).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
