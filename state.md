# State

## Now (2026-11-19 12:01 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 10:39: 17 % week (another agent included); tokens.py 10:40 PM 2.8 %; 41 h left: about 2 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: daf83391 (train 61: mv4, wm2, kit2c+kit6, wm1d, docs-f8b); 28 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, DOCS-F8, F8b in.
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
| review-kit7-r1.md (F2) | opus high | starting (kit7 c1dbc47b handed back 11:59) |
| wm7b-r2.md (F3) | opus high | session_016iS3kLKs3MjAmmEjvvYvnT (11:29) |
| kit5-r2.md (F2) | sonnet high | session_016iS3kL5zrc1QTWqPQ4hMJR (10:55) |
| kit3-r1.md (F2) | sonnet high | session_016iS3kL6PAeoBoBoV5rR5f1 (10:06) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 61 green 11:23 (pull passed). 62 = kit2d 188b642d + wm7a f4e4f4d9: gate died 11:4x (shell job), rerun 12:02. wm3 out: its kit::wm::open::Open clashes with wm7a state::Open (6 files); r3 merges head after 62. 63: wm5 f6c67200 MERGE-OK (PM r2); flakes-f 847f0964 MERGE-OK (opus r1).
5. Next (sonnet unless complex): WM7b hosted (owner 10:52), opus 4 %, after wm7a r3; KIT6 in kit2c; KIT7 opus 5 % started 11:04. Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- F27 closed 07:47: fuzzel 1.11.1 in a fresh lane (flakes-f). Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
