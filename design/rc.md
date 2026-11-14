# Release candidate: criteria and state

All criteria must hold. Then the PM opens the pull request that CI needs, tags the rc
and publishes it (owner 11-11 10:31 UTC: "You may released when all clean."). The state
column is rewritten with state.md; plans and history are in git.

| # | Criterion | State 2026-11-14 |
|---|---|---|
| 1 | UX loop: no BLOCK or MAJOR finding open in any look; an independent pass confirms. | open: about 35 MAJOR |
| 1b | The owner sees each look in the criterion 11 screencast (xp on fallback art; construct with xp on every face); the PM verdict holds unless he objects within 24 h (11-14 17:17). | not started |
| 2 | Persona round 4 on the candidate: every basic task reached by mouse, keyboard and voice via fake-voice (Erika, Kemal, Lena, Max). | not started |
| 3 | Merge gate green three times in a row on the candidate (no flake); the opt-in gates (battle, starved watchdog, shader hang, mc scale, construct-hud, construct-keys, demo) green once. | no: 5 known flakes |
| 4 | GitHub CI green (check, msrv, e2e, coverage, docs, pre-commit) on a pull request. | never run |
| 5 | Packaging: the package workflow builds the RPM and the Arch package (D14) on the CI pull request and on the tag; installed, a session starts (smoke). | packages.yaml on claude/ci-r2 (MERGE-OK 21:1x); proof in the CI PR |
| 6 | Code review per area (core, kit, xp, tiling, mc, construct, assistant, voice): simplicity and readability first, then extraction and duplication across plugins, correctness, security (charter 11: confined choractl mcp with grants, every run_program confirmed, agent spawns only through sandbox_cmd unless unsandboxed: true), dead code, docs. Every HIGH and every security MEDIUM fixed; other MEDIUM findings go to the known issues with a reason. | not started |
| 7 | cargo-deny and cargo-machete clean; dependency updates applied. | deny red |
| 8 | Docs: README, install, configuration, glossary, nav, CHANGELOG with an rc entry; an honest known-issues list. | not started |
| 9a | Logging audit per component and the `chora-report` script (backlog L1). | not started |
| 9 | Flakes closed or explained; no known window loss; the 30-day stability gate restarts with the rc as the new base. | open: backlog "Flakes and gates" |
| 10 | Not needed for the rc: a real-hardware tty smoke beyond what CI and an owner run give (the owner runs it before 1.0). | waived |
| 11 | Visual acceptance (owner 11-14 17:08): a realistic screencast session on the candidate (demo/tour from claude/tour, rewritten from the persona tasks: real programs, human typing and pointer paths, waits for animations, slips and recoveries, every look, transitions and window drags) with a timeline; a haiku swarm reviews each segment against design/looks and the principles, an opus pass merges the findings (reports/journey-1.md); no BLOCK or MAJOR open. | not started |

Order: land, flakes, CI and packaging, main-path MAJOR fixes, code review per area,
one pixel pass per look, persona round 4 and docs, three greens, visual acceptance (11), tag (state.md Queue).
After the tag: persona round 5 at full breadth (owner 11-11 ~18:00: every look and
persona, voice-only in English and German, a ricer, two screens, a session of several
hours for leaks and stalls, the battle script; its fixes before 1.0), then the reflection
loop, then feature sprints (decisions-now.md). Demos and GIFs come from the criterion 11
recording (owner 11-11 12:22 "impressive demos as features land").
