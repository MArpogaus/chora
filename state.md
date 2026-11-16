# State

## Now (2026-11-16 09:22 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 41 % at 09:17 Mon (owner). forecast (tokens.py 09:17): 8.0 %/day, ~79 % at reset; target ~12 %/day; 40->41 in 28 min with 9 lanes: WIP 6 build lanes, no new lane before the next reading.
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
| multilook-r1.md (rc 1) | sonnet med, claude/multilook | running 08:46; sh-23, co-16; ti-20 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | started 08:55 |
| secfix-a-r2.md (rc 6) | sonnet med, claude/secfix-a | r1 FIX (FIFO open, TOCTOU); r2 10:0x |
| secfix-b-r1.md (rc 6) | sonnet med, claude/secfix-b | rab-60, rab-61 |
| xp-polish-r1.md (rc 1) | sonnet med, claude/xp-polish | xp-23 paperclip, xp-24 fields |
| tiling-look-r1.md (rc 1) | sonnet high, claude/tiling-look | ti-21 waybar, wobbly |
| mc-r1.md (rc 1, 11) | sonnet high, claude/mc | mc-32 BLOCK, mc-33..37; 10:1x |

## Queue (by day; budget 10 % a UTC day)
1. Mon: multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. UX loop (rc 1, 11): journey-1 FIX -> on handback xp-polish r2 (xp-25..27), tiling-look r2 (ti-22..24), construct r1 after multilook (co-18, co-19), media r3 last; persona r4 Wed; last round Thu 12:00.
3. Wed-Thu: flakes F17, F24, F26, F27, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none (setup fix applied 09:0x, owner OK)
