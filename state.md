# State

## Now (2026-11-16 09:09 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 40 % at 08:49 Mon (owner). forecast (tokens.py 08:49): 7.2 %/day, ~74 % at reset; owner 08:52: use the whole week -> target ~12 %/day (cap 12.5 % from next merge). More lanes in parallel.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 66588880 (train 23, 08:3x UTC: media; all steps incl. doors passed). Train 24 (claude/f25 df101c14) gate running. CI green 66588880 (train 23).
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-23, co-16 (multilook r1), xp-23, xp-24, ti-21 (owner 09:01) MAJOR open; ti-1 not chora, Apps cell in CI e2e, fuzzel>=1.11 pinned (train 22) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; green 8e7effad, 3c0603e8; F25 cause found (f25 r2, df101c1f): damage scissor; review, then train |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | all reviewed 08:59, no HIGH; sec MEDIUM in secfix a, b; other MEDIUM to known issues |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour v3 merged (train 23); to the owner after sh-23, co-16 fixed |

## Lanes (cloud; model effort; brief briefs/<lane>.md)
| Lane | Model, branch | State |
|---|---|---|
| f25-r2.md (rc 4) | done, claude/f25 df101c1f | MERGE-OK; train 24 gate running |
| multilook-r1.md (rc 1) | sonnet med, claude/multilook | running 08:46; sh-23, co-16; ti-20 |
| testwatch-r1.md (rc 3, 4) | haiku low, session branch | started 08:55 |
| secfix-a-r1.md (rc 6) | sonnet med, claude/secfix-a | raa-23 fix; raa-3 words |
| secfix-b-r1.md (rc 6) | sonnet med, claude/secfix-b | rab-60, rab-61 |
| xp-polish-r1.md (rc 1) | sonnet med, claude/xp-polish | xp-23 paperclip, xp-24 fields |
| tiling-look-r1.md (rc 1) | sonnet high, claude/tiling-look | ti-21 waybar, wobbly |
| media-r2.md (rc 11) | sonnet med, claude/media | tour v4 by scenario, ~5 min |

## Queue (by day; budget 10 % a UTC day)
1. Mon: train 24 (f25), CI on PR #106; multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. UX loop (owner 09:08): tour v4 -> PM views every chapter + 1 reviewer -> fix lanes -> re-record until no MAJOR; persona r4 (haiku) Tue. (rc 1, 2, 11)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3) Then sprint 1: C10 config parity, dedup, cleanup.

## Needs owner
- none (setup fix applied 09:0x, owner OK)
