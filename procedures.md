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
   Then for r in sleep size lint: tools/$r-ratchet (secs); a breach: back to its lane, no train (t128).
3. Bash run_in_background (2 h), CARGO_BUILD_JOBS=2: (tests/merge-gate.sh --push claude/autonomous-dev-session-3w-i33pg7
   > /tmp/claude-0/trainN.log 2>&1; echo EXIT $? >> /tmp/claude-0/trainN.log). --jobs 2. Oldest first.
4. Red: bisect by halves; a real fault goes back to its lane. After: worktree remove --force,
   rm -rf ~/.cache/sccache, df -h /; state.md, status page, Queue realigned.

## Lane start (first: the finding is open and its cell red on the session head; briefs name only shas on origin)
Brief (Model line `^Model: (haiku|sonnet|opus|Fable)`, opus/Fable "because <reason>") -> Lanes row naming the brief
-> commit, push -> git push origin <fullsha>:refs/heads/claude/X -> tools/start_lane.sh
-> create_session(source_url https://github.com/MArpogaus/chora, source_revision claude/X, model claude-sonnet-5-5 |
claude-opus-5-5 | claude-haiku-5-5, effort, tags ["chora-lane"], prompt "You are a build lane of the chora session (PM <pm id>, your parent). Your brief is briefs/X.md on
claude/session-log; its full text is below. Follow it exactly, in this same turn, until done or blocked." + the brief text
-> send_later 6-7 min
start check -> session id into Lanes. Step 0: no setup.sh; env setup. Handback: archive the worker. Product code: the lane runs the full gate, no --only (t99b).

## Seeded review (06:12)
Base..head and diffstat; the brief's done criteria as checks; known findings by id; a command and files per check;
no gate (cargo test -p only); a failed Done-when line (gate incomplete, red): INCOMPLETE, not MERGE-OK; a fix with no red test commit before it is MAJOR; PM review only under 200 lines or a pure move; <= 120 k, <= 0.5 %, <= 8 findings; computed verdict. Template briefs/review-wm2-r1.md.
Opus high for look, core, design, security reviews (critic 11-24); sonnet for ports; PM review for small diffs.

## Ids
Status page: status/index.html -> publish with url=https://claude.ai/artifact/GXmcSEwuzRMXF1QiyUXHFe (always pass url).
PM watchdog trig_011UMxwYQRqwjHmDdYnz6mQa (every 2 h). PM transcript: /root/.claude/projects/-home-user/.
