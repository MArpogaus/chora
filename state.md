# State

## Now (2026-11-19 17:04 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 13:50: 25 % week; tokens.py 16:58 PM 5.8 %; 38 h left: 75 % at 2 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: 85764974 (train 65: merge-65, integ, moved-tests-ct; green on rerun 16:57); 31 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, DOCS-F8, F8b in.
burn-down: F1 done; NET1-2, KIT1-2, KIT4, WM1 in; F2-F5, F7, F8 open; F6 partly; 1.9 days. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next week.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); open findings to known issues |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green on 29e2cd77 13:13 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; decision records D78-D95 train 37; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | train 14; flakes r5 train 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; no tour work (owner 11-17 14:20); new tour with checks after refactor and user test (next week) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| matrix-cap-r1.md (F5, F6) | sonnet high | session_016iS3kMpUWeyja15x89YQKL (14:51; base alone 235 s) |
| moved-tests-mk-r3.md (F5) | haiku high | session_016iS3kQE68SuUq1P3CTXdcg (17:02; r2 4904ef79 MERGE-OK: mc met; xp panels, pointer open) |
| debt-down-r1.md (F4) | sonnet high | session_016iS3kNwNGkLo2yS3ZYgvtV (16:14; F4 open, rows above RC) |
| review-copies-0-r1.md (F2) | sonnet high | session_016iS3kLPkCYLHDPZTSEoJrD (17:04; copies-0 ee53e09c: cut-check clean, gate 41/41) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 65 green 16:57 after one rerun (first run: matrix 248 s over its 240 s cap). 66 gate 16:58: 85764974 + moved-tests-mk 31df2bb9. Next: matrix-cap when MERGE-OK (own train first), then mk r2 4904ef79 (MERGE-OK), debt-down, copies-0.
5. audit-f (3 of 8 met; 12.3 %): DEBT-R done 16:13 (F4 open), DEBT-DOWN (F4 1.5 %) and COPIES-0 (F2 2.5 %) from 85764974; FLAKE-SERIES after matrix-cap merges (F5 1 %), DOCS-F8c last (1 %).

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
