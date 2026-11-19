# State

## Now (2026-11-19 06:06 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); tokens.py 06:06: PM 0.6 % today. Day cap lifted; use the week fully (owner 06:04), about 40 % a day, no waste.
models: haiku tests; sonnet builds; opus reviews, hard causes.
session branch: 4417ab86 (train 55: docs-f8); 23 greens (30b, 33-41, 43-55); MV1, MV2 x3, CUT1, NET1, NET2, KIT2, WM1, KIT4, KIT1, DOCS-F8 in.
burn-down: F1 done; NET1-2, KIT1-2, KIT4, WM1 in; F2-F5, F7, F8 open (F2: every look on kit, copies 0; F3: WM1b-d); F6 partly; 2.1 days.

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
| wm1b-r1.md (F2, F3) | sonnet high, claude/wm1b | 0ffc53a8 MERGE-OK (review r1, 3 MINOR); train 57 |
| kit2b-r2.md (F2) | sonnet medium, claude/kit2b | r1 FIX (rk-1 MAJOR); r2 session_016iS3kLM8gQdZfpG4r53S3j
| mv3-r1.md (F4) | sonnet medium, claude/mv3 | 917df9d0 MERGE-OK; train 56 05:16 |
| net3-r1.md (F7) | opus high, claude/net3 | session_016iS3kLZ9zKCda38a6cRs6a 05:56 |
| wm2-r1.md (F3) | opus high, claude/wm2 | session_016iS3kMAkdWBVDyLsYxLUjZ 05:56 |
| wm1c-r1.md (F2, F3) | sonnet high, claude/wm1c | starting 06:1x |
| kit6-r1.md (F2) | sonnet high, claude/kit6 | starting 06:1x |
| cut2-r1.md (F6) | sonnet high, claude/cut2 | starting 06:1x |

## Queue (cost in % of the week; 95 % to Sat 03:59, 5 % reserve)
4. Use the week (owner 06:04): 6 build lanes always full; each handback an opus review, a clean head its train (serial, batched).
5. Thu: trains 56-58; WM1c, KIT6, CUT2 now; then KIT2c, WM1d, KIT2d, MV4 (F2-F4). Fri: WM3, WM4, WM7a (opus, F3), KIT5, KIT3 (F2), WM6, NET4, docs F8b. Sat: recount, coverage, week report. Builds 30 %, reviews, r2 30 %.
6. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27: setup re-saved by owner 11-19 06:01; verify fuzzel >= 1.11 at the next lane start. Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
