# ux r4 (rc 1): close review-ux-r3 (FIX M1)

Lane: new opus worker, branch claude/ux-r3 (continue it at 42d5d4b2; worktree /home/user/wt/ux-r4).
One round, small and fixed. Read dev-rules.md, conventions.md, then reports/review-ux-r3.md and
reports/ux-r3.md. Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (newest).
Stop at 250 k context and hand over. ti-1 is out of scope (waits for the owner).
Items, a red unit or gate cell first for each:
1. M1: the chat header never grows past two lines in any look (plain, glass, bubble), at any
   chat.width and prompt length. The no-helper words get their two lines whole (as now); a picked
   helper's `name · state · task` keeps the name and state whole and ends the task in `…`. Stop and
   Close stay where they are for a 1800-character prompt at chat.width 240 and at 1024x768.
2. MINOR: the glass header without a state word obeys the same cap and never crosses the separator.
3. MINOR: the hud-chat meta calls a line whole only if it fits in width and height (a line cut at
   the bottom is not whole).
Keep KISS: one cap rule for all three looks, no new options.
Output: the full tests/merge-gate.sh once, green; reports/ux-r4.md on claude/session-log (item, red
commit, fix commit, cell, 1x captures). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most
15 lines).
Rules: push only claude/ux-r3 and claude/session-log; author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job; install nothing.
