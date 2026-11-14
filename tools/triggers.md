# Scheduled tasks of the PM (recreate on takeover; bound to the PM session unless noted)

## PM watchdog, cron `15 */2 * * *`, bound to the PM session
PM watchdog (every 2 h, rules.md Routines). First check that /home/user/wt/log is on claude/session-log (else run tools/takeover.sh). Check the Lanes table of /home/user/wt/log/state.md: get_session for each cloud worker listed there; act on finished or stuck lanes (review, fix round, merge). If nothing finished, got stuck or changed, reply only "watchdog: no change" and stop, with no commit. Otherwise rewrite state.md (one log commit) and reply in two lines plus done, running, next, blocked, budget.

## PM self-check, cron `50 12 * * *`, bound to the PM session
PM daily self-check (rules.md Routines). Answer the three questions in rules.md (RC burn-down on track? reviewed branch unmerged > 12 h? anything running not in the Queue?) in one block of reflections/selfcheck-<UTC date>.md, time from date -u. Each "no" gets its action now or tops the Queue. One log commit. Reply in two lines plus done, running, next, blocked, budget.

## PM heartbeat, cron `40 */2 * * *`, fresh session per fire, haiku, push on
Read-only liveness check of the chora PM session PM_ID. Call get_session for PM_ID and list_events for it with kinds ["result"] and limit 20. Alarm when: the session is archived or its status_bucket is failed; or one of its last three result events contains "blocked by hook" or "Compaction blocked"; or its newest result event is older than 3 hours (the watchdog fires every 2 hours). A result that only reports a usage limit is no alarm. Healthy: reply exactly "heartbeat: ok". Alarm: reply one line "PM DOWN: <reason>, since <UTC time>; take over with tools/takeover.sh (README of claude/session-log, section PM down)". Do not message, change or archive any session.
