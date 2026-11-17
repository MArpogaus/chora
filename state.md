# State

## Now (2026-11-17 13:42 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 58 % at 12:09 Tue (owner); tokens.py 13:42: 4.6 % (PM container). Even pace would be 52 %: 6 % over; 42 % for 87 h = 11.5 %/day, no reserve. Opus today: critic only.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 29e2cd77 (train 41 13:13: deny-chrono, rc-docs); 10 greens (30b, 33-41); train 42 red (pull gate JSON, my half fix).
burn-down 10:56: rc today (rc.md 3, 4, 5, 7, 8); week goal after the tag: design/foundation.md F1-F8 by Sat 03:59.

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
| construct-r3.md (rc 1) | sonnet med, claude/construct | merged train 40 11:47 |
| media-r6.md (rc 11, 8) | sonnet med, claude/media | v8 12:47 (reports/media-r6.md); harness claude/media b29fbf0a: train 43; archived |
| tiling-look-r4.md (rc 1) | sonnet med, claude/tiling-look | merged train 39 08:56 |
| xp-polish-r7.md (rc 1) | sonnet med, claude/xp-polish | merged train 39 08:56 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (cost in % of the week; open lines <= 42 % - 5 % reserve)
1. Now (1 %): rc.4 on dev, main (13:41), untagged (owner 14:20); train 43 media harness (b29fbf0a; rc 11).
2. After the tag (F1, 6 %): Fable plan (marks the week's F2 subset, the F3 effect hook)+2 opus reviewers.
3. Slices F2-F4 sonnet, haiku tests F5 (22 %); flakes F17, cut gate F6 (3 %); F7 contracts and invariants (5 %), docs F8 (1 %).
4. Sat 03:59: foundation review; then simulated user test, new tour with checks (plan re-plan 14:24).

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
