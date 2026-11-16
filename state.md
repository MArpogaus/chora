# State

## Now (2026-11-16 07:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 38 % at 01:17 Mon (owner). Pace from owner readings: 33 % Sun 09:20 -> 38 % Mon 01:17 = 7.5 %/day; forecast Sat 03:59 ~76 % at this pace. tokens.py 06:57 counts only this PM container (1.6 %), not cloud lanes. Cap 10 %/day.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 66f13d68 (train 22, 05:23 UTC: harness rc; train 21 3c0603e8). CI green 8e7effad, 3c0603e8; e2e F25 red 66f13d68.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ti-1 not chora; Apps click cell in CI e2e, fuzzel>=1.11 pinned (train 22); sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; green 8e7effad, 3c0603e8; F25 red 66f13d68 (same values, one frame): f25 r2 on sonnet high |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; security a (r5) and b merged; area reviews Tue |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| f25 r2 (rc 4) | cloud sonnet high, claude/f25 (66f13d68) | briefs/f25-r2.md | running since 06:30 (7th start, session ...LELQh) |
| media r1 (rc 11, 1b, 1) | cloud sonnet medium, claude/media (66f13d68) | briefs/media-r1.md | running since 05:37 (session ...QEesX); tour ch1, ch4 recorded |

## Queue (by day; budget 10 % a UTC day)
1. Mon-Wed: media r1 running (screencast by Wed for the 24 h window), then ledger r3 (haiku). (rc 11, 1b, 1)
2. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
