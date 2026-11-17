# State

## Now (2026-11-17 01:49 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 52 % at 01:46 Tue (owner; Fable 7 %); tokens.py 01:46: 0.7 %. Rise was the Fable lane (owner 01:49: relax, opus ok); 48 % for 98 h = 0.49 %/h.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 4105f8d1 (train 33 23:27: ci-fix r1-r2); CI 10/10 green 23:50. Next: testwatch.
burn-down 12:5x: 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp r1, multilook; mc-32 BLOCK and 12 MAJOR in review or train queue; co-18 BLOCK open. Likely tag Tue 11-24 (Wed 12:00).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; 7 MAJOR (review-tour-v5) in 4 lanes |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b 20:23, 33 23:27 green (2); F26 held (doors 2/2), F28 holds (3/3 after rebuild); F17 open |
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
| tiling-look-r3.md (rc 1) | sonnet med, claude/tiling-look | j1-4, j1-5, rv5-1 started 01:05; start check ok 01:14 |
| xp-polish-r5.md (rc 1) | sonnet med, claude/xp-polish | j1-2/N1 started 01:05; start check ok 01:14 |
| construct-r2.md (rc 1) | sonnet high, claude/construct | N2, rv5-3 started 01:05; start check ok 01:14 |
| mc-r6.md (rc 1) | sonnet high, claude/mc | rv5-2 started 01:05; start check ok 01:14 |
| testwatch-r1.md (rc 3) | haiku low, session branch | 4105f8d1 3/3 x3 (23:53); idle |
| media r3, flakes r6, ci-fix etc. | | merged or done, archived |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: lanes tiling r3, xp r5, construct r2, mc r6 (rc 1); then reviews, trains, tour v6 (rc 11).
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none
