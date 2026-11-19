# State

## Now (2026-11-19 09:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 06:52: 3 % week; tokens.py 08:52 PM 2.2 % (lanes not counted); 6 lanes since 07:40; use the week (06:04).
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
| wm3-r1.md (F3) | sonnet high | session_016iS3kMQapsj8QueGbfqBuP (09:18) |
| kit5-r1.md (F2) | sonnet high | session_016iS3kNgFvoGjshMusuMiZV (07:40) |
| wm7a-r2.md (F3) | sonnet high | session_016iS3kN6hpfMtLbawkpssbH (09:17) |
| kit2c-r2.md (F2) | sonnet high | session_016iS3kNz4FAJW7nH4GHrt8f (09:15) |
| kit2d-r1.md (F2) | sonnet high | session_016iS3kM1JMwzfGGNbUurCv7 (08:50) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 59 green 09:27; 60 mv4 7e339880, wm2 8db02220 running 09:33 (size list takes mv4); wm1d conflicts wm2 (tiling state.rs): r3 brief, next free slot after 60; kit2c r2 merges wm2; docs-f8b 5f97b0eb with kit2c. Pace 1.5-2 %/h.
5. Next (sonnet unless complex, owner 06:56): WM3 running 09:18; KIT3 (F2, after KIT2c-d). Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- Cube form (WM7b, 14:55 'after WM7a'): wm7a measured hosted works (reports/wm7a.md); default hosted, flat stays fallback; by 11-19 14:00 UTC.
- F27 closed 07:47: fuzzel 1.11.1 in a fresh lane (flakes-f). Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
