# mc r2 (rc 1): the review fix

Lane: sonnet, cloud, branch claude/mc-fix (at 4feb4dc0). One round. Stop at 250 k context
and hand over. Read dev-rules.md and conventions.md (with the skills it names) first.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree
/home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.
Items (reports/review-mc-r1.md):
1. MC-1 MAJOR plugins/chora-mc/src/main.rs:2669 keyboard_to: with the chat open a page turn
   moves the keyboard from the chat field to the page. Red first: extend cell terminal-chat of
   tests/gate-mc-paths.sh (Super+a, Super+Right, then typed text must reach the chat field);
   then skip FocusNode while Chat::Open.
2. MC-2: add tests/gate-mc-paths.sh as a step of tests/merge-gate.sh (it passes on the branch).
Nothing else (rules.md 6). Push after every commit.
Done when: `tests/merge-gate.sh --worker mc-paths` passes (if the step name differs, name it);
the cube cell (F14) may fail until claude/flakes merges, do not count it.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): commits, red and green.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/mc-fix; never check
out another branch in an existing worktree.
