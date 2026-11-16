# State

## Now (2026-11-16 07:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 39 % at 07:00 Mon (owner). forecast (tokens.py 07:58): 6.6 %/day since Sun 09:20, ~71 % at reset Sat 03:59 UTC; room 12.5 %/day; cap 10 %/day. Local count sees only the PM container.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 66f13d68 (train 22, 05:23 UTC: harness rc; train 21 3c0603e8). CI green 8e7effad, 3c0603e8; e2e F25 red 66f13d68.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-23, co-16 MAJOR open (tour v3): multilook r1; ti-1 not chora; Apps click cell in CI e2e, fuzzel>=1.11 pinned (train 22); sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; green 8e7effad, 3c0603e8; F25 red 66f13d68 (same values, one frame): f25 r2 on sonnet high |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; security a (r5) and b merged; area reviews Tue |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour v3 recorded (13:37, media/2026-11-16/tour-v3); merge after review |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| f25 r2 (rc 4) | cloud sonnet high, claude/f25 (66f13d68) | briefs/f25-r2.md | running since 06:30 (7th start, session ...LELQh) |
| media r1 (rc 11, 1b, 1) | handed back 07:56, claude/media 00225164 | briefs/media-r1.md | review (sonnet medium, local); gate: doors mc launcher-chat (F26) |
| multilook r1 (rc 1) | cloud sonnet medium, claude/multilook (66f13d68) | briefs/multilook-r1.md | sh-23, co-16 MAJOR from the tour; ti-20 verify |

## Queue (by day; budget 10 % a UTC day)
1. Mon: media r1 review and train; multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 11, 1b, 1)
2. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
