# State

## Now (2026-11-16 09:22 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 41 % at 09:17 Mon (owner). forecast (tokens.py 09:17): 8.0 %/day, ~79 % at reset; target ~12 %/day; 40->41 in 28 min with 9 lanes: WIP 6 build lanes, no new lane before the next reading.
models: haiku low tests; sonnet medium builds; opus reviews, hard causes, complex lanes (reason). Cap 12.5 %/day from train 24.
session branch: caf9c80d (train 24, 09:2x: f25 damage scissor; all steps passed). CI on PR #106 pending.
burn-down (daily): MAJOR open 5 (sh-23, co-16, xp-23, xp-24, ti-21), lanes 6 build, 4.6 d left; tag Fri if the MAJORs close by Wed and the Thu round is clean.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | 5 MAJOR open (burn-down line); ti-1 not chora, fuzzel>=1.11 pinned |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | F25 fix merged (train 24, caf9c80d); CI e2e step 6 to watch on PR #106 |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Tour, UX loop (testable, 09:20) | v4 recording (media r2); last round Thu 12:00 |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| multilook-r1.md (rc 1) | sonnet med, claude/multilook | running 08:46; sh-23, co-16; ti-20 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | started 08:55 |
| secfix-a-r1.md (rc 6) | sonnet med, claude/secfix-a | raa-23 fix; raa-3 words |
| secfix-b-r1.md (rc 6) | sonnet med, claude/secfix-b | rab-60, rab-61 |
| xp-polish-r1.md (rc 1) | sonnet med, claude/xp-polish | xp-23 paperclip, xp-24 fields |
| tiling-look-r1.md (rc 1) | sonnet high, claude/tiling-look | ti-21 waybar, wobbly |
| media-r2.md (rc 11) | sonnet med, claude/media | tour v4 by scenario, ~5 min |

## Queue (by day; budget 10 % a UTC day)
1. Mon: train 24 (f25), CI on PR #106; multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. UX loop: tour v4 -> PM views + 1 reviewer -> fixes -> re-record; last round starts Thu 12:00 (owner 09:16); persona r4 Wed. WIP 6 build lanes. (rc 1, 2, 11)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none (setup fix applied 09:0x, owner OK)
