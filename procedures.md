# PM procedures (injected after compaction with rules, routines, state)

Identity on every commit: git -c user.name="Marcel Arpogaus" -c user.email="38564291+MArpogaus@users.noreply.github.com".

## Log commit
fetch and merge origin/claude/session-log; commit; push HEAD:claude/session-log (reject: fetch, merge, push).
check_log: state, routines, rules, procedures <= 3000 bytes (procedures by hand), decisions-now <= 4000; state lines <= 300 chars;
Now <= `date -u`; a session-branch change needs a status/ change; the budget line names tokens.py HH:MM.

## Train (owner 07:01: batch)
1. git worktree add -q --detach /home/user/wt/trainN <session head sha>.
2. Each MERGE-OK sha in merge order: git -c identity merge -q --no-ff <sha>
   -m "Merge branch 'claude/X' into claude/autonomous-dev-session-3w-i33pg7".
3. Background, timeout 7200000: (tests/merge-gate.sh --push claude/autonomous-dev-session-3w-i33pg7
   > /tmp/claude-0/trainN.log 2>&1; echo EXIT $? >> /tmp/claude-0/trainN.log). --jobs 1 until flakes-l r2 merges (4 reds at 2-3, green at 1; 11-20).
4. Red: bisect by halves; a real fault goes back to its lane. After: worktree remove --force,
   rm -rf ~/.cache/sccache, df -h /; state.md, status page, Queue realigned.

## Lane start (first: the finding is open in its ledger and its cell is red on the session head)
Brief (Model line `^Model: (haiku|sonnet|opus|Fable)`, opus/Fable "because <reason>") -> Lanes row naming the brief
-> commit, push -> git push origin <fullsha>:refs/heads/claude/X (fetch the object first) -> tools/start_lane.sh
-> create_session(source_url https://github.com/MArpogaus/chora, source_revision claude/X, model claude-sonnet-5-5 |
claude-opus-5-5 | claude-haiku-5-5, effort, tags ["chora-lane"], prompt "You are a build lane of the chora session.
Read your brief with `git fetch origin claude/session-log && git show origin/claude/session-log:briefs/X.md` and
follow it exactly, in this same turn, until done or blocked. The PM session is <pm id>.") -> send_later 6-7 min
start check -> session id into Lanes. Step 0: no setup.sh (auto mode refuses it in lanes, 11-20 00:48); rely on env setup. Handback: archive the worker.

## Seeded review (owner 06:12)
Base..head and diffstat; the brief's done criteria as checks; known findings by id; a command and files per check;
no gate (cargo test -p only); <= 120 k, <= 0.5 %, <= 8 findings; computed verdict. Template briefs/review-wm2-r1.md.
Sonnet high for ports; opus for core, design, security; a PM review for small diffs (PM not the builder).

## Ids
Status page: status/index.html -> publish with url=https://claude.ai/artifact/GXmcSEwuzRMXF1QiyUXHFe (always pass url).
PM watchdog trig_011UMxwYQRqwjHmDdYnz6mQa (every 2 h). Transcript of the PM: /root/.claude/projects/-home-user/.
