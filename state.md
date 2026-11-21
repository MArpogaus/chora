# State

## Now (2026-11-21 17:53 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 17:45 week 6.1 % (0.44 %/h since reset, under pace 8.3 %; 6 lanes); disk: one train at a time on the PM machine (not a lane limit); 6 lanes
session branch: 892d035c; 58 greens; t101 green 17:45 (7 branches); t102 running (gate-drift, deps, xp-rd). F5 series r4 5/5 green at --jobs 2 (16:21).
burn-down: F1-F4, F6, F8 met (audit r4); F5 met by flake-series r4 16:21 (5/5 at 2 jobs, one host; rc 9 rows open); F7 open, carried (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R4-CO 03:32: fails on T2 (co-41 MAJOR, no key list by mouse or F1); tiling r3 22:03: 0 breaches |
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
| co-hint-r1.md (rc 2, F7: co-42 hint) | sonnet medium, claude/co-hint c69354ae | session_016iS3kL6rhVtqF6gejqFJ7c 14:10 |
| keys-route-r1.md (rc 2, Queue 5: stray keys) | sonnet high, claude/keys-route 892d035c | session_016iS3kLf6vLYHFxmrD54UXA 17:48 |
| p2-r5-co.md (rc 2: construct persona round 5) | sonnet medium, claude/p2-r5-co 892d035c | session_016iS3kMgXGsbJ55KX4ybyps 17:50 |
| kit-cov-r1.md (F5: kit coverage, row 42) | haiku high, claude/kit-cov 892d035c | starting |
| record-v9-r1.md (F7, rc 11: tour v9 take and diff) | sonnet medium, claude/record-v9 892d035c | session_016iS3kP27Gxp9nLotrS2Xgm 17:46 |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 1.2 % on 892d035c (t101 head); t102 running on 892d035c: GATE-DRIFT 96a91f1f (harness first), DEPS a606a00f, XP-RD 2dfc7507 (PM review MERGE-OK); t103 next (harness): H-GUARDS 9b25d65e, P2-HARNESS-L 1bf67e98, TI-XDG 2024358f (PM reviews MERGE-OK).
5. (rc 2-3, process; week-3 D1-D2) running: GATE-WAITS 1.6 %; KIT-COV 0.6 %; KEYS-ROUTE 1.6 %; P2-R5-CO 0.8 %; next as lanes free: P2-R4-TILING 0.6 % (after t103); LOOP-SLEEPS (after co-hint); rows 37-39 stale; REVIEW 1 % after RECORD; REVIEW-DELTA r2 on the Thu head.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
