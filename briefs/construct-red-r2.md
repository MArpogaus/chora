# construct-red r2 (F7, F3): close review r1 of claude/construct-red

Model: sonnet, effort high (the fix is named by the review; one focus rule and a cell)
Lane: cloud, branch claude/construct-red at 28254eb6 (r1). One round, stop at 250 k. Cost about 1.5 %.
Step 0: `command -v cargo python3 jq`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "construct-red BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/construct-red.md "## r1" and "## Review r1" (findings rc-1..rc-3).
1. rc-1 MAJOR (plugins/chora-construct lifecycle.rs:37): a new window that is a child of the pressed window (the xdg parent carried in
   surface.created; a dialog, or Ctrl+N of the same app) gets the keys through kit::wm::focus::give, as in mc, tiling and xp; any other
   new pane keeps the r1 rule (keys back to the walk). Red test commit first (cargo cell with FakeCore: dialog of the pressed window gets
   the keys; an unrelated new window does not), then the fix.
2. rc-2 MINOR: the record: renumber D107 to D116 (docs-f8c holds D107-D115 and merges first); status "Proposed (PM accepts at merge)";
   name the child-window rule of item 1 and the mc app.open exception.
3. rc-3 NIT: the stale "known red under WM2" docstring in tests/acts/construct.py:86.
Done when: rc-1 red then green; `cargo test -p chora-construct -p chora-kit --features chora-kit/testing` and clippy -D warnings clean;
`tests/acts/run.py --no-selftest construct` passes; `tests/merge-gate.sh --jobs 3` once (start it in the background and poll it in the same turn;
a known flake: rerun that step with --only and name it).
Write reports/construct-red.md "## r2" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/construct-red and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
