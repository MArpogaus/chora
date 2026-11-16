# secfix b r1 (rc 6): the two security MEDIUM of the area b review

Model: sonnet, effort medium (two bounded fixes with tests)
Lane: cloud, branch claude/secfix-b (session branch 66588880); merge the session branch first. One round.
Stop at 250 k context. Read reports/review-areas-b-r1.md rows 60 and 61 (on claude/session-log).

1. rab-60 [sec]: API key_envs reach the coder and ACP agent children (coder.rs:284,
   agents.rs:1342, config.rs:407) and tools/agent-sandbox.sh unsets only CHORA_SOCKET,
   WAYLAND_DISPLAY, DBUS (79-81). Remove the key envs from both Commands (one helper, as for
   the choractl child) and unset them in the sandbox script. Red test first: a spawned child
   sees no key env.
2. rab-61 [sec, charter 11b]: config.rs:1141 accepts a `confirm:` list without run_program,
   which replaces the default and leaves run_program unconfirmed. `check` must reject a confirm
   list in which no rule matches run_program. Red test first.
Done when: both red tests fail before and pass after, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/secfix-b-r1.md on claude/session-log. Rules as briefs/flakes-r2.md, push only
claude/secfix-b and that report; never end a turn to wait for a long background job.
