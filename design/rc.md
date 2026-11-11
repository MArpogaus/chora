# Release candidate readiness (plan of 11-11)

User, 11-11 17:10: after the UX/UI loop is clean, finalize the
foundational work and review the code until it is ready for an rc
release; feature sprints only after that.

## Criteria (all must hold)

1. UX loop: no block or major finding open; a second designer pass
   confirms.
2. Persona round 4 on the rc candidate: every basic task reached by
   mouse, keyboard and voice (Erika, Kemal, Lena, Max).
3. Merge gate green three times in a row on the candidate (no flake);
   the opt-in gates (battle, starved watchdog, shader hang, mc scale,
   construct-hud, construct-keys, demo) green once.
4. GitHub CI green (check, msrv, e2e, coverage, docs, pre-commit): needs
   a push to dev/main or a pull request, which the user must allow.
5. Packaging: the release workflow builds the RPM and the Arch package
   and the distrobox image; installed, a session starts (smoke).
6. Whole-piece code review per area (core, kit, xp, tiling, mc,
   construct, assistant, voice): correctness, security, dead code,
   docs; findings HIGH and MEDIUM fixed. Fable on Monday if the week's
   budget is spent; opus before.
7. cargo-deny and cargo-machete clean; dependency updates applied.
8. Docs: README, install, configuration, glossary, nav, CHANGELOG with
   an rc entry; a known-issues list (honest, P5).
9a. Logging audit per component and the `chora-report` script
    (backlog L1), so the post-rc user round yields usable reports.
9. Backlog: flakes F5-F7 closed or explained; no known window loss; the
   30-day stability gate restarts with the rc as the new base.
10. Not needed for rc: real-hardware tty smoke beyond what CI and a
    user run give (the user runs it on his machine before 1.0).

## Order

UX loop fixes -> UX pass 2 -> code review per area (parallel, read
only) -> fix lanes -> persona round 4 -> CI and packaging -> rc tag
(the user tags or approves the tag).

## User decision of 11-11 ~17:20

"You may release when all clean. Review code for simplicity and
readability." So: when every criterion holds, the PM opens the pull
request that CI needs, tags the rc and publishes the release without
asking again. The code review per area judges simplicity and
readability first (KISS: dead code, needless layers, long functions,
unclear names, duplication), then correctness and security.

## User, 11-11 ~18:00

"After the release candidate, we need an extensive user testing round
to really ensure that everything is going smoothly." Plan: after the rc
tag, persona round 5 at full breadth: all four looks, all personas
(low-tech, keyboard-only, voice-only in English and German, ricer),
two screens, every basic task, the cube and the panes, a long session
(hours) for leaks and stalls, plus the opt-in battle script. Findings
are fixed before 1.0; the rc stays the base of the 30-day gate.

## User, 11-11 ~18:20

After the rc gates pass: an intense reflection loop, then sharpen the
charter, the dev rules and the plan for the next days. Plan: the Fable
judge (fresh pool next week) reviews the whole session so far (what
worked, what wasted time or tokens, review findings by kind, merge
conflicts, flakes, worker handoffs, the 5-hour pacing); I add my own
critique; the user's readings and decisions are the yardstick. Output:
reflections/2026-11-rc.md, a charter diff (rules added, rules removed),
dev-rules and brief templates updated, and the plan of the next days.

## After the 11-11 reflection

- CI and packaging move up: a draft pull request (CI) and a release dry
  run start on 11-12, beside the code review, not last.
- Flakes F5-F8 first, or three greens in a row will not hold.
- Scope: fix every HIGH and every security MEDIUM; other MEDIUM findings
  go to the known-issues list with a reason.
- Estimate: rc on Wed 11-18 (range 11-17 to 11-20).
