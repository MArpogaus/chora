# State

## Now (2026-11-21 22:47 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 21:10 week 7.1 % (6.1 + 1.0 Bkk days; 0.41 %/h, under pace 10.3 %); disk: one train at a time
session branch: 1c9e64f7; 60 greens; t103 green 22:45 (rerun; first run acts flake sh-32); t104 running 22:45 (keys-route, co-hint, kit-cov, split-a). F5 r4 5/5 green (16:21).
burn-down: F1-F4, F6, F8 met (audit r4); F5 met by flake-series r4 16:21 (5/5 at 2 jobs, one host; rc 9 rows open); F7 open, carried (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R5-CO 18:10: fails T1, T2 (by mouse in a face), I3 MAJOR co-43 (pre-existing); tiling r3 22:03: 0 breaches |
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
| p2-r4-tiling.md (rc 2: tiling round 4, L1-L3 after harness-l) | sonnet medium, claude/p2-r4-tiling 1c9e64f7 | session_016iS3kQNiJYXxaMEnvcEvjD 22:46 |
| co-i3-r3.md (rc 2: review r2 rv-1 MAJOR, state.rs:65 owned-surface arm) | sonnet medium, claude/co-i3 2b571478 | session_016iS3kQftQgVnSgb4hmQy5h 21:51 |
| tour-fix-r1.md (F7: tour dumps, old replay, 90 reds) | sonnet high, claude/tour-fix 892d035c | session_016iS3kQ73xbF6fr8jWty7bN 18:26 |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 r1 blocked 18:25 (no dumps, no old replay, 90 reds); TOUR-FIX 1.2 %, then RECORD r2; t103 merged 22:45 (H-GUARDS, P2-HARNESS-L, TI-XDG); lanes told.
5. (rc 2-3, process; week-3 D1-D2) running: GATE-WAITS 1.6 %; CO-I3 r3 0.6 %; P2-R4-TILING 0.6 %; next: LOOP-SLEEPS after t104; REVIEW 1 % after RECORD; t104 running: KEYS-ROUTE e16d558e (r2 OK), CO-HINT 51475800, KIT-COV 619da642, SPLIT-A a3c14590 (OK).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
