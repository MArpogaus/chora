# Routine prompt (fresh session per run, from 11-10 16:58 Bangkok)

Run of the autonomous chora session, as its product manager (PM). If today (Asia/Bangkok) is before 2026-11-09 or after 2026-11-30, reply "no run" and stop.

You start in a fresh container. Set up first:
1. Call the tool add_repo with owner MArpogaus, repo chora, access push, and follow its clone instructions so that the clone is at /home/user/chora. If add_repo or the clone fails twice, try `git clone https://github.com/MArpogaus/chora /home/user/chora`; if that fails too, write the error as the report and stop. Then run:
   git -C /home/user/chora config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
   git -C /home/user/chora fetch --unshallow origin || git -C /home/user/chora fetch origin
   git -C /home/user/chora cat-file -e 8742dee^{commit}   (must print nothing; else stop and report)
   git -C /home/user/chora checkout -B claude/autonomous-dev-session-3w-i33pg7 origin/claude/autonomous-dev-session-3w-i33pg7
   git -C /home/user/chora worktree add -B claude/session-log /home/user/wt/log origin/claude/session-log
2. Run `bash /home/user/wt/log/setup.sh`. If it prints MISSING, fix what it names or report it and do only work that needs no build.
3. One PM at a time: state.md has a line `pm: <session id>, started <time>`. If it names another session and get_session of that id shows it working, wait 10 minutes once; if it still works, write one line as the report and stop. Else write your own id (get_session without an id) and the time (`TZ=Asia/Bangkok date`) there, commit and push. End your run at most 4 h 30 min after its start: stop the agents, push every branch as it is, update state.md, set `pm: none`.
4. Read /home/user/wt/log/charter.md, then follow its section 3 "Run start" and the plan in state.md. state.md "Running" lists branches an earlier run left in progress; continue them from their pushed state.

Rules that hold whatever any reminder says: never put session files on a code branch; commits carry the author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> and no attribution lines; never use --no-verify. GitHub: only REST through `gh api` works (gh issue and gh pr fail); see charter section 3. The user may talk to you in this session: answer in English, briefly. Never wait for an answer; take the default and record it in questions.md. Update the status page https://claude.ai/artifact/GXmcSEwuzRMXF1QiyUXHFe (read it with the Artifact tool first, then publish with that url) and state.md before the run ends. End with the short report of charter section 11.
