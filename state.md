# State

## Now (2026-11-16 19:01 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 46 % at 16:49 Mon (owner); tokens.py 19:01: PM +0.2 % since 16:50. 45->46 in 4.6 h = 0.2 %/h; 54 % for 107 h to Sat 03:59 = 0.5 %/h: room. Fable flakes r6 only (owner 16:53: no more Fable); media r3 after next reading.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: aba97fec (train 32: construct r1 + xp r2-r4, 17:49; 32a F26 at doors). Next: tiling r2 review, train 30.
burn-down 12:5x: 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp r1, multilook; mc-32 BLOCK and 12 MAJOR in review or train queue; co-18 BLOCK open. Likely tag Tue 11-24 (Wed 12:00).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 2 BLOCK, 19 MAJOR (journey-1); ti-1 not chora |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | F26 mc, F28 fixed in flakes r6 (review); F17 propose close |
| 4 | CI green | green on aba97fec (PR #106, 18:09); F25 fix holds |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v4 3:18 reviewed (journey-1: FIX); media r3 after fixes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| construct-r1.md (rc 1) | sonnet high, claude/construct | merged train 32 17:49 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | r1 17:07 + aba97fec 18:12 (doors, chat-keys pass; F27 env); idle |
| secfix-a (rc 6) | done | merged train 29 |
| xp-polish-r4.md (rc 1, 3) | sonnet med, claude/xp-polish | merged train 32 17:49 |
| tiling-look-r2.md (rc 1) | sonnet high, claude/tiling-look | MERGE-OK 18:1x (2 MINOR); train 30 F26 at doors 19:00; 30b = + flakes r6 after review |
| flakes-r6.md (rc 3) | Fable high, claude/flakes-r6 | ended 18:22 (limit), e3fb7d46: F26 mc, F28 fixed; F17 close; review after train 30; no opus r7 |

## Queue (by day; budget 12.5 % a UTC day)
1. Now: review flakes r6 (sonnet), then train 30b = tiling + flakes r6 (rc 1, 3). testwatch after.
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none
