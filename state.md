# State

## Now (2026-11-17 03:23 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 52 % at 01:46 Tue (owner; Fable 7 %); tokens.py 03:22: 1.1 % (PM container). Rise was the Fable lane (owner 01:49: relax, opus ok); 48 % for 98 h = 0.49 %/h.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 39fe12d2 (train 34 03:19: mc r6); CI 10/10 on 4105f8d1. Next: testwatch after train 35.
burn-down 03:19: 0 BLOCK, 7 MAJOR in 4 lanes (mc merged, construct and tiling in train 35, xp r6). Likely tag Tue 11-24.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; 7 MAJOR (review-tour-v5) in 4 lanes |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33, 34 green (3); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 4105f8d1 23:50 (10/10 jobs) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v5 reviewed (FIX, reports/review-tour-v5.md); tour v6 after the 4 lanes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| tiling-look-r3.md (rc 1) | sonnet med, claude/tiling-look | aa245abd MERGE-OK (gate line); j1-4, j1-5, rv5-1 tour harness: check in tour v6; train 35 |
| xp-polish-r6.md (rc 1) | sonnet med, claude/xp-polish | c7e4e59e MERGE-OK 03:23 (3 MINOR); train 36 after 35 |
| construct-r2.md (rc 1) | sonnet high, claude/construct |  59a16702 MERGE-OK (2 MINOR); train 35 running since 03:19 |
| mc-r6.md (rc 1) | sonnet high, claude/mc | merged train 34 03:19; worker archived |
| testwatch-r1.md (rc 3) | haiku low, session branch | 4105f8d1 3/3 x3 (23:53); idle |
| older rounds | | merged or archived |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: lanes tiling r3, xp r5, construct r2, mc r6 (rc 1); then reviews, trains, tour v6 (rc 11).
2. UX loop (rc 1, 11): tour v6 (media r4) after the 4 lanes; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
