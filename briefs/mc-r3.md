# mc r3 (rc 1): hand the keyboard back when the chat closes

Lane: sonnet, cloud, branch claude/mc-fix (at a39fbdb7). One round; same rules and setup as
briefs/mc-r2.md. Items from reports/review-mc-r2.md:
1. MC-1c first (red): extend cell terminal-chat so that after the chat closes typed text reaches
   the terminal, plus the reverse case (chat opened on terminal, page turned to mission, chat
   closed: no key reaches the hidden terminal); a unit test of keyboard_to with Chat::Open.
2. MC-1a: on chat close apply the deferred keyboard_to(current, self.focused.clone()).
3. MC-1b: a window the user starts while the chat is open gets the keyboard when the chat closes.
Done when: tests/merge-gate.sh --worker mc-paths passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): commits, red and green.
