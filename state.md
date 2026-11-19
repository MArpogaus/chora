# State

## Now (2026-11-19 19:03 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu; owner 13:50: 25 %; tokens.py 19:02 PM 0.7 % new Bangkok day (16:58: 5.8 %); 33 h left: 75 % at 2.3 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: b5bf800c (train 68: mk r1-r2, copies-0, docs-f8c, green 18:30); 33 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, COPIES-0, DOCS-F8c in.
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
| moved-tests-mk-r3.md (F5) | haiku high | done 18:00: c00dcf3e MERGE-OK (PM review 18:03); train 69 |
| debt-down-r3.md (F4) | sonnet high | r2 review FIX (rd-7 MAJOR: no ungrab on holder exit); r3 session_016iS3kNw5fSqkKfpSP1s3vT (18:26) |
| mc-i5-r1.md (F7, F2) | sonnet high | done 18:11: f7a69761 MERGE-OK (PM review 18:14); train 69 |
| construct-red-r2.md (F7, F3) | sonnet high | done 18:53: da68737d MERGE-OK (opus review r2 19:01; rc-4, rc-5 MINOR to backlog); train 70 |
| audit-f-r2.md (F1-F8) | opus high | starting 19:03: re-measure F, slice the rest |
| flake-series-r1.md (F5) | haiku high | session_016iS3kMzD9HDCezM7N5L3aD (18:00) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 67 green 17:57; 68 green 18:30 (mk r1-r2, copies-0, docs-f8c: F8 met). 69 running from 18:27: mk r3 c00dcf3e, mc-i5 f7a69761. 70 = construct-red da68737d (MERGE-OK 19:01), debt-down r3 if MERGE-OK by then.
5. F work (12.3 %): DEBT-DOWN r3 (F4 1 %), CONSTRUCT-RED r2 (F7, F3 1.5 %), FLAKE-SERIES haiku (F5 1 %); NET3-M1 closed by mc-i5, kit coverage 94.6 % (kitcov), moved modules at 70 % (mk r3); AUDIT-F r2 opus 2 % slices the rest.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
