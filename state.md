# State

## Now (2026-11-15 09:43 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: all models 33 % (09:20 owner). Week resets Sat 11-21 10:59 Bangkok = 03:59 UTC. Pace: at most 93 % at the reset = 10 % a UTC day (0.4 %/h); Sun is over: running opus lanes finish, nothing new on opus before Mon 00:00 UTC.
models (rules.md 5; owner 09:27): haiku (low) tests, gates, personas; sonnet (medium) builds; opus only reviews and hard causes; Model line in every brief.
session branch: 2d9f306f (train 16, 09:28 UTC: harness r5; train 15 0e303d60). Train 17 (flakes r5) gate running.
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-10 and header M1 merged (train 15, 0e303d60); open: ti-1 BLOCK (owner 09:27: setup.sh builds fuzzel 1.11, lane Mon); mc-31 verify (ux r5) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | PR #106: 61d885f0 all green; e2e red on 0e303d60 and 2d9f306f, same pixel (F25, no flake): lane ci f25 |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; 13 MEDIUM security: lanes security a, b r1 |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14); flakes r5 in train 17 |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| train 17 | /home/user/wt/train17 5aa30b54 | flakes r5 (MERGE-OK) | gate 1 running |
| ux r5 | cloud opus ..LwMj9, claude/ux-r5 | briefs/ux-r5.md | mc-31 verify, sh-19, sh-20 |
| security a r1 | 443a1570, review opus medium (local) | briefs/security-a-r1.md | seven fixed, gate green; review running |
| ci f25 r1 | cloud sonnet, claude/ci-f25 | briefs/ci-f25-r1.md | agent.sh step 6 red twice |
| security b r1 | dde7bc90, review opus medium (local) | briefs/security-b-r1.md | six fixed, gate green; review running |

## Queue (by day; budget 10 % a UTC day)
1. Sun: trains 17, CI on its head (F25); reviews of ux r5 and security a, b on opus medium; fix rounds of their findings on sonnet. (rc 1, 3, 4, 6)
2. Mon: fuzzel 1.11 in setup.sh (sonnet low; owner 09:27) then ti-1 check; ledger r3 (haiku cells, opus verdicts). (rc 1)
3. Mon-Tue: media lane on sonnet after ledger r3 is clean: status pictures, screencast, README shots (owner 09:20, 09:27). (rc 11)
4. Tue: persona round 4 as a haiku swarm, triage.py, opus ranks; fixes on sonnet. (rc 2)
5. Wed-Thu: flakes (F16, F25, F24) on haiku series + sonnet fixes; CI6; final review rounds (two clean, opus). (rc 3, 4)
6. Fri 11-20: three greens, tag, CI PR; persona round 5 after the tag. (rc 3)

## Needs owner
- Nothing open.
