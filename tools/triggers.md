# Scheduled tasks of the PM (recreate on takeover; bound to the PM session unless noted)

## PM watchdog, cron `15 */2 * * *`, bound to the PM session
PM watchdog (every 2 h, rules.md Routines). First check that /home/user/wt/log is on claude/session-log (else run tools/takeover.sh). Check the Lanes table of /home/user/wt/log/state.md: get_session for each cloud worker listed there; act on finished or stuck lanes (review, fix round, merge). If nothing finished, got stuck or changed, reply only "watchdog: no change" and stop, with no commit. Otherwise rewrite state.md (one log commit) and reply in two lines plus done, running, next, blocked, budget.

## PM self-check, cron `50 12 * * *`, bound to the PM session
PM daily self-check (rules.md, routines.md). 1) Answer the three questions in rules.md (RC burn-down on track? reviewed branch unmerged > 12 h? anything running not in the Queue?). 2) Spawn one independent opus critic (read-only, own context) on the last 24 h of state.md, answers.md, friction.md and the session-log and session-branch commits; it lists what the PM did wrong, missed or overstated. 3) Answer each critic point and the three questions in one block of reflections/selfcheck-<UTC date>.md, time from date -u. Each "no" or accepted point gets its action now or tops the Queue. One log commit. Reply in two lines plus done, running, next, blocked, budget.

## PM heartbeat, cron `40 */2 * * *`, bound to the host session session_016iS3kMDji77DaAkJwsruvW (haiku, with the repo); verified 18:03 UTC ("heartbeat: ok")
Sessions without a repo never start in this environment (friction 45), so the heartbeat runs in a small
persistent haiku session that has the repo. On a takeover only the PM id in the prompt changes (or the
prompt reads it from state.md). Prompt: see trigger trig_01Km6VqUaYY2zYz476YHk5AK (trig_01Pwz3VkyKtmTZhv46srXYvG on the first host is disabled); on an alarm it pushes
to the owner with PushNotification. The first heartbeat trigger trig_01UQ3xs5ZbNC3LAKuMB7px8R (fresh
session, no repo) is disabled.
