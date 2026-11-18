# State

## Now (2026-11-18 12:55 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 62 % at 23:59 Tue (owner); tokens.py 12:55: PM 1.4 % today; lanes wm1, kit4, kit2, kit1 $20.5 (7 %). About 70 %, even pace 61 %: 9 % over; 30 % for 63 h. No new opus today. Full reset: 15:35.
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
| 11 | Tour, UX loop | v7 MERGE-OK; no tour work (owner 11-17 14:20); new tour with checks after refactor and user test (next week) |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| kit1-r1.md (F2) | sonnet high, claude/kit1 | edb94c25 handed back 11:52 ($2.7, worker archived); gate stopped at e2e apps-click (F27), later steps not run; paths 93.1 %, words 98.5 % (PM llvm-cov); opus security review Thu 00:05 (trig_01GtUuc) |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 30 % - 5 % reserve)
1. (process) Done: rc.4 on dev, main (13:41), untagged (owner 14:20); train 43 merged 14:42.
2. Done 20:35 (F4, F6): MV1+CUT1, MV2 x3 merged (trains 44-47); ratchet --update with the next lane.
3. (F6) Done 06:16: NET2 acts, I1 I3 I4 I7 I8 I14, known-red (trains 49, 50).
4. Now-Fri (F2, F3, 7 %): KIT2, WM1, KIT4 in (trains 51-53); KIT1 review Thu 00:05, train 54 (1.5 %); Fri NET3 (4.5 %), docs F8 (1.5 %). Hold new lanes today: 9 % over pace. F3b-d, F4, F5, F7, user test: next week.
5. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
