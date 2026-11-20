# State

## Now (2026-11-20 01:36 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 01:20 PM 4.3 %; 28 h left: 75 %, 2.7 %/h; 6 lanes
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: bb16f2ff (train 79); 41 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
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
| flake-h-r2.md | sonnet med | b0564df2 in t80 |
| xp-majors-r1.md (RC) | sonnet high | MERGE-OK 81a29fe8 (review: 4 MINOR) t81 |
| mc-answer-r2.md (RC) | sonnet med | MERGE-OK 92f2e52f (PM review: 1 MINOR) t81 |
| persona-env-r1.md (RC2) | sonnet med | 370495ad in t80 |
| review-co-majors-r2.md (RC) | sonnet high | starting; co-majors r2 8e24dcb6 |
| kit-sec-r3.md | sonnet med | session_016iS3kNR1eMLFo3QLJq3f7X |

## Queue (% week; 5 % reserve)
4. Trains (F6): t79 green 01:19 (78b, mc-majors, ti-majors, flake-l; bb16f2ff). t80 = persona-env 370495ad + flake-h b0564df2: gate 01:20. t81: co-majors r2, kit-sec r3, xp-majors 81a29fe8, mc-answer 92f2e52f.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: REVIEW CO r2 0.5 %, KIT-SEC r3 0.8 %; P2 r1 xp: RC2 red, persona env fixed (t80), P2 r2 xp after t80; AUDIT-F r3 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
