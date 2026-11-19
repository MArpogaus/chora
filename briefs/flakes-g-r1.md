# flakes-g r1 (F5): the flakes left after flakes-f

Model: opus, effort high, because the causes are timing races across core round trips, gate scripts and load
(hard causes); each red train costs about 55 min of the one serial merge gate
Lane: cloud, branch claude/flakes-g from claude/flakes-f 847f0964 (MERGE-OK, train 63). One round, stop at 250 k.
Cost about 3 %. Run `git fetch --unshallow` first; merge origin/claude/autonomous-dev-session-3w-i33pg7 at start
and again before the gate (resolve to the session side's types).
Seeds (read these, not the whole repo): reports/flakes-f.md ("Found, not fixed" and "Review r1"), backlog.md
F26 (line 164), F28 (173), FF1 (last line) on claude/session-log. Load: tests/under-load.sh (4 busy loops).
1. cube.open over 300 ms under load (tests/gate-construct-cube.sh:656; plugins/chora-construct/src/cube.rs:1142
   `open_cube`, about 8 sequential round trips): red cell under load, then fewer round trips (pipeline the
   four output.add or one call with a count); a kit Conn API change stays small and documented in docs/api.md.
2. F26 doors "missing" (lead: train 22 dump, a click lands on a hit node over mc's surface s1) and
3. F28 chat-keys "text typed before the pick did not reach the picked helper": for each a cause with file:line
   and a reproducer that fails reliably under load, red first, then the fix of the cause.
4. FF1: rw-1 gate-tiling-audit.sh:86 wait in seconds, not iterations; rw-2 a test of the write_all short-write
   path (crates/chora/src/words.rs:101); rw-4 a per-run timeout in under-load.sh.
Fix the cause, not the wait: no longer sleeps, no retries that hide a real bug. An item without a reliable
reproducer after 45 min: report the evidence and move on.
Done when: each item has red and fix shas or written evidence; each fixed step 10/10 under 4 busy loops;
`tests/merge-gate.sh --jobs 3` full run green on the head (poll in the same turn).
Report: reports/flakes-g.md on claude/session-log (table as reports/flakes-f.md). Then send_message to the PM
session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines). Poll every command in the same turn; never end a
turn to wait.
Rules: push claude/flakes-g and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
