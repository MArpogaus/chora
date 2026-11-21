# State

## Now (2026-11-21 20:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 19:59 week 6.9 % (6.1 + 0.8 Bkk days; 0.43 %/h, under pace 9.6 %); disk: one train at a time
session branch: e87b0ff9; 59 greens; t102 green 19:23 (3 branches); t103 running (harness; conflict merge-gate.sh ci-check, union). F5 r4 5/5 green (16:21).
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
| review-keys-route-r1.md (rc 2: keys-route 541b92bb, core input) | opus high, read-only | session_016iS3kPR4ezTLYdQ6G8edrT 20:00 |
| co-i3-r2.md (rc 2: I3 MAJOR co-43, state.rs:66) | sonnet medium, claude/co-i3 afd63654 | session_016iS3kN5DHMzYVqzbjQCKev 18:47 |
| tour-fix-r1.md (F7: tour dumps, old replay, 90 reds) | sonnet high, claude/tour-fix 892d035c | session_016iS3kQ73xbF6fr8jWty7bN 18:26 |
| split-a-r1.md (F4 headroom: mc, assistant) | sonnet medium, claude/split-a 892d035c | session_016iS3kPvsXkDnbDWVEAfeG1 19:07 |
| gate-waits-r1.md (F5, rc 3: sleeps to waits in six flaky cells) | sonnet medium, claude/gate-waits 1c599c74 | session_016iS3kMaugq8mKyEhmsaZMe 14:01 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) RECORD-V9 r1 blocked 18:25 (no dumps, no old replay, 90 reds); TOUR-FIX 1.2 %, then RECORD r2; t103 running on e87b0ff9 (harness): H-GUARDS 9b25d65e, P2-HARNESS-L 1bf67e98, TI-XDG 2024358f; then lanes merge the session branch.
5. (rc 2-3, process; week-3 D1-D2) running: GATE-WAITS 1.6 %; RV-KEYS 0.5 %; CO-I3 r2 0.8 %; SPLIT-A 1.6 %; next as lanes free: P2-R4-TILING 0.6 % (after t103); LOOP-SLEEPS after keys-route (look held); rows 37-39 stale; REVIEW 1 % after RECORD; t104: CO-HINT 51475800, KIT-COV 619da642 (OK).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
