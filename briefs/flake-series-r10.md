# flake-series r10 (F5, rc 3): five full gates in a row on the t153+154 head, run only (re-earns rc 3 after the t141, t142, t148, t153 reds)

Model: haiku, effort high (tests: run the gate five times and record; no code)
Red on head: none (run-only series; t153 mc-scale red = sh-37 is what it measures)
Lane: cloud, branch claude/flake-series-r10 at 4d8bbe36 (t153+154 merge; its PM gate runs at the same time on another machine). Run only: no product, test or harness change. Stop at 150 k. Cost at most 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`. A failure: send_message the PM "flake-series-r10 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/flake-series.md "## r9" (the form), findings/shared.md sh-37..sh-42 (known flakes).
1) Five full gates, one after the other, never two at once: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate-N.log 2>&1; echo EXIT $? >> /tmp/gate-N.log)' >/dev/null 2>&1 </dev/null &` for N=1..5; poll each in the same turn until its EXIT line. No rerun to green, no --only. After each: `pgrep -fa 'chora --headless'` (must be empty; if not, record and kill), `df -h /`; under 5 G free: `rm -rf ~/.cache/sccache`.
2) Per run: EXIT, count of PASS, each red step with its first FAIL line and the log path; a red step: copy its log to media/2026-11-26/flake-series-r10/run-N-<step>.log (under 2 MB) on claude/session-log.
3) A table: run, exit, red steps, minutes; a line "F5 series: N/5 green on 4d8bbe36"; each red step matched to a known id (sh-37..sh-42) or "new".
Never end a turn to wait; poll every command in the same turn.
Done when: five runs recorded with the table and the series line.
Report: reports/flake-series.md "## r10" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines: "flake-series-r10 DONE: N/5" plus each red step.
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
