# Release candidate: criteria and state

All criteria must hold. Then the PM opens the pull request that CI needs, tags the rc
and publishes it (owner 11-11 10:31 UTC: "You may released when all clean."). The state
column is rewritten with state.md; plans and history are in git.

| # | Criterion | State 2026-11-14 |
|---|---|---|
| 1 | UX loop: no BLOCK or MAJOR finding open in any look; an independent pass confirms. | open: about 35 MAJOR |
| 1b | The owner approves each look from the showcase captures; xp on fallback art; construct with xp on every face. | not asked |
| 2 | Persona round 4 on the candidate: every basic task reached by mouse, keyboard and voice via fake-voice (Erika, Kemal, Lena, Max). | not started |
| 3 | Merge gate green three times in a row on the candidate (no flake); the opt-in gates (battle, starved watchdog, shader hang, mc scale, construct-hud, construct-keys, demo) green once. | no: 5 known flakes |
| 4 | GitHub CI green (check, msrv, e2e, coverage, docs, pre-commit) on a pull request. | never run |
| 5 | Packaging: the release workflow builds the RPM, the Arch package and the distrobox image; installed, a session starts (smoke). | never run |
| 6 | Code review per area (core, kit, xp, tiling, mc, construct, assistant, voice): simplicity and readability first, then correctness, security, dead code, docs. Every HIGH and every security MEDIUM fixed; other MEDIUM findings go to the known issues with a reason. | not started |
| 7 | cargo-deny and cargo-machete clean; dependency updates applied. | deny red |
| 8 | Docs: README, install, configuration, glossary, nav, CHANGELOG with an rc entry; an honest known-issues list. | not started |
| 9a | Logging audit per component and the `chora-report` script (backlog L1). | not started |
| 9 | Flakes closed or explained; no known window loss; the 30-day stability gate restarts with the rc as the new base. | open: backlog "Flakes and gates" |
| 10 | Not needed for the rc: a real-hardware tty smoke beyond what CI and an owner run give (the owner runs it before 1.0). | waived |

Order: land, flakes, CI and packaging, main-path MAJOR fixes, code review per area,
one pixel pass per look, persona round 4 and docs, three greens, tag (state.md Queue).
After the tag: persona round 5 at full breadth, then the reflection loop, then feature
sprints (decisions-now.md).
