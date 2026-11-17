# State

## Now (2026-11-17 09:18 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 55 % at 06:21 Tue (owner); tokens.py 09:18: 3.2 % (PM container). +2 % since 04:20 = 1 %/h, twice the pace; 45 % for 94 h = 0.48 %/h. No new opus until 12:00.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 7e066694 (train 39 08:56: xp r7, tiling r4); CI green on 7e066694 09:18. Tour v7 running.
burn-down 08:57: 0 BLOCK; v6 MAJOR j1-5, N1 merged (train 39), v7 to show; N-1 retest. Tag Fri 11-20 if v7 clean.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; 7 MAJOR (review-tour-v5) in 4 lanes |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-39 green (8); F26 held, F28 holds; F17 open |
| 4 | CI green | green on 7e066694 09:18 (train 39) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; decision records D78-D95 train 37; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop | v6 review 06:26 FIX, 2 MAJOR (reports/review-tour-v6.md); tour v7 running; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| construct-r3.md (rc 1) | sonnet med, claude/construct | drag-back, DB1, B4; session ...LyoZzQD5qMZD32vkP 07:02 |
| media-r5.md (rc 11, 1) | sonnet med, claude/media | tour v7 on 7e066694; session ...PccvgTkknUeRUwmSt, start check ok 09:07 |
| tiling-look-r4.md (rc 1) | sonnet med, claude/tiling-look | merged train 39 08:56 |
| xp-polish-r7.md (rc 1) | sonnet med, claude/xp-polish | merged train 39 08:56 |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: tour v7 (rc 11, 1); construct r3 (rc 1), then train 40.
2. UX loop (rc 1, 11): v7 review; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F22 (CI 06:00), F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
