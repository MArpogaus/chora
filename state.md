# State

## Now (2026-11-16 22:35 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 46 % at 16:49 Mon (owner); tokens.py 21:46: +0.4 % since 19:01. 45->46 in 4.6 h = 0.2 %/h; 54 % for 107 h to Sat 03:59 = 0.5 %/h: room. Fable flakes r6 only (owner 16:53: no more Fable); media r3 after next reading.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: ca2804b4 (train 30b 20:23: tiling r2 + flakes r6). Next: testwatch, CI on ca2804b4.
burn-down 12:5x: 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp r1, multilook; mc-32 BLOCK and 12 MAJOR in review or train queue; co-18 BLOCK open. Likely tag Tue 11-24 (Wed 12:00).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 2 BLOCK, 19 MAJOR (journey-1); ti-1 not chora |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | 30b green 20:23 (1); F26 held (doors 2/2), F28 holds (3/3 after rebuild); F17 open |
| 4 | CI green | ca2804b4 e2e FAIL F31 (xp.sh cleanup race, cases pass); rerun 20:3x |
| 5 | Packaging smoke | arch FAIL on ca2804b4 (tiling hud.yaml not in PKGBUILD); ci-fix r1 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v4 3:18 reviewed (journey-1: FIX); media r3 after fixes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| construct-r1.md (rc 1) | sonnet high, claude/construct | merged train 32 17:49 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | ca2804b4: doors 2/2, tiling-look, chat-keys 3/3 pass (11 fails were stale binaries); idle |
| ci-fix-r1.md (rc 4, 5) | sonnet med, claude/ci-fix | r2 fbcf6055 MERGE-OK (PM review); train 33 running 22:35 |
| xp-polish-r4.md (rc 1, 3) | sonnet med, claude/xp-polish | merged train 32 17:49 |
| tiling-look-r2.md (rc 1) | sonnet high, claude/tiling-look | merged train 30b 20:23 |
| flakes-r6.md (rc 3) | Fable high, claude/flakes-r6 | merged train 30b 20:23 (review MERGE-OK, 2 MINOR); no r7 |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: train 33 ci-fix (rc 4, 5), then CI; backlog r6 Open, tiling rv-3, rv-5.
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none
