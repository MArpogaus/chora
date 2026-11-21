# State

## Now (2026-11-21 06:00 UTC)
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
| chapters-a-r2.md (F7: rca-1, rca-2 MAJOR; 3 MINOR) | sonnet medium, claude/chapters-a 8435d386 | session_016iS3kP8KLcJuSKhSkhQbso 05:01 |
| handover-mo-r1.md (rc 2: sh-23, sd-h1, sd-h2, ca-4 two-output MAJORs) | opus medium, claude/handover-mo a5b77488 | session_016iS3kMiT7ZFUvvmZSp7dwY 05:16 |
| review-xp-fields-r1.md (rc 2: xp-24, ca-1; build 9a6506d6 done 05:57, 11 cells green) | sonnet high, claude/xp-fields 9a6506d6 | starting 06:02 |

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t97 green 05:35; next train: chapters-a r2, flakes-r after review.
5. (F5, F7, rc 2) F5 not met (series r3 2/5 at --jobs 2); open FLK-XP-FACES, FLK-MATRIX-S1, FLK-DOORS-CO. GUIDE-CHECKS r6 0.5 %. CO-KEYS 1.5 %. CHAPTERS-A review r1 FIX (2 MAJOR), r2 0.8 %. HANDOVER-MO 1.5 %, XP-FIELDS r1 done, review 0.5 %.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
