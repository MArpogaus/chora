# secfix c r2 (rc 1, 6): close the three MAJOR of review-secfix-c-r1

Model: opus, effort medium because it is a security fix in voice confirm paths (rule 9)
Lane: cloud, branch claude/secfix-c (continue from cd7a8946). One round. Stop at 250 k context.
Review: reports/review-secfix-c-r1.md (on claude/session-log); fix findings 1-3, 5 (doc only).

1. Red tests first (one commit): security settings.set with an OSC body and with U+009D is refused
   (finding 1); a_question_too_long_for_the_hud_is_refused comes back (finding 2); a detail with ⚠️,
   ZWJ, ZWNJ, NBSP, LRM and \r is NOT refused, one with U+202E, U+2066, ESC, C1, U+200B, U+FEFF is.
2. Finding 3 first: hides() refuses only the narrow set of the review (C0 but \n\t, C1, ESC,
   U+202A-202E, U+2066-2069, U+200B, U+2060-2064, U+FEFF); KISS, one function in chora_kit::chat.
3. Finding 1: client.rs ask_key checks too_long || hides and answers no with a reason to the caller.
4. Finding 2: main.rs answers no before reporting when the own HUD cannot show the whole detail.
5. Finding 5: one doc line on BUTTON_DELAY. Finding 4 to docs/known-issues.md (one line).
Done when: each red test fails before and passes after, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/secfix-c-r2.md on claude/session-log, ending with Refactor proposals. Rules as
briefs/flakes-r2.md; push only claude/secfix-c and that report; never end a turn to wait for a
long background job.
