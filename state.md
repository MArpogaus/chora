# State

## Now (2026-11-21 06:25 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 % (owner 14:39: 65 %); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 05:35 week 3.6 %; disk 18 G free: one build (train) at a time; 6 lanes
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
| co-keys.md (rc 2: co-38, co-40, co-41 MAJOR) | sonnet high, claude/co-keys a76476be | session_016iS3kQPiKokwPWu5EF5vt7 04:26 (restart) |
| flakes-r.md (F5: FLK-DOORS-CO, FLK-MATRIX-S1, FLK-XP-FACES) | sonnet high, claude/flakes-r a5b77488 | session_016iS3kQqiW8EoPxjH4tvYxg 04:25 |
| review-chapters-a-r2.md (F7: rca-1..5) | sonnet medium, claude/chapters-a 186d8506 | session_016iS3kPH9fAtTCPw6p3DmZd 06:05 |
| ti-29-r1.md (rc 2 tiling: launcher order, helper panel covers tile) | sonnet medium, claude/ti-29 598a34ff | starting |
| review-handover-mo-r1.md (ca-4, sd-h2 fixed; sh-23, sd-h1 not red) | sonnet high, claude/handover-mo f7dded7a | session_016iS3kNaKFZRp8cMp7n3q8F 06:15 |

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t97 green 05:35; next train: xp-fields 9a6506d6 (MERGE-OK), chapters-a r2 after review; flakes-r later.
5. (F5, F7, rc 2) F5 not met (series r3 2/5 at --jobs 2); open FLK-XP-FACES, FLK-MATRIX-S1, FLK-DOORS-CO. GUIDE-CHECKS r6, CO-KEYS, reviews CHAPTERS-A r2, HANDOVER-MO r1 running; XP-FIELDS MERGE-OK (xp-37 MINOR next); LEDGER-AUDIT 06:24: BLOCK/MAJOR 0/3, lane-owned. TI-29 0.8 %.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
