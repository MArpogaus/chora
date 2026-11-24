# flake-series r9 (F5, rc 3): five full gates in a row on the train132 head, run only

Model: haiku, effort high (tests: run the gate five times and record; no code)
Lane: cloud, branch claude/flake-series-r9 at c3c37c2e (session head after train132). Run only: no product, test or harness change. Stop at 150 k. Cost at most 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`. A failure: send_message the PM "flake-series-r9 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/flake-series.md "## r8" (the form), findings/shared.md sh-38..sh-41 (known flakes).
1) Five full gates, one after the other, never two at once: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate-N.log 2>&1; echo EXIT $? >> /tmp/gate-N.log)' >/dev/null 2>&1 </dev/null &` for N=1..5; poll each in the same turn until its EXIT line. No rerun to green, no --only. After each: `pgrep -fa 'chora --headless'` (must be empty; if not, record and kill), `df -h /`; under 5 G free: `rm -rf ~/.cache/sccache`.
2) Per run: EXIT, count of PASS, each red step with its first FAIL line and the log path; a red step: copy its log to media/2026-11-24/flake-series-r9/run-N-<step>.log (under 2 MB) on claude/session-log.
3) A table: run, exit, red steps, minutes; a line "F5 series: N/5 green on c3c37c2e"; each red step matched to a known id (sh-38..sh-41) or "new".
Never end a turn to wait; poll every command in the same turn.
Done when: five runs recorded with the table and the series line.
Report: reports/flake-series.md "## r9" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines: "flake-series-r9 DONE: N/5" plus each red step.
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
