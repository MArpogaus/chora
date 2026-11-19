# State

## Now (2026-11-19 13:48 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 10:39: 17 % week (another agent included); tokens.py 12:50 PM 3.9 %; 39 h left: about 2 %/h; 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: dd2a0aa3 (train 62: kit2d, wm7a); 29 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, DOCS-F8, F8b in.
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
| review-kit3-r1.md (F2) | sonnet high | starting (kit3 5b788c72 done 13:47, gate 41/41) |
| cubesplit-r1.md (F4) | sonnet high | session_016iS3kPYxR1GkrNbgQFs8ib (13:45) |
| integ-r1.md (F6) | sonnet high | session_016iS3kNXpgUxiE2BdHxpC8S (13:37) |
| flakes-g-r2.md (F5) | opus high | held: no setup x5 (13:18, 3 of 4 lanes since 13:00 had it); 14:00 UTC default |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 62 green 13:00 (head dd2a0aa3). 63 = flakes-f 847f0964 gating. 64 = wm3 e15155da + wm7b 99c04d54 (known-red resolved, fa457b29) + kitcov ea468e3 (kit 94.6 % lines, PM review ok); kit5b, kit7 on old head, wm5 vs wm3 conflict: integ r1 (sonnet 2 %) merges them, then train 65.
5. (F2, F3, F5) kit3 review r1 (0.5 %), then kit3 into the train after integ; Sat opus check lane measures F1-F8 on the head. Cut: WM4, WM6, NET4.
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- env (12:32 UTC): some new containers lack setup (apt mirror reset; wm7b r3 had it). Re-save Setup script (setup.sh 67f6a9f1, Azure). Default 14:00 UTC (21:00 Bkk): step 0 runs setup.sh itself; veto?
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
