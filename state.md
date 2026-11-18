# State

## Now (2026-11-18 07:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 62 % at 23:59 Tue (owner); tokens.py 06:16: PM 0.7 % since; lanes est 4.5 %. About 67 %, even pace 59 %: about 8 % over; 33 % for 70 h; a lane showed the weekly warning 05:15. Full reset available (owner 15:35): use near 95 %.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: de057ab3 (train 50 06:16: net2 adapters r2); 18 greens (30b, 33-41, 43-50); MV1, MV2 x3, CUT1, NET1, NET2 in.
burn-down: F1 done 14:55; safety net NET1, NET2 merged (plan 3); F2-F5, F7, F8 open (corrected 06:18: F2 needs every look on kit, copies 0); F6 partly; WM1, KIT2 lanes; 2.9 days.

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
| 11 | Tour, UX loop | v7 MERGE-OK 10:19; MINOR rounds, then final tour Thu |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| wm1-r1.md (F3) | opus high, claude/wm1 | 87221071 handed back 07:28 (gate ok but F27 apps-click); review 07:29; coverage: PM after train 51 |
| kit2-r1.md (F2) | sonnet high, claude/kit2 | eb12e379 handed back 07:00 (gate partial: F21 pull flake); review MERGE-OK 07:05 (6 MINOR to backlog); train 51 07:04 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 42 % - 5 % reserve)
1. (process) Done: rc.4 on dev, main (13:41), untagged (owner 14:20); train 43 merged 14:42.
2. Done 20:35 (F4, F6): MV1+CUT1, MV2 x3 merged (trains 44-47); ratchet --update with the next lane.
3. Done 06:16: NET2 acts, I1 I3 I4 I7 I8 I14, known-red (trains 49, 50).
4. Now-Fri (F2, F3, 8.5 %): WM1 tiling (opus), KIT2 mc running; then KIT1, KIT4 (mc, construct; opus security review); Fri NET3 (4.5 %), docs F8 (1.5 %).
5. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
