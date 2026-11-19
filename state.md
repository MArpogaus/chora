# State

## Now (2026-11-19 23:49 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 22:10 PM 2.1 %; 29 h left: 75 %, 2.6 %/h; 6 lanes
models (owner 06:56): haiku tests; sonnet routine; opus complex, planning; Fable by need
session branch: 30727c82 (train 77); 40 greens; MV1-4, CUT1-2, NET1-3, KIT1-7, WM1-2, WM1b-d, COPIES-0, DOCS-F8c-d, MK, MC-I5, CONSTRUCT-RED, CUBE-TESTS, MX1-M2,ID-GATE, GATE-PICK in.
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
| 11 | Tour, UX loop | v7 MERGE-OK; new tour with checks after P2 (owner 11-17) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| flake-h-r2.md (F5) | sonnet medium | session_016iS3kNLhgqk1W1n2yc2oRb |
| temp-rest-r1.md (F5) | sonnet medium | MERGE-OK 2e8e67f3, train 78 |
| temp-dirs-r2.md (F5) | sonnet medium | MERGE-OK 86f42d1f, train 78 |
| flake-k-r1.md (F5) | sonnet high | MERGE-OK 9dde8523, train 78 |
| f24-rest-r1.md (F5) | sonnet medium | MERGE-OK 7231b51a, train 78 |
| pw-r2.md (P2) | sonnet medium | MERGE-OK 5755b578, train 78 |
| p2-r1-xp.md (RC2; r1b no setup) | sonnet+haiku | session_016iS3kLP7F2X6bjVSt3KzWv |
| nits-3-r1.md (F5) | sonnet medium | MERGE-OK 0b5f8c34, train 78 |
| block-v-r2.md (mc-38) | sonnet medium | MERGE-OK ce230288, train 78 |
| mc-majors-r1.md (RC) | sonnet high | session_016iS3kMSygrmeY6GnmekpAL |
| flake-l-r1.md (F5) | sonnet high | session_016iS3kNsnUdKFttt9hzxaMo |
| kit-sec-r2.md (F5) | sonnet high | session_016iS3kNBZH9UzgeiP4vU5iG |
| ti-majors-r1.md (RC, ti-22-24) | sonnet high | MERGE-OK c9ed1cfd, train 79 |
| co-majors-r1.md (RC, co-19-21) | opus high | starting |

## Queue (% week; 5 % reserve)
4. Trains (F6): 77 green 23:39 (harness-gp). 78 running 050a85ca: temp-dirs, temp-rest, f24-rest, nits-3, flake-k, block-v, pw. 79: ti-majors c9ed1cfd.
5. F work: FLAKE-SERIES 5/5, kit 91.06 % (F5); running: FLAKE-L 0.7 %, P2-R1-XP 1.1 %, KIT-SEC r2 0.8 %, MC-MAJORS 1 %, FLAKE-H r2 0.5 %, CO-MAJORS 1.5 %; AUDIT-F r3 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403).
- Lane auto mode refused setup.sh (apt) 23:37: allow, or all tools in env setup? Default: skip it. rc.4 tag: kept local (owner).
