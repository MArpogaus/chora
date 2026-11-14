# State

## Now (2026-11-14 20:17 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (friction 44, 45 closed; heartbeat ok 18:03)
budget: Fable 71 % at 16:44, near its limit now (workers report overage); use it up, max 2 lanes; then opus
session branch: 498d9dea (train 4). rc target: 11-21 (criterion 11 adds a day).
train 5 gate running (flakes, ci-rc, docs, xtc, core-r2, mc, changelog revert); burn-down check 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | ~35 MAJOR; doors r3 (shared bar) running; mc MC-1 fixed; ti-1, co-1 to verify |
| 1b | Owner sees looks in the 11 screencast | with 11 |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | flakes: F14, F13, flick fixed; 3 in a row on claude/flakes; new: xp cascade race, construct-layout 2560 |
| 4 | CI green | ci-rc in train 5; CI PR not opened |
| 5 | Packaging smoke | GitHub Actions jobs in the CI PR |
| 6 | Code review per area | core r1-r3 done (in train 5); plugins r1: 3 HIGH open |
| 7 | deny, machete | clean (ci-rc) |
| 8 | Docs, CHANGELOG, known issues | docs in train 5; changelog by cz at the tag |
| 9 | Logging audit, flakes closed | V02 done; audit not started |
| 11 | Visual acceptance screencast | tour tooling claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| train 5 | local gate, wt/train5 | rules.md 8 | full gate running since 20:16 |
| flakes | cloud Fable (starting), claude/flakes 55c527d4 | briefs/flakes-r3.md | r2 in train 5 (R1 open, exception); r3: R1, F15, races |
| doors | cloud Fable ..PZrjj, claude/doors dd8669a4 | briefs/doors-r3.md | dr2-1 BLOCK, dr2-2 MAJOR |
| mc | claude/mc-fix 39ceb8cb | briefs/mc-r5.md | MERGE-OK; in train 5 |
| core | claude/core-r2 308d22dc | briefs/core-r3.md | MERGE-OK; in train 5 |
| xtc, ci, docs | claude/xtc-fix, ci-rc, docs | briefs | MERGE-OK; in train 5 |

## Queue
1. Train 5 green and pushed; status page. (rc 1, 3, 4, 6, 7, 8)
2. Fix lane (opus): plugins HIGH XP-B-01, MC-01, NO-01; verify ti-1, co-1. (rc 1, 6)
3. Flakes r3: xp cascade race, construct-layout 2560, F15 lost first key, F7, A5, A17. (rc 3)
4. CI PR with packaging jobs; merge-gate --locked; fuzzel 1.11 in setup. (rc 4, 5, 7)
5. MAJOR fix lanes per look from findings/; pixel pass per look. (rc 1)
6. Persona round 4; logging audit; screencast rewrite (Mon). (rc 2, 9, 11)
7. Final review rounds (two clean), three greens, tag. (rc 3)

## Needs owner
- none
