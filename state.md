# State

## Now (2026-11-21 04:06 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 % (owner 14:39: 65 %); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 04:05 PM 3.1 %; disk 11 G free: one build (train) at a time; 5 lanes
session branch: f4f49783; 53 greens; t95b green 02:33 (give-raise T7 MAJOR, co-hud-load co-35); t96 red 03:14 at doors (construct launcher-key missing; flake FLK-DOORS-CO, diff only tests/persona); t96b rerun same head.
burn-down: F1-F4, F6, F8 met (audit r4); F5, F7 open past the 11-21 03:59 deadline, carried into this week (owner 11-17 15:35).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r4, mc r5 meet; P2-R4-CO 03:32: fails on T2 (co-41 MAJOR, no key list by mouse or F1); tiling r3 22:03: 0 breaches; T7 window under xp fixed (give-raise, t95b); harness begin r2 in t96; xp Start (sh-23) not reproduced in full replay r3: known issue |
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
| co-keys.md (rc 2: co-38, co-40, co-41 MAJOR focus, sheet, key list) | sonnet high, claude/co-keys a76476be | starting |
| chapters-a.md (F7: xp, tiling, handover rows checked) | sonnet high, claude/chapters-a f4f49783 | session_016iS3kLLge6driZZFcqW4i5 02:29 |
| mc-column.md (rc 2: mc-54 MAJOR column not back after Super+n) | sonnet high, claude/mc-column a76476be | session_016iS3kMHtkD5w7RGAv7Txcp 03:28 |

## Queue (% week; 5 % reserve)
4. (F6, rc 2-3) t96b running (t96 red: FLK-DOORS-CO): persona-begin dad65a95 alone (harness); then tell lanes to merge; train 97: flakes-q fb85b156, flakes-p bd3ecad4, chapters-b a76476be (PM review 03:30); then dcb77733 and check.py to a seeded review.
5. (F5, F7, rc 2) F5 not met (series r3 2/5 at --jobs 2); open FLK-XP-FACES, FLK-MATRIX-S1, FLK-DOORS-CO. GUIDE-CHECKS r5 PARTIAL (gate-lib red = lane sandbox rm guard: lanes skip it), r6 0.5 %. CO-CAPTURE r1: co-36 not reproduced. CO-KEYS 1.5 %. CHAPTERS-A 1.2 %. MC-COLUMN 1 % (mc-54).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
