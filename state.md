# State

## Now (2026-11-15 04:24 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: all models 20 % (03:32 owner); Fable reset and ended: ux r2 finishes its gate, then opus only
session branch: 98291088 (train 12, 03:52 UTC: flakes r4 F17/GR1, vkms faux device; train 11 c2d0a116).
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ledger r1: 32 -> 7 open (ti-1 waits for pk1; 6 in ux r1) |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | r3 merged; open F16 (tiling bar), F17 (words alarm) |
| 4 | CI green | draft PR #106 to dev (session branch); CI running |
| 5 | Packaging smoke | packages.yaml runs in PR #106 |
| 6 | Code review per area | core done; plugins r1 HIGH fixed; MEDIUM security of plugins open |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs merged; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | V02 done; audit not started |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| train 13 | /home/user/wt/train13 be205f89 | ux r2, ci e2e r5 | gate 1 F16; rerun 04:24 |
| harness r2 | cloud opus ..NivBG, claude/harness | briefs/harness-r2.md | r1 FIX (B1 hook blocks without the ref; M1 pgrep by name) |
| flakes r5 | cloud opus ..MuhXr, claude/flakes-r5 | briefs/flakes-r5.md | F19, F18, F16, F20 |
| logging r1 | cloud opus ..MMmuw, claude/logging-r1 | briefs/logging-r1.md | rc 9a pulled forward (owner 04:05) |

## Queue
1. Watch PR #106 CI; fix what fails. (rc 4, 5)
2. Ledger verify, then MAJOR fix lanes per look. (rc 1)
3. Flakes r4: F16, F17, GR1 regrab swap, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; plugins review MEDIUM security. (rc 1, 6)
5. Harness train when MERGE-OK; then ci e2e r5, ux r2. (rc 1, 3, 4)
6. Ledger verify after train 13; persona round 4 Mon; screencast rewrite Mon. (rc 1, 2, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- none
