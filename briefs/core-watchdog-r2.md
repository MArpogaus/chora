# core watchdog r2 (rc 3, rc 4): fix the review r1 MAJOR (summed waits excuse a costly scene)

Model: sonnet, effort high (core safety code; cause, fix and gate are named in the review)
Lane: cloud, branch claude/core-watchdog (cdaea871), merge the session branch
claude/autonomous-dev-session-3w-i33pg7 first. One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first, then
reports/review-core-watchdog-r1.md. Setup as briefs/flakes-r2.md.

Class invariant: the excused wait of a frame is never larger than the frame's wall-clock time
spent waiting; a costly scene is always blamed, whatever the host load or thread count.

Items, in order:
1. Red first: unit test next to a_costly_scene_beside_idle_threads_still_blames:
   Tasks::frame(0, 10) (4 threads each waiting 10 ms of a 41 ms frame) must Blame. Commit red.
2. Fix crates/chora/src/safety.rs:84-89, 131: use the largest per-thread wait delta (bounded by
   wall time), not the sum. KISS. All safety tests and clippy -D warnings green.
3. tests/agent.sh:180: add "busy" to the grep of the watchdog lines. tests/agent.sh:167-171: a
   failed grim prints a fail message instead of a silent set -e stop.
4. Remove CHORA_FRAME_BUDGET_US=500000 and its header lines from tests/gate-tiling-audit.sh;
   run the 2560x1600@2 cell 3 times under 8 busy loops on 4 cores (say the load) and
   gate-watchdog-starved once.
Not in scope (backlog): the per-frame /proc cost (MINOR), the cfg(test) path (MINOR).
Done when: items 1-4 pass and tests/merge-gate.sh passes once. If the audit fails under load
with the default budget, keep the export, say so with numbers, and hand back; no new tolerance.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as
reports/core-watchdog-r2.md on claude/session-log, with the earliest gate per item.
Rules as briefs/flakes-r2.md, with push only claude/core-watchdog and that report; never end
a turn to wait for a long background job.
