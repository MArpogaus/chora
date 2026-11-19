# State

## Now (2026-11-19 19:18 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 19:02 PM 0.7 % new Bangkok day (16:58: 5.8 %); 33 h left: 75 % at 2.3 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: f2b7112a (train 69: mk r3, mc-i5, green 19:03); 34 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, COPIES-0, DOCS-F8c, MK, MC-I5 in.
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
| review-debt-down-r3.md (F4) | opus high | 121c30aa MERGE-OK 19:18 (rd-9 MINOR, rd-10 NIT to backlog); train 71 after 70 (D-ungrab = D117) |
| construct-red-r2.md (F7, F3) | sonnet high | done 18:53: da68737d MERGE-OK (opus review r2 19:01; rc-4, rc-5 MINOR to backlog); train 70 running 19:06 (known-red empty, D116 accepted) |
| audit-f-r2.md (F1-F8) | opus high | done 19:13: 5/8 met on P (F4 with DD); kit 91.2 % (llvm-cov); 6 slices |
| cube-tests-r1.md (F5) | haiku high | session_016iS3kLXCAumfbUY5Q6Dntk (19:14) from f2b7112a |
| mx1-m2-r1.md (F5) | sonnet high | session_016iS3kNt2MZUSRkTBzPphxj (19:14) from f2b7112a |
| flake-series-r1.md (F5) | haiku high | session_016iS3kMzD9HDCezM7N5L3aD (18:00) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 68 green 18:30; 69 green 19:03 (mk r3, mc-i5). 70 running from 19:06: construct-red da68737d (known-red 0 rows). 71 = debt-down r3 when MERGE-OK.
5. F work: DEBT-DOWN r3 (F4 1 %), FLAKE-SERIES haiku (F5 1 %); AUDIT-F r2 done: CUBE-TESTS haiku 0.8 %, MX1-M2 sonnet 0.3 % now; after train 71: KIT-REPORT 0.8 %, DOCS-F8d 0.4 %, LINT-REASON 0.6 %; AUDIT-F r3 sonnet 1 % Fri late.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
