# ci f25 r1 (rc 4): CI e2e agent.sh step 6 red twice

Model: sonnet, effort medium (one test harness item, cause named in backlog F25)
Lane: cloud, new branch claude/ci-f25 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (2d9f306f or newer). One round. Stop at 250 k context
and hand over. Read dev-rules.md and conventions.md (with the skills it names) first.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree
/home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.

Item: backlog F25. CI e2e (PR #106) failed the same way on 0e303d60 and on 2d9f306f:
tests/agent.sh step 6 "pixel 5 5: 10 8 15 255, the host shows 19 15 28"; green on 61d885f0.
The capture and the grim shot are two moments; the corner may change between them (tiling
bar appears late: waybar.sh waits 10 s when choractl is not on PATH). Not a flake: twice.
1. Find which it is: grim before and after the capture; if the host changes between them it
   is timing, if the host is stable and the capture differs it is a capture bug (then stop
   and report, that is a core finding, not yours to patch).
2. Timing: make the red case first (a commit that delays the bar and fails step 6 locally),
   then wait for the condition (the bar drawn, or a stable host frame), never a sleep.
3. Run tests/agent.sh 10 times under stress-ng --cpu 4 (say the load) and once cold.
Done when: step 6 passes 10/10 under load; tests/merge-gate.sh passes once.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as
reports/ci-f25-r1.md on claude/session-log: cause, commits, runs, the earliest gate that
would have caught it.
Rules as briefs/flakes-r2.md, with push only claude/ci-f25 and that report; never end a turn
to wait for a long background job.
