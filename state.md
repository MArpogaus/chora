# State

## Now (2026-11-15 08:50 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: all models 33 % (09:20 owner; 20 % at 03:32, about 2.4 %/h); opus only
session branch: 0e303d60 (train 15, 08:50 UTC: ux r4; train 14 61d885f0 CI green). Train 16 (harness r5) gate running.
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-10 and header M1 merged (train 15, 0e303d60); open: ti-1 BLOCK (fuzzel 1.11, owner); mc-31 verify (ux r5) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | PR #106: 61d885f0 all green; 0e303d60 e2e red (F25 agent.sh step 6 corner pixel, flake candidate; commented); train 16 reruns |
| 5 | Packaging smoke | rpm and arch jobs green on 61d885f0 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; 13 MEDIUM security: lanes security a, b r1 |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14, 61d885f0); flakes r5 |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| train 16 | /home/user/wt/train16 2d9f306f | harness r5 (MERGE-OK) | gate 1 running; then flakes train 17 |
| ux r5 | cloud opus ..LwMj9, claude/ux-r5 | briefs/ux-r5.md | mc-31 verify, sh-19, sh-20 |
| security a r1 | cloud opus ..Pu2AX, claude/security-a | briefs/security-a-r1.md | MC-02..04, CON-A-04, CON-B-10, AS-B-05, NO-03 |
| security b r1 | cloud opus ..MD5Xa, claude/security-b | briefs/security-b-r1.md | VO-A-01, -02, -07, VO-B-01, -02, -06 |
| flakes r5 | claude/flakes-r5 366c1cb3 | briefs/flakes-r5.md | MERGE-OK (review-flakes-r5; sh-22, F24); train 17 after train 16 |

## Queue
1. Keep PR #106 green each train; CI6 (gate-0.8 plugin runtime, not in CI). (rc 4)
2. ux r3: sh-10, ti-1; mc-31 verify; then ledger r3 before the tag. (rc 1)
3. Flakes r4: F16, F17, GR1 regrab swap, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; security a, b r1 (13 MEDIUM security). (rc 1, 6)
5. Harness train when MERGE-OK; lanes merge it. (rc 3, 4)
6. After ux r5 merges and ledger r3 is clean: media lane (owner 09:20: prune and update screenshots, screencast rewrite), then persona round 4 Mon. (rc 11, 2)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- Environment for ti-1 (BLOCK): may setup.sh build fuzzel >= 1.11 under $HOME (meson via pip, tllist, fcft; no sudo)? Default if no answer by 11-16 12:00 UTC: no; ti-1 is checked in CI on Arch (fuzzel 1.11+) instead.
