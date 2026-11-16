# State

## Now (2026-11-16 00:07 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: 33 % at 09:20 Sun (owner) + Sun over its share; Mon Bkk day 0.0 % local (tokens.py 00:07). Reset Sat 03:59 UTC. Mon: at most 10 %; opus only for the security a review.
models: haiku low tests, gates, trains; sonnet medium builds (high only with a reason); opus only first reviews and hard causes.
session branch: 1014dddf (train 19, 11:24 UTC: ux r5; train 18 4693fe07). No train running.
rc target: Fri 11-20 (tag). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ti-1 not chora; closes with claude/fuzzel-click + fuzzel>=1.11 pin; sh-19, sh-20 merged (train 19); mc-31 not real |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | PR #106: green 5aa30b54, 4693fe07; F25 red on 0e303d60, 2d9f306f, 1014dddf (same values), lead sh-22 (core watchdog) |
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
| security a r3 (rc 6) | 85476db0; review opus medium (local) | briefs/security-a-r3.md | class gate + 2 new finds |
| core watchdog r2 (rc 4) | done 8313f011, claude/core-watchdog | briefs/core-watchdog-r2.md | max per-thread wait; re-review sonnet medium (local) |

## Queue (by day; budget 10 % a UTC day)
1. Mon 00:05: security a r3 review (opus medium); core watchdog r2 re-review on sonnet medium. (rc 4, 6)
2. Mon: train 20 (security a, core watchdog) on MERGE-OK; harness rc r1 review on sonnet. (rc 1, 3, 4)
4. Mon-Tue: media lane on sonnet medium (screencast by Wed for the 24 h window), then ledger r3 (haiku). (rc 11, 1b, 1)
5. Tue: persona round 4 (haiku swarm), triage.py; rc 6 area reviews on sonnet medium, opus only on HIGH. (rc 2, 6)
6. Wed-Thu: flakes F17, F24, CI6; final review rounds. (rc 3, 4, 8)
7. Fri 11-20: three greens, tag, CI PR. (rc 3)

## Needs owner
- Prompt hook: a fail-open UserPromptSubmit hook in settings.local.json with the main duties (your 09:32)? Default: no; duties stay in the injected state.md and rules.
