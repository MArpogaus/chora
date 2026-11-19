# minor-sweep r1 (F5): four small open findings of the reviews, each with its test

Model: sonnet, effort high (routine fixes with red tests)
Lane: cloud, branch claude/minor-sweep from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "minor-sweep r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: backlog.md rows rd-9, rc-6, rm-1, "gate-lib-selftest queued-start"; reports/matrix-cap.md "## mx1-m2 r1 PM review"; reports/docs-f8d.md "## r1".
Each fix: its red test commit first (a test that fails before the fix), then the fix commit.
1. rd-9: an explicit `input.ungrab("move")` emits no `input.ungrabbed` (grabs.rs:75, skip Trigger::Move), as docs/api.md says; a core unit test pins it.
2. rc-6: a core test cell (ipc.rs tests) that `surface.created` carries `parent` as at the first commit and a later set_parent sends no event (docs/api.md, D116). A test only; if the code disagrees with the doc, report it, do not change the doc.
3. tests/gate-lib-selftest.sh queued-start check: accept gate_elapsed <= 1 (SECONDS is whole seconds; m1). Test-only change; no red commit needed.
4. rm-1: restore the harness.judge selftest (a known-red row blocks a second overlap) on a synthetic row, not on a real look row (deleted in f7a69761).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,selftests` green (if `selftests` is not a step name, use the step that runs tests/*-selftest.sh; `tests/merge-gate.sh --list` names them). Poll it in the same turn; never end a turn to wait.
Done when: four items in with red first where named; gate green; reports/minor-sweep.md "## r1" on claude/session-log naming each backlog id closed and its test; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/minor-sweep and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
