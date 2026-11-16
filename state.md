# State

## Now (2026-11-16 10:06 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 43 % at 10:05 Mon (owner). forecast (tokens.py 10:05): 9.7 %/day, ~89 % at reset; room 12 %/day. 41->43 in 48 min (burst; PM opus context is most of the local cost): fewer PM turns, compact at 200 k, sonnet reviews.
models: haiku low tests; sonnet medium builds; opus reviews, hard causes, complex lanes (reason). Cap 12.5 %/day from train 24.
session branch: caf9c80d (train 24, 09:2x: f25 damage scissor; all steps passed). CI green on PR #106 09:41.
burn-down (10:1x, journey 1): open 2 BLOCK (mc-32, co-18) + 19 MAJOR (5 old + 14 tour) ; lanes 6 build, 4.6 d left. Tag Fri at risk: check Wed 12:00, if a BLOCK or >5 MAJOR open, tag Tue 11-24.

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
| construct-r1.md (rc 1) | sonnet high, claude/construct | multilook r1 MERGE-OK (train 27); co-18 BLOCK, co-19, co-20, sh-24 10:5x |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | started 08:55 |
| secfix-a-r2.md (rc 6) | sonnet med, claude/secfix-a | r1 FIX (FIFO open, TOCTOU); r2 10:0x |
| (train 25) secfix-b r1 | merged pending gate | MERGE-OK 10:19 |
| xp-polish-r2.md (rc 1) | sonnet med, claude/xp-polish | r1 MERGE-OK (train 26); r2 xp-25..27 10:4x |
| tiling-look-r1.md (rc 1) | sonnet high, claude/tiling-look | ti-21 waybar, wobbly |
| mc-r1.md (rc 1, 11) | sonnet high, claude/mc | mc-32 BLOCK, mc-33..37; 10:1x |

## Queue (by day; budget 10 % a UTC day)
1. Mon: multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 running (co-18..20, sh-24), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, F28, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none (setup fix applied 09:0x, owner OK)
