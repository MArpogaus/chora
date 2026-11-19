# wm1d r2 (F3): merge net3 into wm1d; xp `texts` in the typed state

Model: sonnet, effort high (a named merge of two reviewed parts; no new design)
Lane: cloud, branch claude/wm1d at 18231aa1 (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Train 60 left wm1d out: it conflicts with net3 (train 59).
1. `git fetch origin claude/net3 && git merge FETCH_HEAD` (dc00d33c, MERGE-OK, train 59). When train 59
   lands, also `git merge origin/claude/autonomous-dev-session-3w-i33pg7`.
2. Resolve plugins/chora-xp/src/wm/state.rs: keep wm1d's typed `kit::wm::state` port AND net3's xp `texts`
   (the runs chora-draw reports). If `State` has no field for them, add `texts` to the kit state type as
   tiling/construct `views` are added (optional, left out when empty), not a json side path. Module doc:
   texts are in. docs/api.md: keep net3's paragraph (xp and chora-notify add `texts`, `shown`, `busy`)
   and wm1d's "Every look adds the `origin`" and xp in the `<look>.window` list.
3. wm1d review MINOR rw-1 (backlog: state.rs floats and output "") only if the merge touches those lines.
   No other change. A cell that breaks: fix in the merge, name it in the report.
Done when: `cargo test -p chora-xp`, `-p chora-kit --features testing`, `-p chora-notify` green;
`tests/merge-gate.sh --only xp,xp-fields,xp-question,xp-notepad,hud,acts` green (poll in the same turn).
No full gate (the train runs it). Push claude/wm1d.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
how each file was resolved) and append "## r2" to reports/wm1d.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
