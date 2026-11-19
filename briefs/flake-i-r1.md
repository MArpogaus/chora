# flake-i r1 (F5): two open flakes, F22 and F31

Model: sonnet, effort high (routine flake fixes, reproduce then fix)
Lane: cloud, branch claude/flake-i from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-i r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md rows F22 and F31.
Do not touch the cells of the running lane flake-h (layout HUD audit, doors helper, construct pull pane); if F22 or F31 already got fixed on 219d5afe, say so with the commit and skip it.
1. F22: crates/chora/tests/log_file.rs:89-92 panic test fails about 1 in 5 under load (race). Reproduce under load first (e.g. 20 runs with `stress`-like parallel cargo test jobs, or nextest --retries 0 in a loop); then fix the race (wait on the condition, not a sleep). Red first where a test can show it; else name the reproduce count before and after (0 of 30 after).
2. F31: tests/xp.sh cleanup `rm -rf $XDG_RUNTIME_DIR` fails "Directory not empty" because killed clients still write into the dir: wait for the killed pids (bounded, e.g. 5 s) before rm. Same pattern in the other tests/*.sh cleanups: fix them the same way in one helper of the gate lib, not copied.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,xp` green, then the test step twice more. Poll it in the same turn; never end a turn to wait.
Done when: both closed or one named as already closed; gate green 3 of 3 for test; reports/flake-i.md "## r1" on claude/session-log with reproduce counts before and after; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-i and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
