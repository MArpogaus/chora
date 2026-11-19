# kit6 r1 (F2): kit::launch (D87 revised)

Model: sonnet, effort high (a kit module on a settled decision record)
Lane: cloud, branch claude/kit6 from dc4bc46d (train 56 head). One round, stop at 250 k. Cost about 2 %.
Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md row KIT6 and C13, docs/decisions.md D87 (pidfd, no waitid).
1. Red first, one commit: FakeCore cell, a program that exits before it maps a window gives `Exited` at
   once (no timeout); a cell for a launch that maps a window.
2. `kit::launch` per D87 revised; xp and tiling launch through it, their own copies deleted in the same merge.
3. tools/copies.toml launch rows lowered by what the ports delete.
4. Visible behaviour unchanged (start menu and launcher acts; known-red only shrinks). No new feature;
   mc and construct only compile fixes.
Done when: cells red then green; merge gate once on the head (poll it in the same turn); kit coverage with
`cargo llvm-cov -p chora-kit --features chora-kit/testing` (if missing, a blocker). Refactor proposals ranked.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit6.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
