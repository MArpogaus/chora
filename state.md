# State

## Now (2026-11-19 10:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 06:52: 3 % week; tokens.py 09:49 PM 2.6 % (lanes not counted, 0.4 %/h PM); 6 lanes since 07:40; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: 4548feb3 (train 59: cut2, net3); 27 greens; MV1-3, CUT1-2, NET1-3, KIT1-2, KIT2b, KIT4, WM1, WM1b-c, DOCS-F8 in.
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
| flakes-f-r1.md (F5) | opus high | session_016iS3kM5cKSFS5yqPLLGoj7 (07:37) |
| review-wm3-r1.md (F3) | sonnet high | starting |
| wm7a-r3.md (F3) | sonnet high | starting |
| kit5-r1.md (F2) | sonnet high | session_016iS3kNgFvoGjshMusuMiZV (07:40) |
| wm5-r1.md (F3) | opus high | session_016iS3kQkLYiWbxmLbvc4Li4 (09:42) |
| kit2d-r2.md (F2) | sonnet high | session_016iS3kM7cNPJuNyXTPLzdbA (10:10) |
| kit3-r1.md (F2) | sonnet high | session_016iS3kL6PAeoBoBoV5rR5f1 (10:06) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 60 red only pull (F21 flake; alone PASS). 61 = 60 + kit2c, docs-f8b, wm1d on daf83391 (PM resolved D104, testing.rs, api.md), running 10:33. wm7a r3 (net3 cube acts merge) 10:30. Pace 1.5-2 %/h.
5. Next (sonnet unless complex, owner 06:56): WM3 review 10:30; KIT3 starting 10:10 (F2). Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- Cube form (WM7b, 14:55 'after WM7a'): wm7a measured hosted works (reports/wm7a.md); default hosted, flat stays fallback; by 11-19 14:00 UTC.
- F27 closed 07:47: fuzzel 1.11.1 in a fresh lane (flakes-f). Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
