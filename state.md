# State

## Now (2026-11-19 14:19 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week reset 05:55 Thu (owner); owner 13:50: 25 % week; tokens.py 13:49 PM 4.6 %; 38 h left: 75 % at 2 %/h (10:39-13:5x ran 2.5 %/h); 6 lanes; use the week (06:04).
models (owner 06:56): haiku tests; sonnet routine, port reviews; opus complex, planning, core reviews; Fable by need.
session branch: 05878d2a (train 63: flakes-f); 30 greens; MV1-4, CUT1-2, NET1-3, KIT1-2, KIT2b-c, KIT4, KIT6, WM1-2, WM1b-d, DOCS-F8, F8b in.
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
| kit3-r2.md (F2) | sonnet high | session_016iS3kNpggngKGiK1zhQ6qQ (13:56; review r1 FIX: rk-1 MAJOR, 5 MINOR) |
| integ-r1.md (F6) | sonnet high | session_016iS3kNXpgUxiE2BdHxpC8S (13:37) |
| focus-port-r1.md (F2, F3, F7) | opus high | starting (audit rank 3; from integ 3619ccb3) |
| moved-tests-xp-r1.md (F5) | haiku high | starting (audit rank 5, xp half) |
| flakes-g-r3.md (F5) | opus high | session_016iS3kLbyuHiuqm149tXExQ (13:58; step 0 setup.sh; adds pull flake) |

## Queue (% of week; 5 % reserve)
4. Trains (F6): 63 green 14:11 (05878d2a). 64 red 14:18: wm3 2 unit tests. 64b = wm7b + kitcov + cubesplit gating; wm3 back to integ (its gate is red too); kit5b, kit7 on old head, wm5 vs wm3 conflict: integ r1 (sonnet 2 %) merges them, then train 65.
5. audit-f (3 of 8 met; 12.3 %): focus-port (F2 3.5 %) and moved-tests xp (F5 1.25 %) start 14:19; after 64b moved-tests rest (haiku 1.25 %); then DEBT-R (F4 0.8 %), COPIES-0 after focus-port (F2 2.5 %), FLAKE-SERIES after flakes-g (F5 1 %), DOCS-F8c last (1 %); kit3 r2.
6. (process) 14:55 decisions (owner 15:35): D87 revised, D92 kept, no Fluent; cube form after WM7a; records 1-14 with docs F8.

## Needs owner
- Delete claude/release-rc4 (403). rc.4 tag: kept local (owner).
