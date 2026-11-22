# State

## Now (2026-11-22 13:42 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.67 %/h = 16 %/day (90.2 % in 135 h), no day cap; tokens.py 12:45 week 9.8 % (6.1 + 3.7; under pace 19.7 %; lanes wait on gates: add reviews, rc 4, 6)
session branch: 6b01c976 (t108 green 13:01: GATE-ALIVE); 63 greens. t109 033f416f runs at --jobs 2 from 13:02.
burn-down: F1-F4, F6, F8 met (audit r4); F5 open again (critic 11-22: one series, hint act red twice; closes on flake-hint + haiku series 5 at 2 jobs on t109 head); F7 open.

## RC (plans/week-3.md 1; rc.4 waiver gone)
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-38 r2; RECORD-V9 |
| 2 | Persona round 4 | xp, mc meet; co T1 T2, tiling I2 T1 T3 open |
| 3 | 3 greens, no flake | open: flake-hint, series r5 |
| 4, 5 | CI green on a PR; packaging | pre-commit green on 6b01c976; CI PR at release |
| 6 | Review per area | r1 MERGE-OK 13:35 (0 MAJOR, rd-5 MEDIUM: MC-FOCUS); Thu round |
| 7 | deny, machete; updates | clean; 7 bumps (DEPS) |
| 8 | Docs, CHANGELOG, known issues | open: DOCS-RC |
| 9 | Flakes, window loss | open: core-diag (t110), mesh-churn, sim-long |
| 11 | Tour v9 | open = F7 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| ti-38-r2.md (F7, rc 2: ti-38 MAJOR; r1 4441f1d4, merge conflicts; 1.5 %) | opus high, claude/ti-38 4441f1d4 | session_016iS3kMg9i8mAmHNyiis9cz 07:34 |
| flake-hint-r1.md (F5, rc 3: ldm-2 hint-click act; 0.8 %) | sonnet medium, claude/flake-hint 3b14c825 | session_016iS3kLq6SXjYGB7qdMUEni 12:46 |
| review-mesh-churn-r1.md (rc 9: sl-1 fix, gate 49/49, 29 MB left; 0.5 %) | opus high, claude/mesh-churn dc6a266d | session_016iS3kNLUJtV2qEtsPm9qgY 13:41 |
| flake-series-r5.md (F5, rc 3: 5 gates at 2 jobs; 1 %) | haiku high, claude/flake-series-r5 3b14c825 | session_016iS3kPEG2bDTyKBSQgj5BM 08:49 |
| mc-focus-r1.md (rc 9, I3: rd-5 MEDIUM, mc misses focus.node; 0.8 %) | sonnet medium, claude/mc-focus 6b01c976 | session_016iS3kNytLH7EhG5sJybjcm 13:37 |
| record-v9-r1.md (F7, rc 11: v9 take vs v8; 1.2 %) | sonnet medium, claude/record-v9 6b01c976 | session_016iS3kPEnmatu6w5XewDQLy 13:33 |

## Queue (% week; 5 % reserve)
3. CI-HEAD dropped (pre-commit green on 6b01c976). Next free slot: DOCS-RC (rc 8; 0.8 %; plan D6).
4. (rc 1, F7) tour round after ti-38 r2 merges;
5. (rc 3; 0 %) t109 at --jobs 2 from 13:02 (033f416f): LAUNCH-TIMER 629faf7b, TEST-GAPS 89dc32c4, LOG-DOWN-CO b974134e, SIM-LONG ce451499, SPLIT-C 96e5c804, SPLIT-B 2af7801c, LOG-DOWN-MC b843bf5f, LOOKS1-MC b0453e19 (gates pending t109).
6. (rc 9) t110: CORE-DIAG 788605f4 (MERGE-OK 13:32; by 01:32), LOG-DOWN-XP ce11e056 (PM MERGE-OK 13:45; by 01:45); then eprintln ratchet down.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
