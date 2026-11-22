# State

## Now (2026-11-22 02:23 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 02:20 week 7.8 % (6.1 + 1.7 Bkk days; 0.35 %/h, under pace 13.4 %: more lanes)
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
| clock-width-r1.md (F5, rc 3: t104b date-dependent clock check) | sonnet medium, claude/clock-width 1c9e64f7 | session_016iS3kLT7bZFcwTSG5m6Srg 00:38 |
| tour-fix-r2.md (F7, rc 11: review r1 FIX rw-1..3 MAJOR) | sonnet high, claude/tour-fix faf958fe | session_016iS3kMNfyagdg57Hy78L29 00:47 |
| ti-r4-r1.md (F7, rc 2: ti-32 MAJOR verify, ti-33) | sonnet high, claude/ti-r4 1c9e64f7 | session_016iS3kM7GXTMuqxfoDNWLxn 02:23 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 r1 blocked 18:25; TOUR-FIX done 00:33 faf958fe (90 to 66 red, 9 findings); REVIEW r1 FIX 00:45 (3 MAJOR); TOUR-FIX r2 0.8 %, then train, RECORD r2.
5. (rc 2-3, process; week-3 D1-D2) MERGE-OK (PM): P2-HARNESS-M 4d738ce8, SH-32 eb831349 (I14 0/30), GATE-WAITS 3c5b29c9; t105 = CLOCK-WIDTH + these 3; TI-R4 1.5 %; next: LOOP-SLEEPS; then t104c: KEYS-ROUTE e16d558e, CO-HINT, KIT-COV, SPLIT-A, CO-I3 042a9c77.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
