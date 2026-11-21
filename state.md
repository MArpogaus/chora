# State

## Now (2026-11-21 08:14 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 % (owner 14:39: 65 %); pace 0.6 %/h = 14 %/day, no day cap; tokens.py 07:30 week 4.4 % (1.3 %/h over 3.5 h, above pace: 5 lanes, sixth slot held); disk 16 G: one build at a time; 6 lanes
session branch: 598a34ff; 55 greens; t97 green 05:35. F5 series waits for flakes-r (a series now blocks trains for 5 h).
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
| handover-mo-r2.md (rc 2: review FIX, rhm-1 MAJOR bare tiling black) | sonnet medium, claude/handover-mo f7dded7a | session_016iS3kNLQayiUMXqd1yigH9 08:14 |
| mc-minor-r2.md (mc: PM review FIX, rmm-1 MAJOR lint ratchet raised) | sonnet low, claude/mc-minor fb9827a0 | session_016iS3kPHwmhVVmbb4d8eaN4 07:51 |

## Queue (% week; 5 % reserve)
4. (F6, F7, rc 2-3) t98 running 07:30 (gate-path fd8d6d0a MERGE-OK, PM review): xp-fields 9a6506d6, chapters-a 77134283, xp-37 d7b5376e (MERGE-OK 07:10), flakes-r 789aab53 (MERGE-OK 06:43; rfr-1 MINOR co). Then handover-mo r2.
5. (F5, F7, rc 2) F5 open (r3 2/5); FLK-XP-FACES open; series after t98. t99 after t98: GUIDE-CHECKS b1b8b73a (07:49), CO-KEYS f70d564f, TI-29 1f9a5180 (PM r2 OK). Running: HANDOVER-MO r2 0.6 %, MC-MINOR r2 0.3 % (r1 FIX: ratchet; mc-50 open).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
