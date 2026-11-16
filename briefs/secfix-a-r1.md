# secfix a r1 (rc 6): the two security MEDIUM of the area a review

Model: sonnet, effort medium (one bounded fix with tests, one text change)
Lane: cloud, branch claude/secfix-a (session branch 66588880); merge the session branch first. One round.
Stop at 250 k context. Read reports/review-areas-a-r1.md raa-3 and raa-23 (on claude/session-log).

1. raa-23 [sec] (NO-03): plugins/chora-notify/src/bus.rs:69 and crates/chora-kit/src/icons.rs:70-74
   decode any absolute path a D-Bus sender names, SVG included, with no size cap. For a sender
   path: decode PNG only, under a file-size and pixel cap; never SVG; anything else falls back to
   the theme icon by name. Red test first (an SVG path and an oversized PNG from a sender are
   refused, a small PNG still shows).
2. raa-3 [sec]: the launcher runs typed text that matches no row as a command (sh -c). PM
   decision: that is the Run box by design (the user's own typing is not a trust boundary; no
   metacharacter filter, POLS). Fix is the words only: the help text and key list of every look
   say that typed text runs as a command; the docs of `answer` and `launcher::run` keep saying it.
Done when: the red test fails before and passes after, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/secfix-a-r1.md on claude/session-log. Rules as briefs/flakes-r2.md, push only
claude/secfix-a and that report; never end a turn to wait for a long background job.
