# flakes r1 (rc 3, rc 9)

Lane: Fable, local, worktree /home/user/wt/flakes on branch claude/flakes from the
session branch head after train 4. One round. Stop at 250 k context and hand over.

Goal: the merge gate green three times in a row (rc 3). Items, in this order
(backlog.md "Flakes and gates", each with its evidence):
1. F10/F11: words::tests::a_stack_overflow_leaves_its_words got signal 14 (SIGALRM)
   instead of 11 under load (train 4 gate 1, 11-14 04:55 UTC). Fix the cause, not
   the wait: the alarm must not race the backtrace.
2. F13, F14, C2: cube navigate-click, "construct does not answer while the wm is
   deaf", "a client xp runs already" (PR_SET_PDEATHSIG for the cube's xp child).
3. F2, F3, F7, A5, A17: reproduce under load (stress-ng or parallel runs); fix the
   ones that reproduce; for the rest write why they cannot recur or keep the item.
4. F4, F5, F6: only if 1-3 are done.

Method: for each fix a red commit first (a test or a loop that fails), then the fix.
Prove each fix with N runs under load (say how many, how loaded). No raised timeouts
without a reason in the commit.
Done when: tests/merge-gate.sh passes three times in a row on the branch, with the
runs' logs listed in the report. Report: per item fixed / reproduced-not-fixed /
not reproduced, commits, the three gate runs.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify; stage by name; no force push; push only
claude/flakes; kill by pid, never pkill -f.
