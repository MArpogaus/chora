# flakes r6 (rc 3: three greens in a row)

Model: Fable, effort high (because three flakes survived five rounds and block rc 3; owner 11-16 16:49)
Lane: cloud, new branch claude/flakes-r6 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (6a067e39 or newer). One round. Stop at 250 k context
and hand over. Read rules.md, conventions.md (with the skills it names) and reports/flakes-r5.md
first; setup as briefs/flakes-r2.md. Items (backlog.md lines F26, F28, F17), red first, cause
before fix, no retry or longer timeout that hides a cause:
1. F26 doors: mc screenshot-door, volume-door, launcher-chat, helper-key "missing" (1 of 3 runs on
   13f86740). Dump of train 22: the hit nodes sit at x 888 and 988 while surface s1 ends at x 832
   (mc column), so the click lands on a hit node and mc does nothing. Logs: train22-gate1-doors.log,
   logs/testwatch/13f86740-doors-1138.log. Reproduce with `tests/merge-gate.sh --only doors` in a
   loop under load.
2. F28 chat-keys: "text typed before the pick did not reach the picked helper" (2 of ~5 runs).
3. F17 words::tests::a_slow_backtrace_is_not_cut_by_the_alarm (crates/chora/src/words.rs:469).

## Done when
Each item: cause named with evidence, red test commit, fix, then 10 runs of its cell pass under
load (report the counts). reports/flakes-r6.md on claude/session-log; push the branch; hand back.
No other scope. If an item resists after a bounded try, report the cause found so far and stop.
