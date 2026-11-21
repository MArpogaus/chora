# co-keys r2 (rc 2, F7; review r1 FIX: rck-1 MAJOR, rck-2..5 MINOR, rck-6 NIT): the F1 hint names a key that works

Model: sonnet, effort medium (construct hints and docs; one MAJOR with a named fix)
Lane: cloud, claude/co-keys at e965470a. Cost at most 0.6 %; stop at 250 k context.
Why: reports/co-keys.md "## Review r1". rck-1: the Set::Flown hint `F1 All keys` (overlay.rs:598) shows in a face and a pane, where F1 works only with `input.grab-any`, which nothing grants by default: a default session names a key that goes to the program (POLS, co-41).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "co-keys BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first (own commit, 3 of 3 red on e965470a): an act in a default session (no grant) that the face hint names a key that opens the key list when pressed (rck-1, rck-4); a unit test that a rebound or dropped `F1: keys` changes the hint chord (rck-3).
2) Fix KISS: the face and cube hints take their chord from the bindings (`keys_of(System::Keys)`); in a face F1 only when its grab held (`rt.grab` true), else Super+F1 (rck-1, rck-3). Docs and construct.yaml say F1 is grabbed in a pane or face only with `input.grab-any`, else it reaches the program and Super+F1 or the hint opens the list (rck-2). construct.py:543 `wait(1.0)` becomes `s.until` cube mode navigate (rck-5); revoke the grant at the end of `keys_acts` (rck-6). Green 5 of 5.
3) `demo/tour/run.py --check construct` once: 0 red (cb-2 known).
4) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,build,test,clippy,construct-hud,construct-extra,doors,acts`. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: rck-1..6 closed with cell names, red on e965470a and green 5 of 5, tour 0 red, gate green.
Report: reports/co-keys.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-keys and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
