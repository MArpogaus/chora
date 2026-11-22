# State

## Now (2026-11-22 00:34 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 23:37 week 7.3 % (6.1 + 1.2 Bkk days; 0.37 %/h, under pace 11.8 %); disk: one train at a time
session branch: 1c9e64f7; 60 greens; t103 green 22:45; t104 red 23:59 only in acts tiling I14 (sh-32, 2nd hit, same act as t103); t104b rerun 00:04 (4 branches). F5 r4 5/5 green (16:21).
burn-down: F1-F4, F6, F8 met (audit r4); F5 met by flake-series r4 16:21 (5/5 at 2 jobs, one host; rc 9 rows open); F7 open, carried (owner 11-17 15:35).

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
| sh-32-r1.md (F5, rc 9: acts tiling I14 flake, 2 of 3 trains) | sonnet high, claude/sh-32 1c9e64f7 | session_016iS3kP3XbvGgWDcqL8QrGn 00:03 |
| p2-harness-m.md (rc 2: known-red filter, L2, T7 vacuous; r4 harness check) | sonnet medium, claude/p2-harness-m 1c9e64f7 | session_016iS3kMxK3mijyHf3skqCwg 23:38 |
| co-i3-r3.md (rc 2: review r2 rv-1 MAJOR, state.rs:65 owned-surface arm) | sonnet medium, claude/co-i3 2b571478 | session_016iS3kQftQgVnSgb4hmQy5h 21:51 |
| review-tour-fix-r1.md (F7: seeded review 892d035c..faf958fe) | sonnet high, claude/tour-fix faf958fe | starting 00:34 |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 r1 blocked 18:25; TOUR-FIX done 00:33 faf958fe (90 to 66 red, 9 findings); REVIEW-TOUR-FIX 0.5 % running, then train, RECORD r2.
5. (rc 2-3, process; week-3 D1-D2) running: GATE-WAITS 1.6 %; CO-I3 r3 0.6 %; P2-HARNESS-M 0.6 %; P2-R4-TILING done 23:36 (NO); SH-32 1 %; next after t104b: ti-32 check on the head, TI-R4 fix lane (ti-31-34), LOOP-SLEEPS; t104b running: KEYS-ROUTE e16d558e, CO-HINT, KIT-COV, SPLIT-A.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
