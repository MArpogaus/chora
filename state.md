# State

## Now (2026-11-14 21:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: Fable 71 % at 16:44, near its limit now (workers report overage); use it up, max 2 lanes; then opus
session branch: 4e3c019e (train 6, 21:32 UTC: ci r2, plugins high; 24/24 green).
rc target: 11-21 (criterion 11). Burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ~35 MAJOR; doors r3 (shared bar) running; mc MC-1 fixed; ti-1, co-1 to verify |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | flakes: F14, F13, flick fixed; 3 in a row on claude/flakes; new: xp cascade race, construct-layout 2560 |
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
| flakes | cloud Fable ..PL9EE, claude/flakes 55c527d4 | briefs/flakes-r3.md | r2 merged (R1 open, exception); r3: R1, F15, races |
| ci r3 | cloud opus ..MJWhG, claude/ci-r3 | briefs/ci-r3.md | PR #106 check, coverage: 9 font tests |
| ci e2e | cloud sonnet (starting), claude/ci-e2e | briefs/ci-e2e-r1.md | PR #106 e2e: windows overlap |
| doors | cloud Fable ..PZrjj, claude/doors dd8669a4 | briefs/doors-r3.md | dr2-1 BLOCK, dr2-2 MAJOR |

## Queue
1. Watch PR #106 CI; fix what fails. (rc 4, 5)
2. doors r3 review and merge. (rc 1)
3. Flakes r3: xp cascade race, construct-layout 2560, F15 lost first key, F7, A5, A17. (rc 3)
4. Verify ti-1 (doors r3), co-1; plugins review MEDIUM security. (rc 1, 6)
5. MAJOR fix lanes per look from findings/; pixel pass per look. (rc 1)
6. Persona round 4; logging audit; screencast rewrite (Mon). (rc 2, 9, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- none
