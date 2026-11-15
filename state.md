# State

## Now (2026-11-15 03:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: all models 20 % (03:32 owner); Fable reset and ended: ux r2 finishes its gate, then opus only
session branch: c2d0a116 (train 11, 03:17 UTC: tiling focus fade, ci-check, vkms unbind).
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
| ux r2 | cloud Fable ..NSzTiz (new Fable lane), claude/ux-r1 | briefs/ux-r2.md | r1 FIX (M1 held draft); started 03:03 |
| harness r2 | cloud opus ..NivBG, claude/harness | briefs/harness-r2.md | r1 FIX (B1 hook blocks without the ref; M1 pgrep by name) |
| ci e2e r5 | cloud sonnet ..P9gSn, claude/ci-e2e3 | briefs/ci-e2e-r5.md | r4 FIX (CI skips pass; 0.8-10 restate) |
| flakes | done ce8f7928, claude/flakes-r4 | briefs/flakes-r4.md | F17, GR1 fixed; F16 not reproduced; review running |

## Queue
1. Watch PR #106 CI; fix what fails. (rc 4, 5)
2. Ledger verify, then MAJOR fix lanes per look. (rc 1)
3. Flakes r4: F16, F17, GR1 regrab swap, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; plugins review MEDIUM security. (rc 1, 6)
5. Harness train when MERGE-OK; train 12 (flakes r4, ci-vkms3) gate running 03:20; then ci e2e r5, ux r2. (rc 1, 3, 4)
6. Persona round 4; logging audit; screencast rewrite (Mon). (rc 2, 9, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- none
