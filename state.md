# State

## Now (2026-11-22 12:46 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day (90.2 % in 135 h), no day cap; tokens.py 12:45 week 9.8 % (6.1 + 3.7; under pace 19.7 %; lanes wait on gates: add reviews, rc 4, 6)
session branch: 3b14c825 (t107 green 08:47: TOUR-FIX r3 + ASK-ONE); 62 greens. t108 GATE-ALIVE a7ffdf22: 45 PASS 12:45.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC (plans/week-3.md 1; rc.4 waiver gone)
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-38 r2; RECORD-V9 |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | open: flake-hint, series r5 |
| 4, 5 | CI green on a PR; packaging | open: pre-commit red; CI-HEAD (sonnet low 0.7 %) |
| 6 | Review per area | open: 34+ merges since adf71956; REVIEW-DELTA (opus 1.5 %) |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | open: DOCS-RC |
| 9 | Flakes, window loss | open: core-diag, mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| ti-38-r2.md (F7, rc 2: ti-38 MAJOR; r1 4441f1d4 done, merge 7a6c0eb8 conflicts, construct act rerun; 1.5 %) | opus high, claude/ti-38 4441f1d4 | session_016iS3kMg9i8mAmHNyiis9cz 07:34 |
| flake-hint-r1.md (F5, rc 3: ldm-2 hint-click act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| core-diag-r2.md (rc 9: sl-3 band, unlink; sl-1 mallopt; 2 %) | opus high, claude/core-diag fc779a76 | session_016iS3kQcXW93puoYVHWyYxt 10:40 |
| mesh-churn-r1.md (rc 9: sl-1 texture churn; 1.5 %) | opus high, claude/mesh-churn 96e5c804 | session_016iS3kLrku4JTjEeufgN2oV 11:31 |
| log-down-xp-r1.md (rc 9: xp eprintln 18; 0.9 %) | sonnet medium, claude/log-down-xp 70cbf9bd | session_016iS3kPYbLrqZVkcAvctZ1S 11:36 |
| flake-series-r5.md (F5, rc 3: 5 gates at 2 jobs; 1 %) | haiku high, claude/flake-series-r5 3b14c825 | session_016iS3kPEG2bDTyKBSQgj5BM 08:49 |

## Queue (% week; 5 % reserve)
3. Next free slots: CI-HEAD (rc 4, 5; 0.7 %), REVIEW-DELTA (rc 6; 1.5 %), then RECORD-V9 (rc 11, F7; 1.2 %), DOCS-RC (rc 8).
4. (rc 1, F7) tour round after ti-38 r2 merges;
5. (rc 3; 0 %) t109 at --jobs 2 after t108: LAUNCH-TIMER 629faf7b, TEST-GAPS 89dc32c4, LOG-DOWN-CO b974134e, SIM-LONG ce451499, SPLIT-C 96e5c804, SPLIT-B 2af7801c, LOG-DOWN-MC b843bf5f, LOOKS1-MC b0453e19 (gates pending t109).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
