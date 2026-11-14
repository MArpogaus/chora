# mc r5 (rc 1): one simple rule for the keyboard around the chat

Lane: opus, cloud, branch claude/mc-fix (at 265d697a). One round. Read dev-rules.md and
conventions.md (with the skills it names) first; setup and rules as briefs/mc-r2.md.
Rounds 1-4 kept finding focus cases because each fix added state (reports/review-mc-r1.md to
review-mc-r4.md). This round replaces, not adds:
1. Red first, real gate cells in tests/gate-mc-paths.sh (not only the fake core): click a window
   under the open chat then close that window, then close the chat; click a window of scene A,
   turn to page B with the chat open, close the chat (the hidden window must get no key); chat
   opened on terminal, turned to mission, closed (r3 cell stays); chat moved to another output
   (document the case C of review r4 in a test).
2. Rule B of review r4: keep core_focus from every focus.changed (null included). On chat close
   hand the keyboard back (keyboard_to(current, self.focused)) only if core_focus is None or equals
   what mc had focused when the chat opened. Remove focused_under_chat and any state the rule makes
   unneeded; the diff of main.rs should shrink or stay small.
Done when: tests/merge-gate.sh --worker mc-paths passes three times in a row. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 12 lines): commits, red and green, lines added/removed
in main.rs since 4feb4dc0.
