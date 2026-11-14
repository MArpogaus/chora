# core r3 (rc 6): the review r2 fix

Lane: opus, cloud, branch claude/core-r2 (at 78f28b2d). One round; setup and rules as
briefs/core-r2.md. Items from reports/review-core-r2.md, red first:
1. V02: a test that drives barge-in, queued and asr-dropped paths and a tool call with text
   arguments and asserts no person text at info; then move those lines to debug (tool calls: log
   the tool name at info, arguments at debug). docs/voice.md stays true.
2. C13: draw the mark in front of the cursor; fix docs/architecture.md:483. A test where a large
   client cursor sits on the mark.
3. C05: a docs commit (docs/api.md or docs/voice.md grant table) whose body carries a
   "BREAKING CHANGE:" footer naming output.configure, output.reserve and the two voice tools, so
   cz bump puts it into the CHANGELOG. No history rewrite.
Done when: tests/merge-gate.sh passes. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most
10 lines) and append to reports/core-r2.md.
