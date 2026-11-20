# flakes-j r1 (F5: no flaky tests): five flakes seen once under --jobs 3

Model: sonnet, effort high (cause-finding in gate scripts and two unit tests; the PM reviews seeded)
Lane: cloud, branch claude/flakes-j from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flakes-j BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md lines FLAKE-DOORS, FLAKE-H MINOR-1, FLAKE-SIG, "F5 hud flake 73b", "FLAKE candidate" (tiling-look ink), F22; reports/flakes-r6.md for the method.
Items, in this order; per item: reproduce under load (tests/under-load.sh or the cell in a loop with --jobs 3), name the cause with evidence, red commit first, then the fix. No retry, no longer timeout, no sleep that hides a cause:
1. FLAKE-DOORS: tests/gate-doors.sh xp helper-door "missing" once (train 78b) plus jq "null has no keys" in the construct part; log the helper-door click count (FLAKE-H MINOR-1).
2. hud 73b: construct-host-hud "a done of another id closed the question" and chat-long-task "timeout 5s: header_capped ... working".
3. FLAKE-SIG: crates/chora images::the_worker_thread_blocks_the_emergency_signal, once in a full `cargo test -p chora`.
4. tiling-look ink-1280x800 tag colour 1 1 1 255 at 12 s under --jobs 3.
5. F22: crates/chora/tests/log_file.rs:89-92 panic test, 1 of 5 under load.
An item not reproduced in 20 loaded runs: report the count and the evidence read, no change, next item.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,hud,chat-keys,tiling-look,gate-lib`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: per item cause or not-reproduced count, red sha, fix sha, then 10 loaded runs of its cell (counts); gate cells pass; reports/flakes-j.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-j and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
