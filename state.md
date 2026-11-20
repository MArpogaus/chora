# State

## Now (2026-11-20 01:54 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 01:45: 40 %; tokens.py 01:20 PM 4.3 %; 26 h left: 60 %, 2.3 %/h; fill 6 lanes
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: b97369d1 (train 80); 42 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
burn-down: F1, F8 done; F2-F5, F7 open; F6 partly; 1.9 d. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next wk.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | xp r1 red |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | train 14, 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| stray-grab-r1.md (RC sh-24, Queue 5) | opus high | session_016iS3kQiKWvRdXmmZ12b8Tw |
| black-pane-r1.md (RC sd-h1, Queue 5) | sonnet high | session_016iS3kPY5q7FRF2zYvKecmv |
| t81 (Queue 4, F6): kit-sec-r3.md, xp-majors-r1.md, mc-answer-r2.md, co-majors-r2 | sonnet | MERGE-OK 1c25af7e 81a29fe8 92f2e52f 8e24dcb6; gate 01:54 |
| p2-r1-tiling.md (RC 2, Queue 5) | sonnet med | session_016iS3kQJpeA8rRTKmK6JD4M |
| p2-r1-mc.md (RC 2, Queue 5) | sonnet med | session_016iS3kPdQowtx92y35qnBx4 |
| p2-r1-construct.md (RC 2, Queue 5) | sonnet med | session_016iS3kMEKgriQXGBac4N2cv |

## Queue (% week; 5 % reserve)
4. Trains (F6): t80 green 01:52 (persona-env, flake-h; b97369d1). t81 gate 01:54: kit-sec 1c25af7e, xp-majors 81a29fe8, mc-answer 92f2e52f, co-majors 8e24dcb6.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: STRAY-GRAB 1.5 %, BLACK-PANE 1 %; P2 r1 tiling, mc, construct 3.3 % started 01:55; P2 r1 xp: RC2 red, persona env fixed (t80), P2 r2 xp after t81; AUDIT-F r3 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
