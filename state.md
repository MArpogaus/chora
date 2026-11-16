# State

## Now (2026-11-16 12:11 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 45 % at 12:10 Mon (owner); tokens.py 12:11: PM 3.7 %. Measured 1 %/h (38->45, 01:17-12:10) = 24 %/d, over the 12.5 cap: hold ledger r3, media r3 to next reading; few PM turns.
models: haiku low tests; sonnet builds, reviews; opus security, hard causes (reason). Rule 9 12:5x.
session branch: 6a067e39 (train 28, 15:58: mc r1+r2, all passed). xp r3 running (hint order); review construct r1 next, then tiling r2.
burn-down 12:5x: 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp r1, multilook; mc-32 BLOCK and 12 MAJOR in review or train queue; co-18 BLOCK open. Likely tag Tue 11-24 (Wed 12:00).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 2 BLOCK, 19 MAJOR (journey-1); ti-1 not chora |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | green on caf9c80d (PR #106, 09:41); F25 fix holds |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v4 3:18 reviewed (journey-1: FIX); media r3 after fixes; last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| construct-r1.md (rc 1) | sonnet high, claude/construct | r1 handback 13:22 (81ff7b60): co-18 BLOCK, co-19, 20, sh-24; K12 pull-out removed; review after mc r2 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | started 08:55 |
| secfix-a (rc 6) | done | merged train 29 |
| xp-polish-r3.md (rc 1, 3) | sonnet med, claude/xp-polish | r3 handback 15:5x (edd0034: draw-order bug, 20/20 runs); review 16:0x |
| tiling-look-r2.md (rc 1) | sonnet high, claude/tiling-look | r2 handback 13:42 (e1dda18f); train 30 = r1+r2 after review, by 23:4x |
| mc-r2.md (rc 1, 11) | done | merged train 28 (mc-32 BLOCK, mc-34, 35, 37) |

## Queue (by day; budget 12.5 % a UTC day)
1. Tue 00:xx: opus F26 doors cause lane (rc 3), replaces ledger r3. testwatch haiku between merges (rc 3, 4).
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none
