# review construct-red r2 (F7, F3): seeded review of claude/construct-red before its merge

Model: opus, effort high, because r2 changes the core event API (surface.created gains `parent`: chora-scene event.rs, crates/chora/src/handlers.rs) and the construct focus hand-off (D31, kit::wm::focus)
Lane: cloud, read-only on code: claude/construct-red head da68737d, base 28254eb6 (11 files, +134 -17, 2 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on da68737d).
Read briefs/construct-red-r2.md (done criteria), reports/construct-red.md "## Review r1" and "## r2" on claude/session-log; `git diff 28254eb6..da68737d`.
Checks, each with its command and files:
1) Red first: 6c065581 fails on the dialog assert and da68737d passes (`cargo test -p chora-construct --features chora-kit/testing`).
2) rc-1 (plugins/chora-construct/src/lifecycle.rs keys_for_new): a child of the pressed window gets kit::wm::focus::give; any other new pane keeps the walk. POLS: nested child (dialog of a dialog), a child that maps after the press moved, a child of a window on another face, a transient popup (xdg_popup is not a toplevel: confirm it never reaches this path). Any path where no one holds the keys?
3) Core API: `parent` in Event::SurfaceCreated (chora-scene event.rs) and handlers.rs announce from toplevel.parent(). Is it additive on the wire (serde default, skip when None), documented in docs/api.md, and correct when the parent is set after the first commit (xdg set_parent later)? Name the gap, if any, as a finding; a later set_parent is MINOR unless a look depends on it.
4) The other looks: xp startup.rs and kit wm/book.rs changed; do mc, tiling, xp compile and behave the same (`cargo test -p chora-kit --features testing`, `cargo test -p chora-xp`)?
5) rc-2: D116 in docs/decisions.md is "Proposed (PM accepts at merge)", names the child rule, the mc app.open exception and the new field; rc-3 docstring gone (tests/acts/looks/construct.py).
6) Known: D-number clashes are the PM's at merge, not findings.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rc-N from rc-4, severity,
file:line, text, fix in one sentence). Append to reports/construct-red.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-construct-red BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
