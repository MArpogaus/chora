# State

## Now (2026-11-17 01:02 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 46 % at 16:49 Mon (owner); tokens.py 23:50: estimate 0.5 %. 45->46 in 4.6 h = 0.2 %/h; 54 % for 107 h to Sat 03:59 = 0.5 %/h: room. Fable flakes r6 only (owner 16:53: no more Fable); media r3 started (pace has room).
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 4105f8d1 (train 33 23:27: ci-fix r1-r2); CI 10/10 green 23:50. Next: testwatch.
burn-down 12:5x: 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp r1, multilook; mc-32 BLOCK and 12 MAJOR in review or train queue; co-18 BLOCK open. Likely tag Tue 11-24 (Wed 12:00).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 0 BLOCK in v5; open MAJOR: j1-2/N1, j1-4, j1-5, N2 (+ reviewer) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b 20:23, 33 23:27 green (2); F26 held (doors 2/2), F28 holds (3/3 after rebuild); F17 open |
| 4 | CI green | green on 4105f8d1 23:50 (10/10 jobs) |
| 5 | Packaging smoke | arch, rpm, podman green on 4105f8d1 23:50 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v5 3:48 on 4105f8d1 (01:01); PM saw j1-4, N2; reviewer running; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | 4105f8d1: doors, chat-keys, xp 3/3 each (23:53); idle |
| construct r1, xp-polish r4 (32), ci-fix r1 (33) | sonnet | merged, archived |
| media-r3.md (rc 11, 1) | sonnet med, claude/media | done 01:01: tour v5 3:48; BLOCK co-18, mc-32 fixed; open MAJOR j1-2/N1 xp editor, j1-4 tiling bar, j1-5 Print, N2 construct pane |
| tiling-look-r2.md (rc 1) | sonnet high, claude/tiling-look | merged train 30b 20:23 |
| flakes-r6.md (rc 3) | Fable high, claude/flakes-r6 | merged train 30b 20:23 (review MERGE-OK, 2 MINOR); no r7 |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: v5 review (rc 11), then fix lanes xp (N1), tiling (j1-4, j1-5), construct (N2) (rc 1); backlog r6 Open, tiling rv-3, rv-5.
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none
