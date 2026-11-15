# State

## Now (2026-11-15 11:24 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: all models 33 % (09:20 owner). Week resets Sat 11-21 10:59 Bangkok = 03:59 UTC. Pace: at most 93 % at the reset = 10 % a UTC day (0.4 %/h); Sun is over: running opus lanes finish, nothing new on opus before Mon 00:00 UTC.
models (rules.md 5; owner 09:27): haiku (low) tests, gates, personas; sonnet (medium) builds; opus only reviews and hard causes; Model line in every brief.
session branch: 1014dddf (train 19, 11:24 UTC: ux r5; train 18 4693fe07). No train running.
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | open: ti-1 BLOCK only (fuzzel 1.11 in setup.sh, lane Mon); sh-19, sh-20 merged (train 19); mc-31 not real |
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
| fuzzel r1 | cloud sonnet low | briefs/fuzzel-r1.md | fuzzel >= 1.11 in setup.sh, ti-1 click |
| security a r2 | 0a3d0040, re-review opus medium (local) | briefs/security-a-r2.md | 2 MAJOR fixed (NodeKey injected); walk keys open |
| core watchdog r1 | cloud sonnet, claude/core-watchdog | briefs/core-watchdog-r1.md | sh-22, F25 |

## Queue (by day; budget 10 % a UTC day)
1. Sun: CI (F25); review of security a r2 and core watchdog on opus medium; fixes on sonnet. (rc 1, 3, 4, 6)
2. Sun-Mon: fuzzel r1 (pulled forward, sonnet low); Mon: ledger r3 (haiku cells, opus verdicts). (rc 1)
3. Mon-Tue: media lane on sonnet after ledger r3 is clean: status pictures, screencast, README shots (owner 09:20, 09:27). (rc 11)
4. Tue: persona round 4 as a haiku swarm, triage.py, opus ranks; fixes on sonnet. (rc 2)
5. Wed-Thu: flakes (F16, F25, F24) on haiku series + sonnet fixes; CI6; final review rounds (two clean, opus). (rc 3, 4)
6. Fri 11-20: three greens, tag, CI PR; persona round 5 after the tag. (rc 3)

## Needs owner
- Nothing open.
