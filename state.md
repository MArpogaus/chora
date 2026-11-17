# State

## Now (2026-11-17 10:52 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 57 % at 10:31 Tue (owner); tokens.py 10:52: 3.4 % (PM container). +2 % since 06:21 = 0.48 %/h, on pace; 43 % for 89 h = 0.48 %/h. Opus again from 12:00 (critic).
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 7e066694 (train 39 08:56: xp r7, tiling r4); CI green on 7e066694 09:18. Tour v7 running.
burn-down 10:52: owner 10:47: quick rc now (rc.md 3, 4, 5, 7, 8). Train 40 died 10:07 (container restart), rerun 10:52. Tag today after CI.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); open findings to known issues |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-39 green (8); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 7e066694 09:18 (train 39) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; decision records D78-D95 train 37; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop | v7 MERGE-OK 10:19; MINOR rounds, then final tour Thu |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| construct-r3.md (rc 1) | sonnet med, claude/construct | r3 MERGE-OK (3 MINOR) 10:03; train 40 10:02 |
| media-r5.md (rc 11, 1) | sonnet med, claude/media | v7 MERGE-OK 10:19 (0 MAJOR); archived |
| tiling-look-r4.md (rc 1) | sonnet med, claude/tiling-look | merged train 39 08:56 |
| xp-polish-r7.md (rc 1) | sonnet med, claude/xp-polish | merged train 39 08:56 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: train 40 rerun (rc 3); media r6 tour v8 baseline; known issues, CI PR, tag (rc 4, 5, 8). construct r4, xp r8, mc r7 cancelled.
2. After the tag: priorities-sprint1.md (planner) -> one Fable plan lane (1.5 h) -> refactor lanes sonnet/opus, haiku tests.
3. Wed-Thu: flakes F17, F22 (CI 06:00), F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
