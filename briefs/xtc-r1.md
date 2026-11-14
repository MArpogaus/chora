# xtc r1 (rc 1): xp, tiling, construct MAJORs

Lane: Fable, cloud, branch claude/xtc-fix from the session branch head. One round.
Input: findings/xp.md xp-2, xp-4; findings/tiling.md ti-2, ti-3, ti-4;
findings/construct.md co-1 (BLOCK), co-2 (claude/session-log). Not yours: xp-1, xp-3,
ti-1, co-4 (lane doors), co-3 (lane flakes); do not touch the shared doors.
Goal: every listed item fixed or, for "verify", shown fixed with a capture and a gate
cell. co-1 first: no lost pane after an mc drag onto the construct output (a gate
cell that drags and checks the panes). xp-2: "Nothing found" in Run, no silent close.
xp-4: a plain text editor entry mapped to the configured editor, installed programs
only. ti-2: Apps list sorted, Screenshot not preselected, empty search says so.
Method: red cell first, then the fix; one capture per fix at 1x (keep in the worker).
Done when: the full merge gate passes. Report: per id fixed/verified/open with cell
and capture path, commits.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify; stage by name; no force push; no pull request;
never commit Microsoft files or captures with them. Stop at 250 k context and hand over. Push only claude/xtc-fix.
