# flakes r2 (rc 3, rc 9): the gate blocker first

Lane: Fable, cloud, branch claude/flakes (at 22b452ed; rebase it on the session branch
claude/autonomous-dev-session-3w-i33pg7 first). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) before you start.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree
/home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.

Items, in this order:
1. F14, now a gate blocker: tests/gate-construct-cube.sh:526 "the construct does not answer
   while the window manager is deaf" failed in every full gate today (mc lane 3 of 3, train 5a
   at 17:14 UTC on 498d9dea plus ci-rc and docs). construct.list does not answer within 2 s
   while chora-xp is stopped (SIGSTOP). Find where the construct waits on the window manager
   and make it answer; no raised timeout. Every merge waits for this.
2. F13 again: the fix of 11-14 10:19 UTC was never pushed and is lost. Redo it: "a click
   during a step of the cube counts", red test "a click during the step back is lost" first.
3. The cube flick under load ("a flick did not go on to the next face", "the window does not
   float during a turn" under stress-ng --cpu 8).
4. Then flakes-r1.md items 1 and 3 (F10/F11 SIGALRM race; F2, F3, F7, A5, A17), 4 if time.

Method: a red commit first (a test or a loop that fails), then the fix; prove each fix with
N runs under load (say N and the load). Push after every commit (a container can die).
Done when: tests/merge-gate.sh passes three times in a row on the branch; list the logs.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 25 lines) and as
reports/flakes-r2.md on claude/session-log: per item fixed / reproduced-not-fixed / not
reproduced, commits, the three gate runs.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push except your own rebase of claude/flakes
before the first push of this round; push only claude/flakes and that report; kill by pid,
never pkill -f; never check out another branch in an existing worktree.
