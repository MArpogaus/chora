# State

## Now (2026-11-14 16:48 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN, took over 16:30 (old PM ..QmmQ6 dead 10:15-16:30, friction 44; archived)
budget: Fable 71 %, all 37 % (16:44); Fable used up this week, max 2 lanes (16:54); non-Fable use low double digits
session branch: 498d9dea (train 4). rc target: 11-21 (criterion 11 adds a day).

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | open: ~35 MAJOR; BLOCK ti-1 (Apps menu clicks), co-1 (panes lost after mc drag) to verify |
| 1b | Owner sees looks in the 11 screencast | not started |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | no: flakes; F13 fix lost (unpushed) |
| 4 | CI green | ci-rc MERGE-OK (10:09 review); CI PR not opened |
| 5 | Packaging smoke | via GitHub Actions jobs (16:47) |
| 6 | Code review per area | core r1 8/12 fixed; plugins r1: 433 findings, 3 HIGH |
| 7 | deny, machete | clean on ci-rc |
| 8 | Docs, CHANGELOG, known issues | docs MERGE-OK, README keys wrong (Super+space) |
| 9 | Logging audit, flakes closed | not started |
| 11 | Visual acceptance screencast | tour tooling on claude/tour 48fc05b5; rewrite from Mon |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| review core | claude/review-core eff84702 | briefs/review-core-r1.md | MERGE-OK 18:01 (CR-1..7 MINOR); merges after F14 |
| mc | cloud opus ..PoeDb, claude/mc-fix 39ceb8cb | briefs/mc-r5.md | r5 done 19:30 (one rule, 3 green runs); in review |
| xtc | claude/xtc-fix 91fb7d96 | briefs/xtc-r1.md | MERGE-OK 17:16; merges after F14 |
| doors | cloud Fable ..PNz8e, claude/doors ea47e27b | briefs/doors-r2.md | r1 FIX (dr-1, dr-2 BLOCK) |
| ci | claude/ci-rc 01fe4a66 | briefs/ci-r1.md | gate 17:14 FAIL at cube (F14); waits for flakes |
| docs | claude/docs b0cfd0b4 (key fix) | briefs/docs-r1.md | waits with ci for F14 |
| review t5 r2 | done ..LjDUA | briefs/review-train5-r2.md | mc FIX, core MERGE-OK |
| core r2 | claude/core-r2 78f28b2d | briefs/core-r3.md | r2 FIX: V02 rest MAJOR |
| flakes | cloud Fable ..MXy5q, claude/flakes 22b452ed | briefs/flakes-r2.md | F14 fix pushed 17:45; F13 next |

## Queue
0. Robustness done 18:03: fail-open hooks, worktree guard, heartbeat verified, takeover.sh (friction 44, 45).
1. Train 5: ci-rc, docs (key fix), doors, mc, xtc, review-core; review each (opus), full gate per merge. (rc 1, 4, 6, 7, 8)
2. Flakes r2 cloud (sonnet/opus): redo F13, cube flick under load. (rc 3)
3. Verify BLOCK ti-1, co-1; fix lane (opus): plugins HIGH XP-B-01, MC-01, NO-01; core C05 (gate + grant), C14 (injected input is no activity), V02 (utterances at debug), trace C13; CR-1 (URLs with userinfo in llm.rs logs), CR-6 (keep *_api_key_env). (rc 1, 6)
4. CI PR with packaging jobs. (rc 4, 5)
5. Pixel pass per look, owner approval; persona round 4; logging audit. (rc 1b, 2, 9)
6. Three greens on the candidate; tag. (rc 3)

## Needs owner
- none
