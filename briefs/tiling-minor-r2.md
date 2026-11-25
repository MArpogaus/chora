# tiling-minor r2 (rc 1 known issues; tiling ti-55, ti-56 fixed t139, no other open BLOCK or MAJOR, rules.md 6): rtm1-1..3, rtd3-1..3

Model: sonnet, effort medium (small test hardening in chora-tiling and its gates; routine)
Lane: cloud, branch claude/tiling-minor-r2 at e39448ad (session head, t139 green). Areas: tests/gate-tiling-reload-notice.sh, plugins/chora-tiling (main.rs tests, the doors name test), crates/chora/src/input.rs (one test or doc comment only). Stop at 250 k. Cost at most 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "tiling-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/tiling-minor.md "## Review r1" (rtm1-1..3) and reports/tiling-doors.md (rtd3-1..3) on claude/session-log, and the vision card (tiling).
Each test hardening: show it red against the unfixed code (temporary revert of the fix it guards, paste the red line, restore), then commit the test.
1) rtm1-1: the reload-notice step waits at most `until_ok no_notice 4`; red: with the dismiss removed it fails.
2) rtm1-2: a unit or doors cell where clients.list errors in `launch_looks` and one notice shows.
3) rtm1-3: a `note_act` unit test under a lock (or say so in the doc comment if no test can reach it; name why).
4) rtd3-1: a test that two real connects of one machine get two different names (or say why the pure test is enough). rtd3-2, rtd3-3 NITs: GAP_LINE rejects mismatched quotes; the test moves below `use super::*;`.
Out of scope: other looks, product behaviour beyond the tests, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: items 1-4 each with a pasted red line or a named reason; full gate EXIT 0 (or foreign reds named).
Report: reports/tiling-minor.md "## r2 (lane tiling-minor-r2)" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tiling-minor-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
