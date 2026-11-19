# wm5 r2 (F3): close the review of wm5 (reports/wm5.md "## Review r1")

Model: sonnet, effort high (named fixes of a reviewed port; no new design)
Lane: cloud, branch claude/wm5 at d14f5342 (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first, then `git merge origin/claude/autonomous-dev-session-3w-i33pg7`.
1. rw-1 MAJOR, red first (own commit): a cell in tests/test_acts.py runs I12 on tiling's real state
   (state.get during a drag of a floating window under another) and expects `held: true` on the dragged
   window; it fails at the red commit. Fix: add `held: bool` (serde default false) to
   `kit::wm::state::Window` (crates/chora-kit/src/wm/state.rs:34); tiling sets it from `dragged`.
   tests/acts/invariants.py:556 keeps reading `held`. Leave I12's `by`/`for` lines (wm3 r2 owns them).
2. rw-2 MINOR, red first: tiling raises the window in its raise order at the press of a drag (as a
   click does); a cell checks the order after the release (dragged window stays on top).
3. rw-3 MINOR: docs/api.md:51 says code 1 (`SceneError::Invalid`); the cell
   `the_lock_order_stays_above_every_plugin` (crates/chora/src/ipc.rs) asserts code 1.
4. NITs: rw-4 api.md:50 and D89: `lock` is reserved, no node holds it, the lock draws outside the
   scene; rw-5 one parse cell: `"order": "top"` is refused (-32602); rw-6 move the doc comment in
   plugins/chora-mc/src/main.rs:1338 back above its test.
Done when: red cells fail at their red commits, green after; `cargo test -p chora-scene -p chora-kit
--features chora-kit/testing`, `-p chora`, `-p chora-tiling`, `-p chora-mc` green;
`python3 -m pytest tests/test_acts.py -q` green; `cargo build --workspace --locked` then
`tests/merge-gate.sh --only acts,tiling-audit,keys,pane-keys` green (poll in the same turn).
No full gate (the train runs it). Push claude/wm5.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
red shas, each finding closed with its cell) and append "## r2" to reports/wm5.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
