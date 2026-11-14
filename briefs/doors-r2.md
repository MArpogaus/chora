# doors r2 (rc 1: shared RC bar), finishing r1

Lane: Fable, cloud, branch claude/doors (at ea47e27b). One round. Stop at 250 k context
and hand over. Read dev-rules.md and conventions.md (with the skills it names) first.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree
/home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.

Scope: briefs/doors-r1.md goals 1 to 5, unchanged. Start from the review
reports/review-doors-r1.md (FIX: dr-1, dr-2 BLOCK, dr-3 MAJOR; MINORs dr-4 to dr-7):
1. Write tests/doors/{xp,tiling,mc,construct}.sh so gate-doors.sh runs (red cells first).
2. Then make every cell green: key set, one door per action, clock, helper state word
   (use Word::NoHelperDoor or drop it), xp-1, xp-3, ti-1.
3. dr-4: drop the shared API that no second look calls; dr-5 is accepted as is.
4. Merge the session branch claude/autonomous-dev-session-3w-i33pg7 into claude/doors
   whenever it moves (train 5 brings claude/xtc-fix); then fix xr-3/dr-6: the tiling
   Apps list reads flavors/shared/applications too.
Known: gate-tiling-power fails on the session branch as well (fuzzel 1.9.2 rejects
selection-radius); not this lane's item, do not count it.
Method: red cell first, then the fix. Push after every commit (a container can die).
Done when: the full merge gate passes except the known F14 cube cell, which the flakes lane
fixes; a capture per look at 1x shows the doors (list the paths).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 25 lines) and as
reports/doors-r2.md on claude/session-log: a table action x look (key, door, cell),
commits, what is left.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/doors and that
report; never commit Microsoft files or captures with them; never check out another
branch in an existing worktree.
