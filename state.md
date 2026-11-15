# State

## Now (2026-11-15 00:12 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: Fable 71 % at 16:44, near its limit now (workers report overage); use it up, max 2 lanes; then opus
session branch: d349a6da (train 8, 00:12 UTC: doors r1-r4, ci e2e; train 7 1366c716: flakes r3, ci r3; 25/25 green).
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
| ci r4 | claude/ci-r4 a3880296 | briefs/ci-r4.md | MERGE-OK (fontconfig in CI); train 9 |
| pk1 | claude/pk1 93521c25 | briefs/pk1-r1.md | MERGE-OK; train 9 |
| ux r1 | cloud Fable (starting), claude/ux-r1 | briefs/ux-r1.md | mc-30, sh-10, sh-11, sh-7, sh-5, ti-4 |
| ci e2e | cloud sonnet ..P9gSn, claude/ci-e2e3 | briefs/ci-e2e-r3.md | PR #106 e2e: agent.sh xs_are; CI4 |
| flakes | cloud Fable ..P8pMY, claude/flakes-r4 | briefs/flakes-r4.md | F17, F16, GR1 |

## Queue
1. Watch PR #106 CI; fix what fails. (rc 4, 5)
2. Ledger verify, then MAJOR fix lanes per look. (rc 1)
3. Flakes r4: F16, F17, GR1 regrab swap, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; plugins review MEDIUM security. (rc 1, 6)
5. Train 9: ci r4, pk1. (rc 4, 5)
6. Persona round 4; logging audit; screencast rewrite (Mon). (rc 2, 9, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- none
