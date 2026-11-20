# co-pane r2 (RC 2, shared bar "no stray keys into clients"): fix review r1 findings of f10842d8

Model: sonnet, effort high (the review names file, line and fix for each finding; routine follow-up)
Lane: cloud, branch claude/co-pane at f10842d8. Stop at 250 k. Cost about 2 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "co-pane BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-pane.md "## r1" and "## Review r1" (findings rv-1..rv-8).
Items, in order (rv-1 first; the rest only after rv-1 is green):
1. rv-1 MAJOR plugins/chora-construct/src/camera.rs:542: a grabbed Super chord in use mode of a cube leaves a bare Super tap for the xp program (XP Start menu opens). Red first: a construct unit test that a cube in use mode swallows the lone Super tap after a chord (and, if tests/persona/act.py can send press and release as separate events, a gate step too). Fix: `swallow_taps()` in `regrab` while a cube is in use mode. Note the price (no Start key by a Super tap inside a face) in docs/cube-modes.md.
2. rv-2 MINOR overlay.rs:316: the sheet takes the keyboard also with `keys_with_pane()`.
3. rv-3 MINOR overlay.rs:342: note the window with `host.note_focus` on open, give it back through `take_focus` on close (front-face check).
4. rv-5 MINOR camera.rs:428: `self.regrab()` after `keys_to_the_walk` in `unseen_focus_to_walk`; test a step that takes the pane out of sight.
5. rv-7 MINOR tests/persona/check.py:89: take a child only when `cube.list` names it a face manager; keep the front-face test for `shown`; a negative selftest case.
6. rv-4 MINOR command.rs:224: only the key sheet makes the Close hit.
7. rv-6 MINOR camera.rs:32, PM decision: the cube follows the hand (docs/cube-modes.md:131): invert dy and dx in `turn` (grab the world); add the direction to plans/construct-requirements.md F5. The first drag after a fresh start: reproduce with a test or write it as by design with the reason.
8. rv-8 NIT: drop the `toggle_help` regrab unless a test needs it.
Each fix: its red test commit first, then the fix; reuse existing helpers (F2). Not in scope: anything else in construct.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,wm-pane,pane-keys,walk,keys` plus `bash tests/persona-selftest.sh`. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/co-pane.md "## r2" with per item sha, test, cell.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
