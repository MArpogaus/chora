# State

## Now (2026-11-16 02:23 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 38 % at 01:17 Mon (owner); Mon Bkk day 0.7 % local (tokens.py 01:17). 62 % left to reset Sat 03:59 UTC = 12 %/day; cap 10 %/day. Opus only for a complex lane with a reason.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 8e7effad (train 20, 01:16 UTC: core watchdog; train 19 1014dddf). CI 10/10 green on 8e7effad.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ti-1 not chora; closes with claude/fuzzel-click + fuzzel>=1.11 pin; sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; all 10 green on 8e7effad (sh-22 fix); F25 was 3 of 5 red before, so one green is not proof: watch 2 more runs |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; 13 MEDIUM security: lanes security a, b r1 |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 merged (train 17) |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| harness rc r1 (rc 1, 3, 4) | cloud sonnet medium, claude/harness-rc | briefs/harness-rc-r1.md | F25, F16/F26 test-side; fuzzel cell + rpm pin |
| security a r4 (rc 6) | done 02:22, 250b3716; review on sonnet medium running (local) | briefs/security-a-r4.md | flag in the event type; hud cell now refuses injected row click |

## Queue (by day; budget 10 % a UTC day)
1. Mon: security a r4 and harness rc r1 reviews on sonnet medium, trains on MERGE-OK. (rc 1, 3, 4, 6)
2. Mon-Tue: media lane on sonnet medium (screencast by Wed for the 24 h window), then ledger r3 (haiku). (rc 11, 1b, 1)
3. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
4. Wed-Thu: flakes F17, F24, CI6; final review rounds. (rc 3, 4, 8)
5. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
