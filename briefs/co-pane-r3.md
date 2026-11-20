# co-pane r3 (RC 2, shared bar "no stray keys into clients"): fix review r2 rw-1 MAJOR of 628bb930

Model: opus, effort high, because rw-1 likely needs a core input change (crates/chora/src/input.rs: a key that is held back sends no modifier change) and core stays lean and safe
Lane: cloud, branch claude/co-pane at 628bb930. Stop at 250 k. Cost about 2.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "co-pane BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-pane.md "## r2" and "## Review r2" (rw-1..rw-3).
Items, in order (the rest only after item 1 is green):
1. rw-1 MAJOR plugins/chora-construct/src/camera.rs:549: the lone-tap grab (Super+Super_L/R) holds back the Super press, so in a cube's use mode the face program never sees Super down and its own Super chords (xp Win+E, Win+R, Win+D) arrive as bare e, r, d. Wanted: the program gets Super and its chords as before r2; only the Super release that follows a construct chord is held back (the release after a grabbed chord must not reach xp as a lone tap). Red first: a test of the key stream the client gets (core input test or a construct test on the forwarded keys, not only the grab map): (a) Super, e, release in use mode reaches the client as Super+e; (b) Super, a construct chord, release leaves no lone Super tap at the client. If tests/persona/act.py can send press and release as separate events, add a gate step too. Fix: the smallest change, in core only if the plugin cannot do it (e.g. a grab flag "swallow the modifier release after a grabbed chord" in input.rs, with a core unit test), or drop the grab and accept the Start menu; say which in docs/cube-modes.md and which Super keys of the program stay. A core change keeps the escape hatch untouched.
2. rw-2 MINOR: the F5 drag sentence (plans/construct-requirements.md:106) in the same words at docs/cube-modes.md:47 and docs/configuration.md:920.
3. rw-3 NIT overlay.rs:310: rewrap the doc comment.
Each fix: its red test commit first, then the fix; reuse existing helpers (F2). Not in scope: anything else.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,wm-pane,pane-keys,walk,keys,xp` plus `bash tests/persona-selftest.sh`. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/co-pane.md "## r3" with per item sha, test, cell.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
