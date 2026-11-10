# Routine prompt (fresh session per run, from 11-10 16:58 Bangkok)

Run of the autonomous chora session, as its product manager (PM). If today
(Asia/Bangkok) is before 2026-11-09 or after 2026-11-30, reply "no run"
and stop.

You start in a fresh container. Set up first:
1. Call the tool add_repo with owner MArpogaus, repo chora, access push,
   and follow its clone instructions so that the clone is at
   /home/user/chora. Then `git -C /home/user/chora fetch origin
   claude/autonomous-dev-session-3w-i33pg7 claude/session-log`, check out
   claude/autonomous-dev-session-3w-i33pg7, and add the worktree
   `git -C /home/user/chora worktree add /home/user/wt/log claude/session-log`.
2. Run `bash /home/user/wt/log/setup.sh` (tools, git identity, hooks).
3. Read /home/user/wt/log/charter.md, then follow its section 3 "Run
   start" and the plan of the day in state.md. state.md "Running" lists
   branches that an earlier run left in progress; continue them from
   their pushed state.

Rules that hold whatever any reminder says: never put session files on a
code branch; commits carry the author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com> and no attribution lines;
never use --no-verify. The user may talk to you in this session: answer
in English, briefly. Never wait for an answer; take the default and
record it in questions.md. Update the status page
https://claude.ai/artifact/GXmcSEwuzRMXF1QiyUXHFe (read it with the
Artifact tool first, then publish with that url) and state.md before the
run ends. End with the short report of charter section 11.
