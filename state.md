# State

## Now (2026-11-22 11:28 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 10:39 week 9.4 % (6.1 + 3.3; under pace 17.8 %: 6 lanes; WM-TESTS after ti-38)
session branch: 3b14c825 (t107 green 08:47: TOUR-FIX r3 + ASK-ONE); 62 greens. t108 GATE-ALIVE a7ffdf22 rerun 10:50 (2 h cap).
burn-down: F1-F4, F6, F8 met (audit r4); F5 met by flake-series r4 16:21 (5/5 at 2 jobs, one host; rc 9 rows open); F7 open, carried.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R5-CO 18:10: fails T1, T2 (by mouse in a face), I3 MAJOR co-43 (pre-existing); tiling r4 23:36 NO: I2 x13 (ti-31), T1 xp-owned window (ti-32 verify), T3 (ti-33); L2, T7 vacuous |
| 3 | Three greens in a row | 30b, 33-40 green (9); latest 88b, 89b |
| 4 | CI green | green on 29e2cd77 13:13 |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59 and delta r1 16:11 (to 1c599c74): MERGE-OK, 0 MAJOR; rd-2 MEDIUM in xp-rd |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | t14, t17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| ti-38-r2.md (F7, rc 2: ti-38 MAJOR; r1 4441f1d4 done, merge 7a6c0eb8 conflicts, construct act rerun; 1.5 %) | opus high, claude/ti-38 4441f1d4 | session_016iS3kMg9i8mAmHNyiis9cz 07:34 |
| log-down-mc-r1.md (rc 9: mc eprintln 27, lc-2; 0.9 %) | sonnet medium, claude/log-down-mc 70cbf9bd | session_016iS3kQGuYZia3PrtJKkqgJ 10:55 |
| core-diag-r2.md (rc 9: sl-3 band, unlink; sl-1 mallopt; 2 %) | opus high, claude/core-diag fc779a76 | session_016iS3kQcXW93puoYVHWyYxt 10:40 |
| mesh-churn-r1.md (rc 9: sl-1 texture churn, malloc_trim; 1.5 %) | opus high, claude/mesh-churn 96e5c804 | starting |
| split-b-r1.md (F4: voice core, agents 1,420, 1,410; 1.6 %) | sonnet medium, claude/split-b 3b14c825 | session_016iS3kQcGNziaNKrScfwCsx 09:06 |
| flake-series-r5.md (F5, rc 3: 5 gates at 2 jobs; 1 %) | haiku high, claude/flake-series-r5 3b14c825 | session_016iS3kPEG2bDTyKBSQgj5BM 08:49 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) TOUR-FIX r3 85c5f3bc and ASK-ONE 322eef20 merged in t107 (3b14c825); next tour round after ti-38 r2 merges.
5. t108 10:50 on 3b14c825: GATE-ALIVE a7ffdf22 alone; lanes merge head; t109 LAUNCH-TIMER 629faf7b + TEST-GAPS 89dc32c4 + LOG-DOWN-CO b974134e + SIM-LONG ce451499 + LOOKS1-MC b0453e19 + SPLIT-C 96e5c804 (PM MERGE-OK); then RECORD r2.
6. OPT-IN r1 done (six PASS; oi-2, oi-4 to DB21). FLAKE-SERIES r5 (rc 3, F5; 1 %) runs on 3b14c825.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
