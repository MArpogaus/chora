# State

## Now (2026-11-14 05:19 UTC)
budget: Fable 4 %, all 3 % (owner, 11-14 04:51 UTC)
session branch: 498d9dea (train 4, 11-14 05:18 UTC: mc, construct, tiling, xp r4, swarm5, harness; full gate green on run 2, run 1 F10).
rc target: 11-20 (was 11-18; recomputed 11-14 after W2-1). Next check: 11-16.

## RC (design/rc.md)
| # | Criterion | State |
|---|---|---|
| 1 | UX: no BLOCK or MAJOR open | open: about 35 MAJOR (reports/swarm-6.md, ux-deep-*.md, ux-cross-look.md) |
| 1b | Owner approves each look | not asked |
| 2 | Persona round 4 | not started |
| 3 | Three greens in a row | no: 5 known flakes |
| 4 | CI green | never run; cargo deny red |
| 5 | Packaging smoke | never run |
| 6 | Code review per area | not started |
| 7 | deny, machete | deny red |
| 8 | Docs, CHANGELOG, known issues | not started |
| 9 | Logging audit, flakes closed | not started |

## Lanes
| Lane | Where | Brief | State |
|---|---|---|---|
| flakes | local Fable, wt/flakes, claude/flakes | briefs/flakes-r1.md | queue 1; running |
| ci | cloud sonnet, claude/ci-rc | briefs/ci-r1.md | queue 2; running |
| doors | cloud Fable, claude/doors | briefs/doors-r1.md | queue 3; running |
| mc | cloud Fable, claude/mc-fix | briefs/mc-r1.md | queue 3; starting |
| xtc | cloud Fable, claude/xtc-fix | briefs/xtc-r1.md | queue 3; starting |
| review core | cloud Fable, claude/review-core | briefs/review-core-r1.md | queue 4; starting |
| review plugins | cloud Fable, read-only | briefs/review-plugins-r1.md | queue 4; starting |
| docs | cloud sonnet, claude/docs | briefs/docs-r1.md | queue 7; starting |

## Queue
1. Flakes: core SIGSEGV pair, cube navigate-click, "xp runs already", construct deaf, chora-voice. (rc 3)
2. CI draft PR, cargo deny, package build and install smoke. (rc 4, 5, 7)
3. Shared doors and MAJOR fix lanes from findings/: doors, mc, xtc (xp, tiling, construct). (rc 1)
4. Code review per area, core and security first; HIGH fixes. (rc 6)
5. Merge each lane as it passes review; full gate per merge. (rc 1, 3)
6. One pixel pass per look, showcase stills, owner approval. (rc 1, 1b)
7. Persona round 4; docs; logging audit. (rc 2, 8, 9)
8. Three greens on the candidate; tag. (rc 3)

## Needs owner
- none (11-14 04:51 UTC: owner accepted defaults (a)-(d): construct preview, fake-voice, xp on fallback art, branches stay, no hardware session).
