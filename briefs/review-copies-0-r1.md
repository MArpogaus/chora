# review copies-0 r1 (F2): seeded review of claude/copies-0 before its merge

Model: sonnet, effort high (port review: copies moved onto chora-kit helpers, no core change)
Lane: cloud, read-only on code: claude/copies-0 head ee53e09c, base 85764974 (28 files, +365 -337). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 41/41).
Read briefs/copies-0-r1.md (done criteria) and reports/copies-0.md "## r1" on claude/session-log; `git diff 85764974..ee53e09c`.
Checks, each with its command and files:
1) No behaviour change: each replaced copy (UnixListener in plugin tests, confirm.answer, open entry in the four state.rs, mc surface filter,
   on_path) does what the old code did; read old against new per call site. mc acts.rs `to != NAME` removed as dead: confirm
   Conn::claimant_at excludes self in every path (crates/chora-kit) and the pinning test asserts it.
2) Kit helpers small and tested: FakeCore respond/record/hold and Surface::entry in crates/chora-kit/src/testing.rs and wm/open.rs, each with a
   test in crates/chora-kit/tests/fake_core.rs; `cargo test -p chora-kit --features testing`.
3) Plugin tests still assert what their names claim after the FakeCore switch (construct cube/tests_moved.rs, tiling overlay/tests_moved.rs, xp wm.rs, notify,
   assistant); no assert lost; `cargo test -p chora-construct -p chora-tiling -p chora-xp -p chora-notify -p chora-assistant`.
4) ee53e09c: the allow in plugins/chora-xp/src/wm.rs (about line 95) was added to pass the ci-check plugin-runtime grep: is it a real exception
   with a reason, or a workaround that hides a disallowed method in production code? Read tests/ci-check (the grep) and the attribute.
5) tools/copies.toml: `tools/cut-check` prints no copies REPORT; every new allow row has a one-line reason that holds.
Known, not findings: debt-down (running) also edits xp wm.rs; a merge conflict there is the train's, not this branch's.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rc-N, severity,
file:line, text, fix in one sentence). Append to reports/copies-0.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-copies-0 BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
