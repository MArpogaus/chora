# review construct-red r1 (F7, F3): seeded review of claude/construct-red before its merge

Model: opus, effort high, because the diff changes the focus hand-off design (D31, kit::wm::focus) and an acts invariant (I13) that guards the cube
Lane: cloud, read-only on code: claude/construct-red head 28254eb6, base ee53e09c (7 files, +137 -35, 5 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on 28254eb6).
Read briefs/construct-red-r1.md (done criteria) and reports/construct-red.md "## r1" on claude/session-log; `git diff ee53e09c..28254eb6`.
Checks, each with its command and files:
1) Red first: a9c870b6 fails on its parent and passes on e46862fc; df070c01 fails on its parent and passes on d8f80eaa (`cargo test -p chora-construct`, `python3 -m pytest tests/test_acts.py`).
2) kit7-a2 (plugins/chora-construct/src/lifecycle.rs, modes.rs): a new pane without a flight hands the keys back to the walk via kit::wm::focus::release. POLS: does the user who presses a window and then opens a pane expect the keys at the walk? Do mc, tiling and xp really give new windows the keys (name file:line)? Any path where no one holds the keys?
3) co-21 changes I13 (tests/acts/invariants.py) instead of the construct. Is this a real wrong invariant (Host::faces hides back faces, so a box for every neighbour cannot hold mid-turn) or a weakened check? Does the new rule ("a face shown alone in a held turn" is flagged) still catch the bug I13 was written for (git log -S I13 tests/acts/invariants.py)?
4) known-red.toml: each removed row has a passing cell named in the report; only net3-m1 left.
5) wm7a-m3: the new FakeCore cell in cube/tests_moved.rs fails with tell_end removed (revert that line locally, run, restore).
6) docs/decisions.md: the lane wrote D107, but D107-D115 are taken by docs-f8c (train 68). A number clash is the PM's at merge (renumber to the next free D), not a finding; check only the record's content.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rc-N, severity,
file:line, text, fix in one sentence). Append to reports/construct-red.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-construct-red BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
