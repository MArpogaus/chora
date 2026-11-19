# State

## Now (2026-11-19 21:03 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 20:26 PM 1.2 % new Bangkok day (16:58: 5.8 %); 32 h left: 75 % at 2.3 %/h; use the week.
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need
session branch: 219d5afe (train 72: cube-tests, mx1-m2, docs-f8d, green 21:08); 37 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, DEBT-DOWN, CUBE-TESTS, MX1-M2 in.
burn-down: F1, F8 done; F2-F5, F7 open; F6 partly; 1.9 days. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next week.

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
| kit-report-r1.md (F2, F4) | sonnet high | MERGE-OK 582ab506, train 73 |
| lint-reason-r1.md (F4) | haiku high | MERGE-OK 1f99a98f, train 73 |
| minor-sweep-r1.md (F5) | sonnet high | MERGE-OK dfa8ddd8, train 73 |
| flake-h-r1.md (F5) | sonnet high | session_016iS3kLiNdKN9zCqfAGKuUt (20:30) from 219d5afe |
| kit-minors-r1.md (F2, F5) | sonnet high | MERGE-OK 88f441e4, train 73 |
| minors-2-r1.md (F4, F5) | sonnet high | MERGE-OK 8e677c1c, train 73 (holds dfa8ddd8, 1f99a98f) |
| id-gate-r1.md (F6) | sonnet high | session_016iS3kPzqf25zVAmDLkSHnx (20:53) |
| flake-i-r1.md (F5) | sonnet high | session_016iS3kPo5jVnFwva6MmkaoG (20:53) |
| flake-j-r1.md (F5) | sonnet high | starting from 219d5afe |
| kit-connect-r1.md (F2) | sonnet high | session_016iS3kLJr2GhWMz7EpzMLch |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 71 green 20:12. 72 green 21:08. 73: lint-reason 1f99a98f, kit-report 582ab506, minor-sweep dfa8ddd8, flake-h, kit-minors 88f441e4, minors-2 8e677c1c.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: FLAKE-J 0.7 %, ID-GATE 0.5 %, FLAKE-I 0.8 %, KIT-CONNECT 0.8 %, FLAKE-H 1 %; done: MINORS-2, LINT-REASON, MINOR-SWEEP, KIT-MINORS, KIT-REPORT; AUDIT-F r3 sonnet 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
