# mc r4 (rc 1): the review r3 fix

Lane: sonnet, cloud, branch claude/mc-fix (at 8595d65e). One round; same rules and setup as
briefs/mc-r2.md. Items from reports/review-mc-r3.md:
1. F1 (MAJOR), red first: a gate cell or unit test where the chat is open on another output,
   and one where the user clicks a window while the chat is open; keyboard_back must not take
   the keyboard in either case. Fix: hand back only when the chat really closed (data.open != true)
   and no other client took focus meanwhile.
2. F2 (regression of this branch): a docked app that had the keyboard before the chat opened gets
   it back on close. Red first.
3. F3: clear the RETURN trap in gate-mc-paths.sh.
F4 (the lost first key) is not this lane's; keep the leading space, the flakes lane traces it.
Done when: tests/merge-gate.sh --worker mc-paths passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): commits, red and green.
