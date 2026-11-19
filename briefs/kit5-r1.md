# kit5 r1 (F2): kit::notice, chora-notify the only server

Model: sonnet, effort high (a move onto a settled pattern; KIT1-2, KIT4, KIT6 set it)
Lane: cloud, branch claude/kit5 from a17e7858 (train 57 head). First `git fetch --unshallow`; when the session
branch moves (trains 58, 59), `git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 250 k. Cost about 3 %.
Read reports/arch-plan-fable.md row KIT5, sections 2 (ARC) and I9, on claude/session-log; reports/kit6.md for the pattern.
1. `kit::notice`: one way for every look to show, place and dismiss a notice; chora-notify is the only
   notification server; no look runs its own. zbus leaves chora-kit (`cargo tree -p chora-kit` without zbus).
2. mc: notices get a slot that never covers the content or the chat (vision card: quiet, never covers work).
   Keep it the smallest design that holds; write the choice in the report as a draft decision record.
3. Cells red first: I9 (tests/acts) in every look; a kit unit test through FakeCore for show, replace, dismiss,
   server gone (honest state, no panic).
4. tools/copies.toml: notice rows to 0.
5. Visible behaviour unchanged except the mc slot; no new feature.
Done when: cells red then green; merge gate once on the head (poll it in the same turn); kit coverage of
`notice` with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (missing tool: blocker, do not install).
Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit5.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
