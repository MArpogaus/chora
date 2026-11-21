# State

## Now (2026-11-21 19:05 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 17:45 week 6.1 % (0.44 %/h since reset, under pace 8.3 %; 6 lanes); disk: one train at a time
session branch: 892d035c; 58 greens; t101 green 17:45 (7 branches); t102 running. F5 series r4 5/5 green at --jobs 2 (16:21).
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
| keys-route-r1.md (rc 2, Queue 5: stray keys) | sonnet high, claude/keys-route 892d035c | session_016iS3kLf6vLYHFxmrD54UXA 17:48 |
| co-i3-r2.md (rc 2: I3 MAJOR co-43, state.rs:66) | sonnet medium, claude/co-i3 afd63654 | session_016iS3kN5DHMzYVqzbjQCKev 18:47 |
| kit-cov-r1.md (F5: kit coverage, row 42) | haiku high, claude/kit-cov 892d035c | session_016iS3kMkZ848r7NTUByYjVq 17:54 |
| tour-fix-r1.md (F7: tour dumps, old replay, 90 reds) | sonnet high, claude/tour-fix 892d035c | session_016iS3kQ73xbF6fr8jWty7bN 18:26 |
| split-a-r1.md (F4 headroom: mc, assistant) | sonnet medium, claude/split-a 892d035c | starting |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 r1 blocked 18:25 (no dumps, no old replay, 90 reds); TOUR-FIX 1.2 %, then RECORD r2; t102 running on 892d035c: GATE-DRIFT 96a91f1f, DEPS a606a00f, XP-RD 2dfc7507; t103 next (harness): H-GUARDS 9b25d65e, P2-HARNESS-L 1bf67e98, TI-XDG 2024358f.
5. (rc 2-3, process; week-3 D1-D2) running: GATE-WAITS 1.6 %; KIT-COV 0.6 %; KEYS-ROUTE 1.6 %; CO-I3 r2 0.8 %; next as lanes free: P2-R4-TILING 0.6 % (after t103); SPLIT-A 1.6 %; LOOP-SLEEPS after keys-route (look held); rows 37-39 stale; REVIEW 1 % after RECORD; t104: CO-HINT 51475800 (MERGE-OK).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
