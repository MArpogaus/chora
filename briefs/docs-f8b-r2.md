# docs-f8b r2 (F8): D104 states the merged form

Model: sonnet, effort low (two sentences of docs)
Lane: cloud, branch claude/docs-f8b at 2d0d0da1. One round, stop at 100 k. Cost about 0.2 %.
Read reports/docs-f8b.md "## PM check r1" on claude/session-log.
1. rw-1: D104 says what kit6 does today (`{ }` wrapper, crates/chora-kit/src/launch.rs:202 on claude/kit6, so
   `exit N` and a replaced shell leave no status) and names the open point: kit6 review rw-1, the next kit lane
   (KIT2c) moves it to a subshell. Same pattern as the open points of D65.
2. rw-2: add "the status file stays".
Done when: pre-commit and gate-lib green; poll them in the same turn; never end a turn to wait.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 3 lines)
and append "## r2" to reports/docs-f8b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report.
Commits by the owner identity, no attribution lines, never --no-verify.
