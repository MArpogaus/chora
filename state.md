# State

## Now (2026-11-14 04:25 UTC)
budget: Fable 4 %, all 3 % (owner, 11-14 04:51 UTC)
session branch: 8f358fa (merged 11-12 03:08 UTC). Train 4 is merging in wt/train4.
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
| Lane | Where | Branch and commit | State |
|---|---|---|---|
| train 4 | local sonnet agent, wt/train4 | mc 10eb3a7f, tiling f9dabd84, xp a836cbd6 (r5 d102940f unreviewed, not taken), construct d3843e0, swarm5 9644cc05, harness 8ca9802 | running; conflicts in chora-assistant chat |

## Queue
1. Train 4 lands; one full gate; worktrees removed. (rc 1, 3)
3. Flakes: core SIGSEGV pair, cube navigate-click, "xp runs already", construct deaf, chora-voice. (rc 3)
4. CI draft PR, cargo deny, package build and install smoke. (rc 4, 5, 7)
5. Findings ledgers, shared-doors decision, then main-path MAJOR fix lanes: xp, tiling, mc. (rc 1)
6. Code review per area, core and security first; HIGH fixes. (rc 6)
7. construct: MAJOR fixes only at d3843e0. (rc 1)
8. One pixel pass per look, showcase stills, owner approval. (rc 1, 1b)
9. Persona round 4; docs; logging audit. (rc 2, 8, 9)
10. Hardware smoke by the owner; tag. (rc 10)

## Needs owner
- none (11-14 04:51 UTC: owner accepted defaults (a)-(d): construct preview, fake-voice, xp on fallback art, branches stay, no hardware session).
