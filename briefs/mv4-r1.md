# mv4 r1 (F4): split mc region.rs and voice chat.rs under 1,500 lines

Model: sonnet, effort high (a mechanical split with tests; MV1-3 set the pattern)
Lane: cloud, branch claude/mv4 from a17e7858 (train 57 head). First `git fetch --unshallow`; merge
origin/claude/autonomous-dev-session-3w-i33pg7 when it moves. One round, stop at 250 k. Cost about 2.5 %.
Read reports/arch-plan-fable.md F4 lines (336-337) and reports/mv3.md on claude/session-log.
1. mc region.rs (2,272) and voice chat.rs (1,536) each below 1,500 production lines, split by responsibility,
   counted with the command of reports/debt-rc.md; a test module moved out is not a drop.
2. No behaviour change: existing tests unchanged and green; move unit tests with their code.
3. Fix on the way only with a red cell first: backlog UX-D1 (list text stays after Escape; region.rs:642) and
   SB2 chat.rs:1303 fit() cut "user:" line. Nothing else.
4. reports/debt-<date>.md with the new counts, lower than debt-rc.md.
Done when: line counts by the command; merge gate once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/mv4.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
