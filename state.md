# State

## Now (2026-11-14 16:48 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (old PM ..QmmQ6 dead 10:15-16:30, friction 44; archived)
budget: Fable 71 %, all 37 % (16:44); no Fable this week (16:47); non-Fable use low double digits
session branch: 498d9dea (train 4). rc target: 11-20.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | open: ~35 MAJOR; BLOCK ti-1 (Apps menu clicks), co-1 (panes lost after mc drag) to verify |
| 1b | Owner approves each look | not asked |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | no: flakes; F13 fix lost (unpushed) |
| 4 | CI green | ci-rc MERGE-OK (10:09 review); CI PR not opened |
| 5 | Packaging smoke | via GitHub Actions jobs (16:47) |
| 6 | Code review per area | core r1 8/12 fixed; plugins r1: 433 findings, 3 HIGH |
| 7 | deny, machete | clean on ci-rc |
| 8 | Docs, CHANGELOG, known issues | docs MERGE-OK, README keys wrong (Super+space) |
| 9 | Logging audit, flakes closed | not started |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| review core | cloud Fable ..MFd4q, claude/review-core | briefs/review-core-r1.md | told 16:48: finish gate, hand over, stop |
| mc | cloud Fable ..QEL2G, claude/mc-fix 4feb4dc0 | briefs/mc-r1.md | told 16:48: finish gate, report, stop |
| xtc | done ..QVmwt, claude/xtc-fix 91fb7d96 | briefs/xtc-r1.md | to review |
| doors | done ..N9VG6, claude/doors ea47e27b | briefs/doors-r1.md | to review; merges before mc, xtc |
| ci | blocked ..MMUYU, claude/ci-rc 01fe4a66 | briefs/ci-r1.md | MERGE-OK; packaging moves to Actions |
| docs | done ..NQmGA, claude/docs 41d02965 | briefs/docs-r1.md | MERGE-OK after README key fix |
| flakes | lost (local, died 10:35), claude/flakes 22b452ed | briefs/flakes-r1.md | restart as cloud lane, not Fable |

## Queue
0. Robustness: fail-open hooks, worktree guard, heartbeat, takeover.sh (friction 44).
1. Train 5: ci-rc, docs (key fix), doors, mc, xtc, review-core; review each (opus), full gate per merge. (rc 1, 4, 6, 7, 8)
2. Flakes r2 cloud (sonnet/opus): redo F13, cube flick under load. (rc 3)
3. Verify BLOCK ti-1, co-1; plugins HIGH XP-B-01, MC-01, NO-01 fix lane. (rc 1, 6)
4. CI PR with packaging jobs. (rc 4, 5)
5. Pixel pass per look, owner approval; persona round 4; logging audit. (rc 1b, 2, 9)
6. Three greens on the candidate; tag. (rc 3)

## Needs owner
- none
