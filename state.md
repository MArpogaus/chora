# State

## Now (2026-11-17 06:21 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 55 % at 06:21 Tue (owner); tokens.py 06:21: 2.7 % (PM container). +2 % since 04:20 = 1 %/h, twice the pace; 45 % for 94 h = 0.48 %/h. No new opus until 12:00 but the secfix r2 diff review.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: aa8aa8c0 (train 37 05:55: decisions r5); CI green on f1a353a2 05:22. Next: tour v6.
burn-down 04:11: 0 BLOCK; of 7 MAJOR 3 merged (mc 1, construct 2), xp 1 merged (train 36), tiling 3 to confirm in tour v6. Tag Fri 11-20 if v6 clean.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; 7 MAJOR (review-tour-v5) in 4 lanes |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b, 33-37 green (6); F26 held, F28 holds; F17 open |
| 4 | CI green | green on f1a353a2 05:22 (train 36) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; decision records D78-D95 train 37; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v5 reviewed (FIX, reports/review-tour-v5.md); tour v6 after the 4 lanes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| tiling-look-r3.md (rc 1) | sonnet med, claude/tiling-look | merged train 35 04:11; j1-4, j1-5, rv5-1 checked in tour v6 |
| xp-polish-r6.md (rc 1) | sonnet med, claude/xp-polish | merged train 36 05:03 |
| construct-r2.md (rc 1) | sonnet high, claude/construct | merged train 35 04:11 |
| media-r4.md (rc 11, 1) | sonnet med, claude/media | tour v6 on aa8aa8c0; starting 05:56 |
| secfix-c-r2.md (rc 1, 6) | opus med, claude/secfix-c | r1 review FIX (3 MAJOR) 06:03; r2 starting |
| testwatch-r1.md (rc 3) | haiku low, session branch | f1a353a2 3/3 x3 (05:27); idle |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: tour v6 media r4 (rc 11); secfix c r2, 3 MAJOR of review r1 (rc 1, 6); then construct r3 DB1 with v6 findings.
2. UX loop (rc 1, 11): tour v6 (media r4) after the 4 lanes; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F22 (CI 06:00), F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- F27 (environment, by Thu): lanes restore an 11-15 setup cache with fuzzel 1.9.2; e2e apps-click fails there. Fix: environment menu, Edit, re-save the setup script. Default: trains in the PM container (fuzzel 1.11).
