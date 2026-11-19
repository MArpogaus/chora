# State

## Now (2026-11-19 23:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 22:10 PM 2.1 %; 29 h left: 75 %, 2.6 %/h; 6 lanes
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: a61e8eaa (train 76); 39 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
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
| flake-h-r2.md (F5; r1 FIX rw-1 no load proof) | sonnet medium | session_016iS3kNLhgqk1W1n2yc2oRb |
| temp-rest-r1.md (F2, F5) | sonnet medium | MERGE-OK 2e8e67f3, train 78 |
| temp-dirs-r2.md (F2, F5) | sonnet medium | MERGE-OK 86f42d1f, train 78 |
| flake-k-r1.md (F5) | sonnet high | MERGE-OK 9dde8523, train 78 |
| f24-rest-r1.md (F5) | sonnet medium | MERGE-OK 7231b51a, train 78 |
| pw-r2.md (P2; r1 review FIX rw-1, rw-2) | sonnet medium | session_016iS3kMxhcmP8FUsVMZMJEN |
| nits-3-r1.md (F5) | sonnet medium | MERGE-OK 0b5f8c34, train 78 |
| block-v-r2.md (mc-38) | sonnet medium | MERGE-OK ce230288, train 78 |
| mc-majors-r1.md (RC, mc-33-37) | sonnet high | session_016iS3kMSygrmeY6GnmekpAL |
| flake-l-r1.md (F5) | sonnet high | session_016iS3kNsnUdKFttt9hzxaMo |
| review-kit-sec-r1.md (F5; kit-sec cb2976c1 done) | opus high | session_016iS3kQ3zvBR5uTP72XiUtD |
| ti-majors-r1.md (RC, ti-22-24) | sonnet high | session_016iS3kN6CpA9ijuBr2jMwMF |

## Queue (% week; 5 % reserve)
4. Trains (F6): 76 green 23:06 (ig-fix, kit-report, minors-2, flake-i, kit-minors). 77 running: harness-gp 66f384b4. 78: temp-dirs r2 (holds kc-fix), temp-rest 2e8e67f3, f24-rest, nits-3 0b5f8c34, flake-k 9dde8523, block-v ce230288.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: FLAKE-L 0.7 %, PW r2 0.7 %, KIT-SEC review 0.5 %, MC-MAJORS 1 %, FLAKE-H r2 0.5 %, TI-MAJORS 1 %; AUDIT-F r3 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
