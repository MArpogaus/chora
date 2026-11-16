# State

## Now (2026-11-16 08:42 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 39 % at 07:00 Mon (owner). forecast (tokens.py 07:58): 6.6 %/day since Sun 09:20, ~71 % at reset Sat 03:59 UTC; room 12.5 %/day; cap 10 %/day. Local count sees only the PM container.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 66588880 (train 23, 08:3x UTC: media; all steps incl. doors passed). Train 24 (claude/f25 df101c14) gate running. CI green 8e7effad, 3c0603e8; e2e F25 red 66f13d68.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-23, co-16 MAJOR open (tour v3): multilook r1; ti-1 not chora; Apps click cell in CI e2e, fuzzel>=1.11 pinned (train 22); sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fixed (train 22); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; green 8e7effad, 3c0603e8; F25 cause found (f25 r2, df101c1f): damage scissor; review, then train |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; security a (r5) and b merged; area reviews Tue |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour v3 merged (train 23); to the owner after sh-23, co-16 fixed |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| f25 r2 (rc 4) | handed back 08:15, claude/f25 df101c1f | briefs/f25-r2.md | MERGE-OK (reports/review-f25-r2.md, 2 MINOR); train 24 gate running |
| multilook r1 (rc 1) | cloud sonnet medium, claude/multilook (66f13d68) | briefs/multilook-r1.md | 3rd start 08:15 (PENDING 2x); sh-23, co-16 MAJOR; ti-20 verify |

## Queue (by day; budget 10 % a UTC day)
1. Mon: train 24 (f25), CI on PR #106; multilook r1 (sh-23, co-16); then ledger r3 (haiku). (rc 1, 4, 11)
2. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
3. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
4. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- xp fallback wallpaper (CC-BY-SA) resembles Bliss in the pictures: a glance (licensing is yours). Default: keep.
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
