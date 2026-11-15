# harness r4 (rc 3, owner 11-15 02:10): close review-harness-r3 (FIX M1, M2)

Lane: new opus worker, branch claude/harness (continue it at 4321f0e8; worktree
/home/user/wt/harness4). One round, small and fixed. Read dev-rules.md, conventions.md, then
reports/review-harness-r3.md. Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (newest).
Stop at 250 k context and hand over.
PM decision (11-15 06:2x): the red-first commit hook refuses only when it proves the miss on a lane
branch. On the base branch itself (HEAD is claude/autonomous-dev-session-3w-i33pg7, or base..HEAD
cannot hold the earlier, pushed test commit) it prints one note and passes; the merge gate's
red-first step stays the enforcing check. Same rule as r3: fewer branches, never block a valid commit.
Items, a red case first for each (tests/merge-gate-selftest.sh or tests/guards.sh):
1. M2: on the session branch: test commit, push, fix commit passes with the note.
2. M1: no-foreign-kill sees commands inside "$(...)": `running "$(pgrep -x chora-xp)"`,
   `kill "$(pidof chora)"`, `pids="$(pgrep chora)"` are refused again; `kill -s KILL $(pgrep -x chora)`
   too. KISS: blank only the literal parts of strings, not substitutions.
3. m1: these pass (warn at most): `for t in pgrep pkill`, `need pgrep pkill`, `pgrep $scope`, an awk
   program that matches /pkill chora/; an arithmetic `<< shift` must not hide the rest of the file.
4. m2: in gate-0.8-plugin-runtime.sh an allow opens only on an allow marker line, not on any comment
   naming clippy::disallowed_methods.
5. m3: correct the claim in reports/harness-r3.md (the session branch is not red on ci-check; the
   red came from the merge with this branch's stricter rules). Keep the 4321f0e8 allow.
Output: the full tests/merge-gate.sh once (serial), green; tests/ci-local.sh once; reports/harness-r4.md
on claude/session-log. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/harness and claude/session-log (report); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job.
