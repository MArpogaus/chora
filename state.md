# State

## Now (2026-11-22 06:43 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 06:43 week 8.5 % (6.1 + 2.4 Bkk days; under pace 16 %: 5th, 6th lane; WM-TESTS after ti-38, same xp wm files)
session branch: 7a6c0eb8 (t106 green 06:42, all 49 steps; CLOCK-WIDTH in); 61 greens.
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
| gate-alive-r1.md (F6, D91: B4, H3, T2, T3; 1.3 %) | sonnet medium, claude/gate-alive 1c9e64f7 | session_016iS3kNRAqJWwcFQdn1C6Nz 04:25 |
| launch-timer-r1.md (slice 39: F-2, GIVE_UP, StatusFile; 1.6 %) | sonnet high, claude/launch-timer 1c9e64f7 | session_016iS3kNfazT1Ae2LVddkKvB 06:17 |
| looks1-mc-r1.md (rc 1 looks: 10 mc known-red rows; 1.4 %) | sonnet medium, claude/looks1-mc 7a6c0eb8 | starting |
| sim-long-r1.md (rc 9: leaks, stalls over 3 h; 1.0 %) | haiku high, claude/sim-long 7a6c0eb8 | starting |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) TOUR-FIX r3 done 06:20 85c5f3bc (84 to 66 red, tiling I2 23 to 0); PM review r3 MERGE-OK; in t107 with ASK-ONE 322eef20 (PM review MERGE-OK; ao-1 to DB7 d).
5. t107 06:44 on 3b14c825 detached: TOUR-FIX 85c5f3bc + ASK-ONE 322eef20; then RECORD r2.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
