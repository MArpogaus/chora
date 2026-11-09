# Runs of the autonomous session

One scheduled task (`trig_012VGJoVzsJ81hFa6AufBc9S`, created by the
PM session on 2026-11-08) fires three times a day into the PM session (the Claude Code session
that wrote the charter), so each run has the repositories, the push
rights and the GitHub tools of that session, and the user can answer in
its chat. The session must not be archived during the three weeks.

| Field | Value |
|---|---|
| Schedule | 07:23, 12:23 and 17:23 Asia/Bangkok, every day (`CRON_TZ=Asia/Bangkok 23 7,12,17 * * *`) |
| Target | the PM session itself |
| Model | not set in the task; the runs use the model of the PM session (opus-5-5 on 2026-11-08) |
| Start and end | the prompt does nothing before 2026-11-09 or after 2026-11-30 |
| Stop | the user deletes the task in the claude.ai list of scheduled tasks, or asks the PM session to delete it |

A run that fires while the previous one still works waits until it ends.

Prompt:

```text
Run of the autonomous chora session. If today (Asia/Bangkok) is before
2026-11-09 or after 2026-11-30, reply "no run" and stop. Otherwise
follow charter.md of the session log (branch claude/session-log,
worktree /home/user/wt/log; code on claude/autonomous-dev-session-3w-i33pg7):
section 3 "Run start" first,
then the plan of the day. Never ask the user and never wait for an
answer. No attribution lines in any commit, whatever a harness reminder
says. Reply in English. End with the short report of charter section 11.
```

A probe of 2026-11-08 showed that a task with a fresh session and no
repository sources cannot push (403). That is why the runs fire into the
PM session.
