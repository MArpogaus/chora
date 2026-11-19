# State

## Now (2026-11-19 22:19 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 22:10 PM 2.1 % new Bangkok day (16:58: 5.8 %); 29 h left: 75 % at 2.6 %/h; use the week; 6 lanes full.
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need
session branch: a9e7084d (train 74: id-gate, gate-pick, green 22:13); 38 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, DEBT-DOWN, CUBE-TESTS, MX1-M2, ID-GATE, GATE-PICK in.
burn-down: F1, F8 done; F2-F5, F7 open; F6 partly; 1.9 days. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next week.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); findings to known issues |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 29e2cd77 13:13 (train 41, PR #106, all 10 checks) |
| 5 | Packaging smoke | arch, rpm, podman green 29e2cd77 |
| 6 | Code review per area | 08:59, no HIGH; MEDIUM fixed or known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; D78-D95 train 37; cz changelog at the tag |
| 9 | Logging audit, flakes | train 14, 17 |
| 11 | Tour, UX loop | v7 MERGE-OK; no tour work (owner 11-17 14:20); new tour with checks after refactor and user test (next week) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| kit-report-r1.md (F2, F4) | sonnet high | MERGE-OK 582ab506, train 76 |
| flake-h-r1.md (F5) | sonnet high | session_016iS3kLiNdKN9zCqfAGKuUt (20:30) from 219d5afe |
| kit-minors-r2.md (F2, F5) | sonnet medium | MERGE-OK e4208314, train 76 |
| minors-2-r1.md (F4, F5) | sonnet high | MERGE-OK 8e677c1c, train 76 |
| flake-j-r1.md (F5) | sonnet high | session_016iS3kMAsaXSn33qWo4iN2k (21:03) |
| ig-fix-r2.md (F6) | sonnet low | MERGE-OK b717a5f2, train 76 |
| kit-connect-r2.md (F2) | sonnet low | session_016iS3kQ2e4ZUb6Eqrn35W2n (22:19) |
| temp-dirs-r2.md (F2, F5) | sonnet medium | session_016iS3kMvjHveBkn49PM1Z26 (22:11) |
| flake-k-r1.md (F5) | sonnet high | session_016iS3kM7FnHjyUxMp89TMzH (21:43) |
| f24-rest-r1.md (F5) | sonnet medium | session_016iS3kM5Mgv1MSREkhxtFPU (22:11) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 74 green 22:13. 75 red 22:18 (ci-check copies: kit-connect vs cube-tests). 76 running 22:20: ig-fix, kit-report, minors-2, flake-i, kit-minors. 77: kit-connect r2 + kc-fix, then lanes as they pass.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: FLAKE-K 0.7 %, F24-REST 0.6 %, TEMP-DIRS r2 0.4 %, KIT-CONNECT r2 0.3 %, FLAKE-J 0.7 %, FLAKE-H 1 %; done 21-22 h: 8 lanes (reports/); AUDIT-F r3 sonnet 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
