# State

## Now (2026-11-17 04:29 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 53 % at 04:20 Tue (owner); tokens.py 03:22: 1.1 % (PM container). +1 % since 01:46 = 0.4 %/h; 47 % for 96 h = 0.49 %/h (opus ok, owner 01:49).
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: fe8ba6d0 (train 35 04:11: construct r2, tiling r3); CI green on fe8ba6d0 04:27. Next: train 36 xp, testwatch.
burn-down 04:11: 0 BLOCK; of 7 MAJOR 3 merged (mc 1, construct 2), xp 1 in train 36, tiling 3 to confirm in tour v6. Tag Fri 11-20 if v6 clean.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; 7 MAJOR (review-tour-v5) in 4 lanes |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33, 34, 35 green (4); F26 held, F28 holds; F17 open |
| 4 | CI green | green on fe8ba6d0 04:27 (train 35) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v5 reviewed (FIX, reports/review-tour-v5.md); tour v6 after the 4 lanes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| tiling-look-r3.md (rc 1) | sonnet med, claude/tiling-look | merged train 35 04:11; j1-4, j1-5, rv5-1 checked in tour v6 |
| xp-polish-r6.md (rc 1) | sonnet med, claude/xp-polish | c7e4e59e MERGE-OK (3 MINOR); train 36 running since 04:11 |
| construct-r2.md (rc 1) | sonnet high, claude/construct | merged train 35 04:11 |
| mc-r6.md (rc 1) | sonnet high, claude/mc | merged train 34 02:42; worker archived |
| decisions-r3.md (rc 8) | sonnet low, claude/decisions | r2 80a5931 in review; r3 marks D87-94 proposed (owner 04:41) |
| testwatch-r1.md (rc 3) | haiku low, session branch | 4105f8d1 3/3 x3 (23:53); idle |
| older rounds | | merged or archived |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: train 36 xp (rc 1); testwatch; tour v6 media r4 (rc 11); then construct r3 DB1 with v6 findings, decision records DB2 (rc 8).
2. UX loop (rc 1, 11): tour v6 (media r4) after the 4 lanes; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
