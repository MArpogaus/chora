# wm2 r3 (F3): merge the session branch into wm2 and resolve the book-focus conflict

Model: sonnet, effort high (a named merge of two reviewed parts; no new design)
Lane: cloud, branch claude/wm2 at 12305e5a (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Train 59 left wm2 out: it conflicts with wm1b/wm1c (the book port of mc).
1. `git fetch origin claude/autonomous-dev-session-3w-i33pg7 && git merge FETCH_HEAD` (base 1d1aa5fd or later).
2. Resolve docs/api.md, plugins/chora-mc/src/manage.rs, plugins/chora-mc/src/windows.rs so both hold:
   the book (`self.book.give/refused/confirm`, `settle(.., &Named::Ask)`, `Named`, `fn gave`) AND
   `chora_kit::wm::focus` (`self.focus.gave(&surface)`, `impl Policy for Mc`). One focus state:
   drop the old `self.focused` reset where `self.focus` replaces it. docs/api.md: keep the wm1c text
   and add the wm2 focus paragraph.
3. No other change. A cell that breaks: fix in the merge, name it in the report.
Done when: `cargo test -p chora-mc`, `-p chora-tiling`, `-p chora-kit --features testing`, `-p chora input::`
green; `tests/merge-gate.sh --only keys,pane-keys,wm-pane,acts,mc-handover,mc-paths` green (poll in the same turn).
No full gate (the train runs it). Push claude/wm2.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
how each file was resolved) and append "## r3" to reports/wm2.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
