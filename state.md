# State

## Now (2026-11-19 22:53 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 22:10 PM 2.1 %; 29 h left: 75 %, 2.6 %/h; 6 lanes
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: a9e7084d (train 74); 38 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
burn-down: F1, F8 done; F2-F5, F7 open; F6 partly; 1.9 d. Sat (07:00): ~20 merges, 85-95 %; F1,3,4,6,8 likely, F5 open, F2 partly, F7 next wk.

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
| flake-h-r1.md (F5) | sonnet high | session_016iS3kLiNdKN9zCqfAGKuUt |
| kit-minors-r2.md (F2, F5) | sonnet medium | MERGE-OK e4208314, train 76 |
| minors-2-r1.md (F4, F5) | sonnet high | MERGE-OK 8e677c1c, train 76 |
| ig-fix-r2.md (F6) | sonnet low | MERGE-OK b717a5f2, train 76 |
| temp-rest-r1.md (F2, F5) | sonnet medium | MERGE-OK 2e8e67f3, train 78 |
| temp-dirs-r2.md (F2, F5) | sonnet medium | MERGE-OK 86f42d1f, train 78 |
| flake-k-r1.md (F5) | sonnet high | session_016iS3kM7FnHjyUxMp89TMzH |
| f24-rest-r1.md (F5) | sonnet medium | MERGE-OK 7231b51a, train 78 |
| pw-r1.md (P2) | sonnet high | session_016iS3kNj1dj1ApmjZmprkka |
| nits-3-r1.md (F5) | sonnet medium | MERGE-OK 0b5f8c34, train 78 |
| block-v-r1.md (RC1) | sonnet high | session_016iS3kL7pjJSN94pqbtmfcF |
| flake-l-r1.md (F5) | sonnet high | session_016iS3kNsnUdKFttt9hzxaMo |
| kit-sec-r1.md (F5) | sonnet high | session_016iS3kPTiu5i9QdTant26v4 |

## Queue (% week; 5 % reserve)
4. Trains (F6): 74 green 22:13. 76 rerun (rm-2 flake): ig-fix, kit-report, minors-2, flake-i, kit-minors. 77: harness-gp 66f384b4. 78: temp-dirs r2 (holds kc-fix), temp-rest 2e8e67f3, f24-rest, nits-3 0b5f8c34.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: FLAKE-L 0.7 %, FLAKE-K 0.7 %, PW 1.5 %, KIT-SEC 1 %, BLOCK-V 1.2 %, FLAKE-H 1 %; AUDIT-F r3 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
