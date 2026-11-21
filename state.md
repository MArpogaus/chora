# State

## Now (2026-11-21 05:13 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 % (owner 14:39: 65 %); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 05:01 PM 3.4 % (day); disk 11 G free: one build (train) at a time; 5 lanes
session branch: a5b77488; 54 greens; t96b green 04:20 (persona-begin harness; lanes told to merge). t97 running from 04:23: flakes-q (conflict with co-hud-load: same fade fix, head's kept, both tests), flakes-p, chapters-b, mc-column.
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
| guide-checks-r6.md (F7: xp-task Tahoma row, I13 proof) | sonnet medium, claude/guide-checks 07aa97d4 | session_016iS3kMLifBUjoYFXgqaZ75 03:44 |
| co-keys.md (rc 2: co-38, co-40, co-41 MAJOR focus, sheet, key list) | sonnet high, claude/co-keys a76476be | session_016iS3kQPiKokwPWu5EF5vt7 04:26 (restart) |
| flakes-r.md (F5: FLK-DOORS-CO, FLK-MATRIX-S1, FLK-XP-FACES) | sonnet high, claude/flakes-r a5b77488 | session_016iS3kQqiW8EoPxjH4tvYxg 04:25 |
| chapters-a-r2.md (F7: review r1 FIX, rca-1 notice text, rca-2 I5 dialog; 3 MINOR) | sonnet medium, claude/chapters-a 8435d386 | session_016iS3kP8KLcJuSKhSkhQbso 05:01 |

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t97 running (flakes-q, flakes-p, chapters-b, mc-column); review-pm-fixes r1 MERGE-OK 05:13 (dcb77733, 1499cfdb; rpm-1 MINOR to backlog).
5. (F5, F7, rc 2) F5 not met (series r3 2/5 at --jobs 2); open FLK-XP-FACES, FLK-MATRIX-S1, FLK-DOORS-CO. GUIDE-CHECKS r5 PARTIAL (gate-lib red = lane sandbox rm guard: lanes skip it), r6 0.5 %. CO-KEYS 1.5 %. CHAPTERS-A review r1 FIX (2 MAJOR), r2 0.8 %. MC-COLUMN r1 MERGE-OK.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
