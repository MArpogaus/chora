# security a r5 (rc 6, rc 1): prove a person's chat-row click still opens the chat

Model: sonnet, effort medium (small test round; the gaps are named)
Lane: cloud, branch claude/security-a (250b3716); merge the session branch first. One round.
Stop at 250 k context. Read dev-rules.md and conventions.md, then reports/review-security-a-r4.md
(FIX: 1 MAJOR, 3 MINOR).

1. MAJOR (host.rs:373, gate-hud.sh:108): a test that a person's (not injected) press on the
   "Chat ..." row opens the chat, driving chat_pressed(ROW, false) or the assistant event path,
   and the same for the figure. Mutation check: break the open path, the test fails (say so in
   the report). If the gate can mark a press as a person's, restore a positive hud cell beside
   the refusal cell; if not, say why and leave the backlog line.
2. MINOR: tests/injected-optout.py refuses `injected: ref _x`, `x @ _`, `(_)`, `_ | true` and a
   plain name bound to the flag; guards.sh cases for each.
3. MINOR: one driven wiring test each for mc and assistant `event` with an injected pointer
   event (deleting the hears_pointer line must fail it); the kit test also covers hears_pointer.
Not in scope: r3 point 9; the two host-bound tests.
Done when: items 1-3 pass, clippy -D warnings, tests/merge-gate.sh once. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines) and as reports/security-a-r5.md on
claude/session-log. Rules as briefs/flakes-r2.md, with push only claude/security-a and that
report; never end a turn to wait for a long background job.
