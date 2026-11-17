# State

## Now (2026-11-17 11:49 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 57 % at 10:31 Tue (owner); tokens.py 11:49: 4.0 % (PM container). +2 % since 06:21 = 0.48 %/h, on pace; 43 % for 89 h = 0.48 %/h. Opus again from 12:00 (critic).
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 733967aa (train 40 11:47: construct r3); 9 greens (30b, 33-40); CI on PR at the tag.
burn-down 10:56: rc today (rc.md 3, 4, 5, 7, 8); week goal after the tag: design/foundation.md F1-F8 by Sat 03:59.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | waived for this rc (owner 10:47); open findings to known issues |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-40 green (9); F26 held, F28 holds; F17 open |
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
| construct-r3.md (rc 1) | sonnet med, claude/construct | merged train 40 11:47 |
| media-r6.md (rc 11, 8) | sonnet med, claude/media | tour v8 baseline, started 11:5x |
| tiling-look-r4.md (rc 1) | sonnet med, claude/tiling-look | merged train 39 08:56 |
| xp-polish-r7.md (rc 1) | sonnet med, claude/xp-polish | merged train 39 08:56 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: media r6 tour v8 baseline; known issues, CI PR, tag (rc 4, 5, 8). construct r4, xp r8, mc r7 cancelled.
2. After the tag (foundation F1-F8): Fable plan with priorities-sprint1, rework-audit (F1) -> slices sonnet/opus, haiku tests (F2-F6) -> tour v9 vs v8 (F7), docs (F8).
3. Flakes F17 (F5), cut gate (F6).
4. Sat 03:59: foundation review; next week feature sprint 1.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
