# kit2c r2 (F2): merge wm2 into kit2c; tiling focus and actions together

Model: sonnet, effort high (a named merge of two reviewed parts; no new design)
Lane: cloud, branch claude/kit2c at c29a3e9c (continue it; review r1 MERGE-OK). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Train 60 (mv4, wm2) left kit2c out: it conflicts with wm2 in tiling.
1. `git fetch origin claude/wm2 && git merge FETCH_HEAD` (8db02220, MERGE-OK). When train 60 lands, also
   `git merge origin/claude/autonomous-dev-session-3w-i33pg7`.
2. Resolve plugins/chora-tiling/src/drag.rs and main.rs so both hold: wm2's `chora_kit::wm::focus`
   (Focus, Policy; tiling ShownFocus gone; click moves the keys, hover does not, D55) AND kit2c's
   `kit::actions` binding (Common as Host, no own action table). No other change; a cell that breaks: fix
   in the merge, name it in the report.
Done when: `cargo test -p chora-tiling`, `-p chora-kit --features testing`, `-p chora input::` green;
`tests/merge-gate.sh --only keys,pane-keys,wm-pane,acts,tiling-audit,walk` green (poll in the same turn).
No full gate (the train runs it). Push claude/kit2c.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
how each file was resolved) and append "## r2" to reports/kit2c.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
