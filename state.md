# State

## Now (2026-11-18 11:54 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 62 % at 23:59 Tue (owner); tokens.py 10:32: PM 1.2 % since; lanes wm1 $12.6, kit4 $3.2, kit2 $2.0 (est 6 %). About 69 %, even pace 61 %: about 8 % over; 31 % for 65 h; lanes show the weekly warning. No new opus today (kit4 re-review was the last). Full reset (owner 15:35): near 95 %.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 5990b001 (train 53 10:31: kit4); 21 greens (30b, 33-41, 43-53); MV1, MV2 x3, CUT1, NET1, NET2, KIT2, WM1, KIT4 in.
burn-down: F1 done 14:55; NET1, NET2, KIT2 (mc), WM1 (tiling), KIT4 (ask: mc, construct; ask.rs 97.5 %) in; F2-F5, F7, F8 open (F2: every look on kit, copies 0; F3: WM1b-d); F6 partly; KIT1 next; 2.7 days.

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
| kit1-r1.md (F2) | sonnet high, claude/kit1 | edb94c25 handed back 11:52 ($2.7, worker archived); gate PASS but e2e apps-click (F27); PM llvm-cov running; opus security review Thu 00:05 (trig_01GtUuc) |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 42 % - 5 % reserve)
1. (process) Done: rc.4 on dev, main (13:41), untagged (owner 14:20); train 43 merged 14:42.
2. Done 20:35 (F4, F6): MV1+CUT1, MV2 x3 merged (trains 44-47); ratchet --update with the next lane.
3. Done 06:16: NET2 acts, I1 I3 I4 I7 I8 I14, known-red (trains 49, 50).
4. Now-Fri (F2, F3, 7 %): KIT2, WM1, KIT4 in (trains 51-53); KIT1 review Thu 00:05, train 54 (1.5 %); Fri NET3 (4.5 %), docs F8 (1.5 %). Hold new lanes today: 8 % over pace.
5. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
