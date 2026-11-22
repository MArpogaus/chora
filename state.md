# State

## Now (2026-11-22 06:20 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 06:16 week 8.4 % (6.1 + 2.3 Bkk days; under pace 15.6 %: 5th lane; WM-TESTS after ti-38, same xp wm files)
session branch: 1c9e64f7; 60 greens; t104 red 23:59 acts tiling I14 (sh-32); t104b red 00:36 only in tiling-look bar-font: clock 120 wide (limit 119), date-dependent check (Sunday); CLOCK-WIDTH, then t104c.
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
| ti-38-r1.md (F7, rc 2: ti-38 MAJOR, joining look takes the keyboard) | opus high, claude/ti-38 e86b3cfa | session_016iS3kP4PwZqxBu9ivmLzFw 04:14 |
| ask-one-r1.md (rc 6 security: DB7 a-c, RF3; 1.2 %) | sonnet high, claude/ask-one 1c9e64f7 | session_016iS3kNummjLTbC6uiUi5PU 04:24 |
| gate-alive-r1.md (F6, D91: B4, H3, T2, T3; 1.3 %) | sonnet medium, claude/gate-alive 1c9e64f7 | session_016iS3kNRAqJWwcFQdn1C6Nz 04:25 |
| launch-timer-r1.md (slice 39: F-2, GIVE_UP, StatusFile; 1.6 %) | sonnet high, claude/launch-timer 1c9e64f7 | session_016iS3kNfazT1Ae2LVddkKvB 06:17 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) TOUR-FIX r3 done 06:20 85c5f3bc (84 to 66 red, tiling I2 23 to 0); PM review r3 MERGE-OK; joins t107 after t106; then RECORD r2.
5. t105 killed at the 2 h task limit 04:33 (48/49 PASS, e2e PASS alone 04:37; --only never pushes); t106 04:38 on 7a6c0eb8 detached (no limit): t105 f9e2f3f6 + TI-R4 e86b3cfa, KEYS-ROUTE e16d558e, CO-HINT 51475800, KIT-COV 619da642, SPLIT-A a3c14590, CO-I3 042a9c77; TOUR-FIX after r3.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
