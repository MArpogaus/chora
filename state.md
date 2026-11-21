# State

## Now (2026-11-21 12:06 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 %; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 11:21 week 5.0 % (0.7 %/h over 7.4 h, on pace; PM opus most of it; lanes 2); disk 16 G: one build at a time; 6 lanes
session branch: 1c599c74; 56 greens; t98 green 08:46 (gate-path, xp-fields, chapters-a, xp-37, flakes-r). F5 series after t99c (blocks trains 5 h).
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
| co-keys-r3.md (rc 2, F7: t99 red, three construct acts) | sonnet medium, claude/co-keys f70d564f | session_016iS3kMdG5AZVg4Vkvm6R5t 09:59 |

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t99b red at agent (handover-mo test, fixed r3 73180551); t99c red at doors (tiling launcher-chat missing; FLK-DOORS-TILING-CHAT); t99d running 12:09 on 32a505db: guide-checks, ti-29, mc-minor, handover-mo 73180551. Then RECORD-V9 1.2 % on its head; CO-KEYS r3 0.6 % joins t100.
5. (F5, F7, rc 2) F5 series after t99b (r3 2/5, blocks trains 5 h); then REVIEW opus 1 % after RECORD; FLK-XP-FACES open; mc-50, rmm-2, rfr-1 open (MINOR, after the tour).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
