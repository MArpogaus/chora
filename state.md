# State

## Now (2026-11-16 04:46 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 38 % at 01:17 Mon (owner); Mon Bkk day 1.0 % local (tokens.py 03:37). 62 % left to reset Sat 03:59 UTC = 12 %/day; cap 10 %/day. Opus only for a complex lane with a reason.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 3c0603e8 (train 21, 04:17 UTC: security a; train 20 8e7effad). Train 22 gate 2 running (66f13d68; gate 1 F26 doors). CI green on 8e7effad and 3c0603e8.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ti-1 not chora; closes with claude/fuzzel-click + fuzzel>=1.11 pin; sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; F16 fix in harness rc (review); F17, F26 open |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; all 10 green on 8e7effad (sh-22 fix); F25 was 3 of 5 red before, so one green is not proof: watch 2 more runs |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; security a (r5) and b merged; area reviews Tue |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| harness rc r2 (rc 1, 3, 4) | MERGE-OK (PM re-review); train 22 gate running (66f13d68; agent.sh conflict resolved) | briefs/harness-rc-r2.md | F16 fixed (waybar killed by early signals); F25 same-frame retry, unproven; F26 diag only |

## Queue (by day; budget 10 % a UTC day)
1. Mon: train 22 (harness rc). (rc 1, 3, 4)
2. Mon after train 22: media r1 (briefs/media-r1.md) on sonnet medium (screencast by Wed for the 24 h window), then ledger r3 (haiku). (rc 11, 1b, 1)
3. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
4. Wed-Thu: flakes F17, F24, F26, CI6; final review rounds. (rc 3, 4, 8)
5. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
