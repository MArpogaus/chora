# kitcov r1 (F5): chora-kit at least 80 % lines, unit tests only

Model: haiku, effort high (tests only: unit cells through kit testing, no production change)
Lane: cloud, branch claude/kitcov from the session head dd2a0aa3 (claude/autonomous-dev-session-3w-i33pg7).
One round, stop at 250 k. Cost about 1.5 %.
Step 0: `cargo check -p chora-kit --features chora-kit/testing` and `cargo llvm-cov --version`; either fails: send BLOCKED (no setup) to the PM at once and stop.
1. Measure: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`;
   put the per-file table (lines %) in the report, before any change.
2. For each chora-kit file under 80 % lines, lowest first: unit tests in that file's `#[cfg(test)]` module
   (or tests/ of chora-kit) through kit testing (FakeCore); each test asserts a behaviour named in docs/api.md
   or the item's doc comment, not just that a line runs. No production code change; a test that finds a bug
   stays as `#[ignore = "bug: <one line>"]` and goes in the report as a finding.
3. Measure again; table before and after; total chora-kit lines >= 80 % is the target.
Done when: the after table; `cargo test -p chora-kit --features chora-kit/testing` green;
`cargo clippy -p chora-kit --all-targets --features chora-kit/testing -- -D warnings` clean;
`tests/merge-gate.sh --only build,ci-check,pre-commit` green (poll it in the same turn). No full gate (the train runs it).
Report: append "## r1" to reports/kitcov.md on claude/session-log (tables, tests added, findings), then
send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines). Poll every command in the
same turn; never end a turn to wait.
Rules: push claude/kitcov and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
