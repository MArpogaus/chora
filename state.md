# State

## Now (2026-11-21 06:36 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 % (owner 14:39: 65 %); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 06:27 week 3.9 %; disk 18 G: one build at a time; 6 lanes
session branch: 598a34ff; 55 greens; t97 green 05:35 (flakes-q, flakes-p, chapters-b, mc-column; lanes told to merge). F5 series waits for flakes-r (a series now blocks trains for 5 h).
burn-down: F1-F4, F6, F8 met (audit r4); F5, F7 open past the 11-21 03:59 deadline, carried into this week (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R4-CO 03:32: fails on T2 (co-41 MAJOR, no key list by mouse or F1); tiling r3 22:03: 0 breaches; T7 window under xp fixed (give-raise, t95b); harness begin r2 in t96 |
| 3 | Three greens in a row | 30b, 33-40 green (9); latest 88b, 89b |
| 4 | CI green | green on 29e2cd77 13:13 |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | t14, t17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| guide-checks-r6.md (F7: Tahoma, I13) | sonnet medium, claude/guide-checks 07aa97d4 | session_016iS3kMLifBUjoYFXgqaZ75 03:44 |
| co-keys.md (rc 2: co-38, co-40, co-41 MAJOR) | sonnet high, claude/co-keys a76476be | session_016iS3kQPiKokwPWu5EF5vt7 04:26 |
| flakes-r.md (F5: FLK-DOORS-CO, FLK-MATRIX-S1, FLK-XP-FACES) | sonnet high, claude/flakes-r a5b77488 | session_016iS3kQqiW8EoPxjH4tvYxg 04:25 |
| ti-29-r1.md (rc 2 tiling: launcher order, helper panel covers tile) | sonnet medium, claude/ti-29 598a34ff | session_016iS3kMGHWcQ12HZmuhWrPd 06:25 |
| xp-37-r1.md (xp MINOR: rxf-1..4, 6; pixel round 2 of 2) | sonnet medium, claude/xp-37 9a6506d6 | session_016iS3kM5HiUCpS3mS1QRtvH 06:28 |
| gate-path.md (F5 harness: selftest under rm-guard PATH) | sonnet low, claude/gate-path 598a34ff | starting |

## Queue (% week; 5 % reserve)
4. (F6, F7, rc 2-3) t98 red 06:40, gate-lib (rm-guard on PATH breaks selftest; red on 598a34ff too); GATE-PATH 0.3 % fixes, then t98 with it: xp-fields 9a6506d6 + chapters-a 186d8506 (MERGE-OK). Then flakes-r, handover-mo r2.
5. (F5, F7, rc 2) F5 open (r3 2/5); FLK-XP-FACES, FLK-MATRIX-S1, FLK-DOORS-CO. Running: GUIDE-CHECKS r6, CO-KEYS, TI-29 0.8 %, XP-37 0.6 %. HANDOVER-MO review FIX 06:27 (rhm-1 MAJOR: bare `chora -- chora-tiling` has no bar): r2 after ti-29. Audit: BLOCK/MAJOR 0/3.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
