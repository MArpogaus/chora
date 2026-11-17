# State

## Now (2026-11-17 12:50 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 58 % at 12:09 Tue (owner); tokens.py 12:50: 4.5 % (PM container). Even pace would be 52 %: 6 % over; 42 % for 87 h = 11.5 %/day, no reserve. Opus today: critic only.
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
| media-r6.md (rc 11, 8) | sonnet med, claude/media | v8 done 12:47 (reports/media-r6.md, claude/media-v8); harness on claude/media 0818f569: review, train; archived |
| tiling-look-r4.md (rc 1) | sonnet med, claude/tiling-look | merged train 39 08:56 |
| xp-polish-r7.md (rc 1) | sonnet med, claude/xp-polish | merged train 39 08:56 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 42 % - 5 % reserve)
1. Now (2 %): train 41 (deny-chrono, rc-docs; rc 7, 8) e2e; train 42 media harness (ef646e59, review FIX fixed; rc 11); CI PR, packaging smoke on the tag commit, tag (rc 4, 5, 8).
2. After the tag (F1, 6 %): Fable plan (marks the week's F2 subset, the F3 effect hook) + 2 opus reviewers; v8 fresh-eyes review (F7 baseline, 1 %).
3. Slices F2-F4 sonnet, haiku tests F5 (22 %); flakes F17, cut gate F6 (3 %); tour v9 F7 (3 %), docs F8 (1 %).
4. Sat 03:59: foundation review; then reports/plan-after-foundation.md P2-P5 (to 11-30).

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
