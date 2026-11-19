# State

## Now (2026-11-19 05:56 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); tokens.py 05:56: PM 0.5 % today. Day cap lifted (owner 03:41); opus lanes with a reason, no waste.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 4417ab86 (train 55: docs-f8); 23 greens (30b, 33-41, 43-55); MV1, MV2 x3, CUT1, NET1, NET2, KIT2, WM1, KIT4, KIT1, DOCS-F8 in.
burn-down: F1 done 14:55; NET1, NET2, KIT2 (mc), WM1 (tiling), KIT4, KIT1 (paths, words) in; F2-F5, F7, F8 open (F2: every look on kit, copies 0; F3: WM1b-d); F6 partly; 2.1 days.

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
| docs-f8-r1.md (F8) | sonnet medium, claude/docs-f8 | merged train 55 (4417ab86) |
| wm1b-r1.md (F2, F3) | sonnet high, claude/wm1b | 0ffc53a8 MERGE-OK (review r1, 3 MINOR); train 57 |
| kit2b-r2.md (F2) | sonnet medium, claude/kit2b | review r1 FIX (rk-1 MAJOR: run/give new in construct); r2 session_016iS3kLM8gQdZfpG4r53S3j 05:43 |
| mv3-r1.md (F4) | sonnet medium, claude/mv3 | 917df9d0 MERGE-OK; train 56 05:16 |
| net3-r1.md (F7) | opus high, claude/net3 | session_016iS3kLZ9zKCda38a6cRs6a 05:56 |
| wm2-r1.md (F3) | opus high, claude/wm2 | session_016iS3kMAkdWBVDyLsYxLUjZ 05:56 |
| testwatch-r1.md (rc 3) | haiku low | idle |

## Queue (cost in % of the week; open lines <= 30 % - 5 % reserve)
2. Done (F4, F6): MV1+CUT1, MV2 x3 (trains 44-47).
3. (F6) Done: NET2 acts, known-red (trains 49, 50).
4. Now (F2-F4, F7, F8): kit2b r2; trains 56 mv3, 57 wm1b, 58 kit2b; NET3 (4.5 %), WM2 (4 %) opus. Next: KIT2c-d, WM1c-d (sonnet). F5, user test: next week.
5. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27: owner re-saves the setup script (11-19, steps sent). Delete claude/release-rc4 in GitHub (session push 403). rc.4 tag: kept local (owner).
