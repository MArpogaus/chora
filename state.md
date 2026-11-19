# State

## Now (2026-11-19 07:07 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 06:52: 3 % week; tokens.py 06:52 PM 0.8 %; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: dc4bc46d (train 56: mv3); 24 greens; MV1, MV2 x3, CUT1, NET1, NET2, KIT2, WM1, KIT4, KIT1, DOCS-F8, MV3 in.
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
| wm1b-r1.md (F2, F3) | sonnet high, claude/wm1b | 0ffc53a8 MERGE-OK; train 57 06:16 |
| kit2b-r2.md (F2) | sonnet medium, claude/kit2b | 517ce830 MERGE-OK (PM r2); train 58
| net3-r1.md (F7) | opus high, claude/net3 | bc54896a; review-net3-r1.md |
| wm2-r2.md (F3) | sonnet high, claude/wm2 | r1 FIX (rw-1 MAJOR); session_016iS3kPxLbWirfzDNrMXErf |
| flakes-f-r1.md (F5) | opus high, claude/flakes-f | session_016iS3kPLuNxC9L4qF4UPzzP |
| wm1c-r1.md (F2, F3) | sonnet high, claude/wm1c | 5e72e4f9 MERGE-OK (1 MINOR backlog); train 58 |
| kit6-r1.md (F2) | sonnet high, claude/kit6 | session_016iS3kQVP9HrenFDKnEYDaa |
| cut2-r1.md (F6) | sonnet high, claude/cut2 | session_016iS3kNdkNBFAYBgQRp7tcg |

## Queue (% of week; 5 % reserve)
4. Trains (procedures.md): 57 a17e7858 (wm1b); 58: kit2b, WM1c, NET3 if MERGE-OK; WM2 after r2; then KIT6, CUT2 last. Pace 1.5-2 %/h.
5. Next (sonnet unless complex, owner 06:56): KIT2c after train 58, WM1d, KIT2d, KIT3, KIT5 (F2); MV4 region.rs, chat.rs (F4); WM7a (F3); docs F8b (F8). Sat: opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- F27: setup re-saved by owner 11-19 06:01; fuzzel >= 1.11 unchecked: next lane start check. Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
