# flakes-k r1 (F5: no flaky tests): the four train-82 flakes under load, the doors Escape pattern

Model: sonnet, effort high (cause-finding in unit tests and gate scripts under load; the PM reviews seeded)
Lane: cloud, branch claude/flakes-k from 6192f696 (flakes-j, MERGE-OK, train 84). One round, stop at 250 k. Cost about 1.2 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flakes-k BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md lines FLK-AG1, FLK-IN1, FLK-TR1, FLK-HUD1, FJ-R1, FJ-R2; reports/flakes-j.md (r1 and the PM review: method, the xp chat_close fix 6192f696).
Items, in this order; per item: reproduce under load (tests/under-load.sh or the cell/test in a loop beside a `cargo build` or with --jobs 3), name the cause with evidence, red commit first, then the fix. No retry, no longer timeout, no sleep that hides a cause:
1. FLK-AG1: tests/agent.sh step 22 hello "user is connected from another program": the step 21 client still holds the name.
2. FLK-IN1: crates/chora inject::tests::injected_input_keeps_the_session_idle got [false, true] (10 ms idle timer).
3. FLK-TR1: crates/chora ipc::tests::a_reservation_for_a_keep_nodes_client_ends_with_it, tracing-subscriber "tried to clone Id(57005), but no span exists".
4. FLK-HUD1: tests/gate-hud.sh context-picks-folder "missing" (5 s on figure_left).
5. FJ-R2: tests/doors tiling, mc, construct: chat_close presses Escape before the chat field holds the keyboard; same fix as xp 6192f696 (wait for `leave` on the probe window), red first per look only where it shows.
6. FJ-R1: images.rs churn test 5 s -> 1 s, keep it red on the old scan (show it).
An item not reproduced in 20 loaded runs: report the count and the evidence read, no change, next item.
Must not touch: crates/chora/src/input.rs, plugins/chora-construct/src/camera.rs, cube.rs (running lanes own them).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,doors,hud,e2e`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: per item cause or not-reproduced count, red sha, fix sha, then 10 loaded runs of its test or cell (counts); gate cells pass; reports/flakes-k.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-k and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing those two branches and the send_message report; do not ask again.
