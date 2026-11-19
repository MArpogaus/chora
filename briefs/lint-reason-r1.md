# lint-reason r1 (F4 margin): every #[allow]/#[expect] has a reason

Model: haiku, effort high (mechanical: add reasons; no code change)
Lane: cloud, branch claude/lint-reason from the train 71 head 6aa0bd37. One round, stop at 250 k. Cost about 0.6 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "lint-reason r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md "## r2" (rank 6, row F4).
Do not touch files that kit-report or docs-f8d change: plugins/chora-{xp,mc,construct}/src/main.rs, crates/chora-kit/src/lib.rs, tools/copies.toml, docs/, CONTRIBUTING.md, plugins/chora-construct/src/lifecycle.rs, crates/chora-kit/src/wm/mod.rs.
1. Count before: `rg -U '#\[(allow|expect)\([^]]*\]' crates plugins services --type rust | grep -vc reason` (multi-line aware; note the number).
2. Each attribute without a reason gets `reason = "<why, true for that line>"`; where the lint fires, `#[expect]` may replace `#[allow]`; where it no longer fires, remove the attribute. No other code change.
3. `cargo clippy --workspace --all-targets -- -D warnings` clean (unfulfilled expect is an error).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` green (poll it in the same turn; never end a turn to wait).
Done when: the step 1 count is 0 outside the excluded files, clippy clean; reports/lint-reason.md "## r1" on claude/session-log with before/after counts, head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/lint-reason and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
