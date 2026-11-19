# State

## Now (2026-11-19 18:12 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 13:50: 25 % week; tokens.py 16:58 PM 5.8 %; 38 h left: 75 % at 2 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: 7d9ec18e (train 67: matrix-cap, green 17:57); 32 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, DOCS-F8, F8b in.
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
| matrix-cap-r1.md (F5, F6) | sonnet high | merged train 67 (17:57) |
| moved-tests-mk-r3.md (F5) | haiku high | done 18:00: c00dcf3e MERGE-OK (PM review 18:03); train 69 |
| debt-down-r2.md (F4) | sonnet high | session_016iS3kL9NWEvkDk3CywjPVD (17:33; review r1 FIX: rd-1 MAJOR xp hears a construct event) |
| mc-i5-r1.md (F7, F2) | sonnet high | done 18:11: f7a69761 MERGE-OK (PM review 18:14); train 69 |
| construct-red-r1.md (F7, F3) | opus high | done 18:07: 28254eb6; opus review session_016iS3kMG9aRS9Wrv1CjuDEY (18:08) |
| docs-f8c-r1.md (F8) | sonnet high | done 17:48: 65ef41a5 MERGE-OK (PM review 17:50); train 68 |
| flake-series-r1.md (F5) | haiku high | session_016iS3kMzD9HDCezM7N5L3aD (18:00) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 65 green 16:57 after one rerun. 66 red 17:14 (MX1); 67 green 17:57. 68 running from 17:57: mk r1 31df2bb9, mk r2 4904ef79, copies-0 ee53e09c, docs-f8c 65ef41a5 (F8 met at merge). 69 = mk r3 c00dcf3e, mc-i5 f7a69761 (+ construct-red if MERGE-OK).
5. audit-f (3 of 8 met; 12.3 %): DEBT-DOWN (F4 1.5 %) and COPIES-0 (F2 2.5 %) from 85764974; FLAKE-SERIES haiku from 7d9ec18e (F5 1 %, 18:00), DOCS-F8c started 17:14 (1 %); KNOWN-RED (F7 state part, 4 rows): MC-I5 (1 %) and CONSTRUCT-RED (opus 3 %) from ee53e09c.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
