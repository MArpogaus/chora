# State

## Now (2026-11-15 08:12 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: all models 20 % (03:32 owner); Fable reset and ended: ux r2 finishes its gate, then opus only
session branch: 61d885f0 (train 14, 08:11 UTC: logging r2; train 13 be205f89). Train 15 (ux r4) gate running.
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | sh-10 and header M1 fixed on claude/ux-r3 (MERGE-OK 08:05, train 15); open: ti-1 BLOCK (fuzzel 1.11, owner); mc-31 verify |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | PR #106: all 10 jobs green on be205f89 (05:17); keep green per train |
| 5 | Packaging smoke | rpm and arch jobs green on be205f89 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; MEDIUM security of plugins open |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | logging audit merged (train 14, 61d885f0); flakes r5 |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| train 15 | /home/user/wt/train15 0e303d60 | ux r4 (MERGE-OK) | gate 1 running |
| harness r5 | cloud opus ..PE7LZ, claude/harness | briefs/harness-r5.md | r4 FIX (M: allow window with trailing comment hides plugin string calls) |
| flakes r5 | cloud opus ..MuhXr, claude/flakes-r5 | briefs/flakes-r5.md | F19, F18, F16, F20 |

## Queue
1. Keep PR #106 green each train; CI6 (gate-0.8 plugin runtime, not in CI). (rc 4)
2. ux r3: sh-10, ti-1; mc-31 verify; then ledger r3 before the tag. (rc 1)
3. Flakes r4: F16, F17, GR1 regrab swap, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; plugins review MEDIUM security. (rc 1, 6)
5. Harness train when MERGE-OK; lanes merge it. (rc 3, 4)
6. Persona round 4 Mon; screencast rewrite Mon. (rc 2, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- Environment for ti-1 (BLOCK): may setup.sh build fuzzel >= 1.11 under $HOME (meson via pip, tllist, fcft; no sudo)? Default if no answer by 11-16 12:00 UTC: no; ti-1 is checked in CI on Arch (fuzzel 1.11+) instead.
