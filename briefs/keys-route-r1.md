# keys-route r1 (RC bar: no stray keys; plans/week-3.md row 34)

Model: sonnet, effort high (key routing across core input, xp and construct; four small fixes with red tests)
Lane: cloud, claude/keys-route at 892d035c (session head after train 101). Cost at most 1.6 %; stop at 250 k context. One area: key routing (crates/chora/src/input.rs, state.rs choose_manager_of; plugins/chora-xp/src/wm/keys.rs; plugins/chora-construct/src/camera.rs) and their gate cells.
Why (backlog.md on claude/session-log, each pre-existing, all on the RC bar "no stray keys"):
- HT-rx-1 (backlog.md:350): xp keys.rs:331 focus_moved gives xp-icons the keys while xp's own Start menu is open and a tiling window exits; fix: add !own_node().
- construct rx-1 (backlog.md:351, reports/co-pane.md "Review r3" rx-1): after a person's construct chord (Super+F1) choose_manager_of picks construct and xp's Super tap, Super+e, Super+d are swallowed until a click or focus change. choose_manager_of ignores injected_press_is_real (buttons honour it), so no gate reaches it.
- CO-P2-FIX r2 rw-2 (backlog.md:305): camera.rs:476 in_pane() can hold with a cube open; unbound Super chords are swallowed then (swallow only when cube.is_none()).
- CO-P2-FIX r1 (backlog.md:294): xp opens Start on a construct-swallowed Super chord.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "keys-route BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Each item: red test commit first (unit test, and a gate step where a cell can reach it), then the fix. Order: construct rx-1 first (core: make choose_manager_of honour injected_press_is_real the way buttons do, so a gate can reach it; then the fix of the choice itself), then rw-2, HT-rx-1, CO-P2-FIX r1.
Core rule (KISS, POLS): a key the person presses goes to exactly one client or to none by a named binding; no client keeps a held chord after the focus or the manager changes. The escape hatch (press_action: VT switch, Quit, Reset, RevokeAll) stays first and untouched. docs/api.md input.key changes in the same commit as the code if behaviour changes.
Gate cells to reach: injected-presses, handover-shared, acts, pull (gate-construct-pull.sh). A bare `sleep` you add is refused: use until_ok with a bound.
Gate: full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/keys-route merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: four items each with a red and a green commit, cargo test -p chora, -p chora-xp, -p chora-construct green, the full gate green.
Report: reports/keys-route.md "## r1" on claude/session-log (items, red and fix shas, the cell or test that pins each, gate summary, anything left open with file:line); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/keys-route and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
