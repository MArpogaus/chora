# State

## Now (2026-11-18 02:33 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 62 % at 23:59 Tue (owner); tokens.py 02:22: PM 0.4 % since; lanes NET1, NET2 est 2 %. Even pace 57 %: about 7 % over; 36 % for 73 h. Full reset available (owner 15:35): use near 95 %.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: f31ff880 (train 48 01:40: net1 FakeCore + state.get); 16 greens (30b, 33-41, 43-48); MV1, MV2 x3, CUT1, NET1 in.
burn-down: F1 done 14:55 (arch-plan-fable.md, 2 reviews); F2 done 01:40 (net1, review MERGE-OK); F3-F8 open, 4 lanes; 3.5 days.

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
| net2-harness-r2.md (F3) | sonnet med, claude/net2-harness | r1 FIX 02:22 (2 MAJOR: setup leak, missing key passes); r2 start 02:22 |
| net2-r1.md adapters (F3) | opus high, claude/net2-adapters | 5da2712c handed back 02:33 (gate PASS, 3 new known-red); review 02:33; re-merge harness r2 first |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 42 % - 5 % reserve)
1. (process) Done: rc.4 on dev, main (13:41), untagged (owner 14:20); train 43 merged 14:42.
2. Done 20:35 (F4, F6): MV1+CUT1, MV2 x3 merged (trains 44-47); ratchet --update with the next lane.
3. Wed-Thu (F2 done 01:40 train 48; F3, 7 %): NET2 acts, I1 I3 I4 I7 I8 I14, known-red (opus, 2 lanes).
4. Thu-Fri (F4, F5, F8, 8.5 %): WM1 tiling, KIT1, KIT2 mc, KIT4; Fri NET3 if NET2 in, docs F8 (6 %).
5. (process) PM decisions 14:55 (owner agreed 15:35; P1 runs into next week): D87 revised (pidfd, no waitid) and D92 kept, no Fluent (KISS); cube form after WM7a; records 1-14 filed with docs F8.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
