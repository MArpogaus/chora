# keys-route r2 (RC bar: no stray keys; fix round after reports/keys-route.md "## Review r1")

Model: sonnet, effort high (one core key-routing fix with a red test first and a gate step; the fix is named by the review)
Lane: cloud, claude/keys-route at 541b92bb (r1 head). Cost at most 1.0 %; stop at 250 k context. One area: key routing (crates/chora/src/state.rs choose_manager_of, crates/chora/src/input/place.rs grab_action, crates/chora/src/input.rs, crates/chora/src/ipc.rs tests) and one gate step.
Why (reports/keys-route.md "## Review r1" on claude/session-log, 75eef604):
- rw-1 MAJOR state.rs:1001: the r1 early return reverses M4: with a face or pane window focused and the pointer on a screen manager's claim, that manager's chord (tiling Super+Return) opens its window in the face or pane. Reviewer scratch test: new window to `space` at 892d035c, to `face` at 541b92bb.
- rw-2 MINOR: no gate cell reaches the key-made choice.
- rw-3, rw-4 NIT: report sha 851e8960 not ff617c83; input.rs:2844 doc paragraph on the wrong test.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "keys-route BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first: add the face case to `a_grabbed_key_of_the_user_chooses_its_manager_where_the_person_is` (ipc.rs:2876; fixture face_window_with_a_chord_on_the_screen, a real Super+F1 of the screen manager, a new toplevel goes to the screen manager); commit, it fails at 541b92bb.
2) Fix as the review names it: drop the early return in choose_manager_of (keep the M4 choice); grab_action (input/place.rs:31) never holds back the grabs of the keyboard focus owner, and a grab of the focus owner ends a key-made choice. The r1 tests (the three in input.rs, the xp HT-rx-1 test) stay green. docs/api.md input.key in the same commit if behaviour changes. Escape hatch (press_action) untouched.
3) rw-2: one test-mode gate step (gate-handover-shared.sh or injected-presses): tiling on the screen, xp in a pane, injected Super+Return with the pointer on tiling (window opens in tiling), then Super+e (goes to xp); until_ok with a bound, no bare sleep. Red before step 2 if reachable, else say why.
4) rw-3, rw-4: fix the sha in the report text of r2 and move the doc paragraph.
Gate: full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/keys-route merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Run it in the background and poll in the same turn; never end a turn to wait.
Done when: rw-1 red and fix commits, the gate step, cargo test -p chora, -p chora-xp, -p chora-construct green, the full gate green.
Report: reports/keys-route.md "## r2" on claude/session-log (shas, each rw-N and its pin, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/keys-route and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
