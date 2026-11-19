# State

## Now (2026-11-19 20:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 20:26 PM 1.2 % new Bangkok day (16:58: 5.8 %); 32 h left: 75 % at 2.3 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: 6aa0bd37 (train 71: debt-down D117, green 20:12); 36 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, COPIES-0, DOCS-F8c, MK, MC-I5, CONSTRUCT-RED, DEBT-DOWN in.
burn-down: F1, F8 done; NET1-2, KIT1-2, KIT4, WM1 in; F2-F5, F7 open; F6 partly; 1.9 days. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next week.

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
| cube-tests-r1.md (F5) | haiku high | MERGE-OK 03d0bcfd, train 72 |
| mx1-m2-r1.md (F5) | sonnet high | MERGE-OK 5197fa22, train 72 |
| kit-report-r1.md (F2, F4) | sonnet high | session_016iS3kMTc9oNYxZLiK2tahV (20:13) from 6aa0bd37 |
| docs-f8d-r1.md (F8) | sonnet high | MERGE-OK c8249e2c, train 72 |
| lint-reason-r1.md (F4) | haiku high | session_016iS3kNum3AruGdhEDHT2iE (20:13) from 6aa0bd37 |
| minor-sweep-r1.md (F5) | sonnet high | starting 20:4x from 219d5afe |
| flake-h-r1.md (F5) | sonnet high | starting 20:4x from 219d5afe |
| kit-minors-r1.md (F2, F5) | sonnet high | starting 20:4x from 219d5afe |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 69 green 19:03; 70 green 19:32 (construct-red, known-red 0). 71 green 20:12 (debt-down, D117). 72 running 20:36: cube-tests, mx1-m2, docs-f8d. 73: kit-report, lint-reason, minor-sweep, flake-h, kit-minors.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); CUBE-TESTS, MX1-M2, DOCS-F8d done; running: MINOR-SWEEP 0.8 %, FLAKE-H 1 %, KIT-MINORS 0.8 %, KIT-REPORT 0.8 %, LINT-REASON 0.6 %; AUDIT-F r3 sonnet 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
