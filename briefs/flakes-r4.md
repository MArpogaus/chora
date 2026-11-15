# flakes r4 (rc 3)

Lane: Fable, cloud, branch claude/flakes reset to nothing new: start a new branch claude/flakes-r4
from the session branch claude/autonomous-dev-session-3w-i33pg7 (d349a6da). One round. Stop at
250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it names) first.
Setup as briefs/flakes-r2.md. Items (backlog), red first:
1. F17: words::tests::a_slow_backtrace_is_not_cut_by_the_alarm failed after a container restart
   ("the handler of SIGSEGV is stuck after 10 ms: no backtrace", crates/chora/src/words.rs:469).
   Find why the progress check of the r1 fix still races (cold page cache, a first backtrace that
   loads debug info); fix the cause, no raised timeout without a reason.
2. F16: the doors step showed tiling's column all "missing" with "FAIL: no bar" once (train 8
   gate 1); waybar or the bar module did not come up in time. Reproduce under load; fix the wait
   or the start order.
3. GR1: swap ungrab_all + grab to the kit's start_grabs/end_grabs in xp (wm.rs:1371 first), tiling
   (main.rs:586, 1610), mc (main.rs:960), construct (scene.rs:184); one cell for xp's cube-key
   regrab around a construct hand-over.
4. If context remains: F7, A5, A17 (backlog).
Done when: tests/merge-gate.sh passes three times in a row under load (say the load) and once right
after a cold start (fresh build, dropped caches if you can). Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines) and as reports/flakes-r4.md on claude/session-log.
Rules as briefs/flakes-r2.md, with push only claude/flakes-r4; never end a turn to wait for a long
background job; run long gates in the foreground or in parts.
