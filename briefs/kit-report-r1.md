# kit-report r1 (F2, F4): one report() in the kit instead of three copies

Model: sonnet, effort high (routine extraction to the kit)
Lane: cloud, branch claude/kit-report from the train 71 head 6aa0bd37 (debt-down in). One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-report r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md "## r2" (rank 4, row F2), CONTRIBUTING.md "Where shared code lives".
1. Red first: a unit test per look (xp, mc, construct) that pins the exact printed line of its report today; commit it (it passes; it guards the move).
2. One `report(name, err)` in crates/chora-kit/src (the module that fits, or a new one with its CONTRIBUTING.md line, D115); xp, mc and construct call it; no `fn report` outside the kit.
3. tools/copies.toml: a row "own report()" at max 0 so `tools/cut-check` blocks a new copy; tests/cut-check-selftest.sh still ok.
4. Optional, only if small: kit `testing::connect` for the test allows "the test connects to its fake core"; the `#[allow]` total must not rise.
Gate: `tests/merge-gate.sh --jobs 3` on the head, green (start it in the background and poll it in the same turn; never end a turn to wait).
Done when: no `fn report` outside the kit, printed texts unchanged (the step 1 tests pass), cut-check 0 copies, allow count not above 6aa0bd37; reports/kit-report.md "## r1" on claude/session-log with counts before/after and head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-report and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
