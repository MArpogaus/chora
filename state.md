# State

## Now (2026-11-21 08:46 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: new week 11-21 04:00 to 11-28 03:59 UTC (owner reset); old week ended near 70 %; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 08:22 week 4.7 % (1.1 %/h over 4.4 h, above pace; PM opus is most of it, lanes cheap: PM reviews short, lanes 2-4); disk 16 G: one build at a time; 6 lanes
session branch: 1c599c74; 56 greens; t98 green 08:46 (gate-path, xp-fields, chapters-a, xp-37, flakes-r). F5 series after t99 (blocks trains 5 h).
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

## Queue (% week; 5 % reserve)
4. (F7, rc 2-3) t99 running 08:46 on 1c599c74: guide-checks b1b8b73a, co-keys f70d564f (conflict camera.rs, known-red.toml resolved by PM: both sides kept), ti-29 1f9a5180, mc-minor 12869800. Then handover-mo r2.
5. (F5, F7, rc 2) F5 open (r3 2/5); FLK-XP-FACES open; series after t99; mc-50, rmm-2, rfr-1 open. Running: HANDOVER-MO r2 0.6 %.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
