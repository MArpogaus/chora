# State

## Now (2026-11-19 10:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 10:39: 17 % week (another agent included); tokens.py 10:40 PM 2.8 %; 41 h left: about 2 %/h; 6 lanes; use the week (06:04).
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
| wm3-r2.md (F3) | sonnet high | session_016iS3kNQwBx7TAFsVTz5juX (10:35) |
| wm7b-r1.md (F3) | opus high | session_016iS3kNrkiV5gM6vDZnWG4q (10:55) |
| kit5-r2.md (F2) | sonnet high | session_016iS3kL5zrc1QTWqPQ4hMJR (10:55) |
| review-wm5-r1.md (F3) | opus high | session_016iS3kN7WBzT8Nfjm51xb1h (10:42) |
| kit3-r1.md (F2) | sonnet high | session_016iS3kL6PAeoBoBoV5rR5f1 (10:06) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 60 red pull (F21 flake). 61 = 60 + kit2c, docs-f8b, wm1d on daf83391 (PM resolved conflicts), running 10:33. 62: kit2d 188b642d MERGE-OK (PM r2), wm7a f4e4f4d9 (r3 merge); wm3 FIX (I12 key) to r2. Pace 1.5-2 %/h.
5. Next (sonnet unless complex): all 6 lanes stopped 10:42-48 on the 5-h limit, resumed 10:53 (kit5, flakes-f past 250 k: hand back). WM7b hosted (owner 10:52), opus 4 %, after wm7a r3; KIT6 in kit2c; then KIT7. Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- F27 closed 07:47: fuzzel 1.11.1 in a fresh lane (flakes-f). Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
