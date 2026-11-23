# flk-hang r1 (F5, rc 3: FLK-HANG)

Model: sonnet, effort low (one test line; cause known)
Lane: cloud, claude/flk-hang at 6b01c976 (session head). Cost at most 0.3 %; stop at 150 k context. One area: crates/chora/src/hang.rs tests only; no product change.
Why: backlog.md FLK-HANG on claude/session-log: train t110e red at crates/chora/src/hang.rs:346 `assertion failed: start.elapsed() >= 9 * S` in a_main_loop_that_blocks_for_5_s_triggers_the_watchdog. The 9 s stop Timer is created before `start = Instant::now()`, so an on-time timer can end the loop just under 9 s after start.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check -p chora`; a failure: send_message the PM "flk-hang BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Confirm the cause with file:line. Red first: one commit that makes the race deterministic without a sleep (e.g. a test-local helper that shows start taken after the timer can exceed its deadline, or the assertion run against a start taken after insertion with the timer due), red on the unfixed test.
2) Fix: take `start` before the sources are inserted (or compare with the timer's deadline). No raised bar, no bare sleep, no ignore.
3) `cargo test -p chora --lib hang` 20 times in a loop: 20/20; paste the count.
Gate: full `tests/merge-gate.sh --jobs 2` (no --only) on claude/flk-hang merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7, detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line; do not rerun to green. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: cause with line, red then green commits, 20/20, the gate green or its foreign reds named.
Report: reports/flk-hang.md "## r1" on claude/session-log (shas, cause, red and green lines, count, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/flk-hang and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
