# wm3 r2 (F3): close the review of wm3 (reports/wm3.md "## Review r1")

Model: sonnet, effort high (named fixes of a reviewed port; no new design)
Lane: cloud, branch claude/wm3 at 0e0bd700 (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first, then `git merge origin/claude/autonomous-dev-session-3w-i33pg7`.
1. rw-1 MAJOR, red first (own commit): a cell in tests/test_acts.py feeds I12 a state in the shape xp
   writes today (open entry with `by`, as plugins/chora-xp/src/wm/state.rs) and expects the system menu
   or modal of the fullscreen window to be seen; it fails at the red commit. Fix: invariants.py:491, 512
   read `by`; the fixtures test_acts.py:694, 703 use `by`. grep that no reader of `for` is left.
2. rw-2 and rw-3 MINOR: xp's Escape follows `open.escape()` (newest first), so `state.open` and Escape
   agree; recency is then the one order. A cell: open Help panel, then Start menu; Escape shuts the menu
   first. Name the order in docs/api.md.
3. rw-5 MINOR: docs/api.md:1544, one sentence: every look fills `open` (`id`, `kind`, `modal`, optional
   `by`; kinds menu, popup, dialog, sheet, launcher, switcher).
4. rw-6 NIT: drop the unused `Why` argument of `shut_at` (or rename the type `Shut`); one `Why` in kit::wm.
5. rw-4: no code; record in reports/wm3.md "accepted for now: mc and construct build `Open` from their
   own state; xp keeps its models beside the list" and a ranked refactor proposal.
Done when: cells red then green; `cargo test -p chora-kit --features testing`, `-p chora-xp`, `-p chora-mc`,
`-p chora-construct`, `-p chora-tiling` green; `python3 -m pytest tests/test_acts.py -q` green;
`cargo build --workspace --locked` then `tests/merge-gate.sh --only xp,acts,keys,hud` green (poll in the same
turn). No full gate (the train runs it). Push claude/wm3.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
red sha, each finding closed with its cell) and append "## r2" to reports/wm3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
