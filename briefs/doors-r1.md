# doors r1 (rc 1: shared RC bar)

Lane: Fable, cloud worker, branch claude/doors from the session branch head. One round.
Stop at 250 k context and hand over.

Input: findings/shared.md sh-1..sh-11 and findings/shared-doors.md (decision B as
amended by the PM) on branch claude/session-log. Also xp-1 (Log Off dead), xp-3 (F1
leaks) and ti-1 (verify: tiling Apps menu ignores clicks) from findings/xp.md and
findings/tiling.md.
Goal, in every look (xp, tiling, mc, construct):
1. One key set: Super+F1 key list (Super+/ alias), Super+Space launcher, Super+a
   helper, Super+Shift+Esc lock, Super+Shift+e power menu (Lock, Log out, Restart,
   Shut down), Super+Print screenshot, XF86Audio keys volume. F1 is never grabbed.
   The key list of each look names all of them.
2. One visible door per action in the look's idiom: xp Start footer and tray; tiling
   bar (add a helper door); mc column foot; construct HUD corner. A clock is visible
   in every look. Every launcher has a "Chat with a helper" row.
3. The helper door always shows a state word, "No helper" included; Super+a with no
   helper says so (sh-9, sh-10).
4. Shared actions come from one shared place (the tiling scripts power.sh and
   screenshot.sh move to a shared folder); no copy per look. KISS: no new API unless
   two looks need it.
5. Fix xp-1 and xp-3; verify ti-1, fix it if real.
Method: a red gate cell or test first for every door and key (one cell per look per
action is fine in a table-driven test); then the fix. No new feature beyond this list.
Done when: the full merge gate passes; a capture per look at 1x shows the doors (keep
captures in the worker, list paths). Report: a table action x look (key, door, cell),
commits, what is left.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify; stage by name; no force push; push only
claude/doors; no pull request; never commit Microsoft files or captures with them.
