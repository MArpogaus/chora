# flakes-g r3 (F5): rerun of r1 (r1, r2 containers had no setup; 14:00 default: the lane runs setup.sh)

Model: opus, effort high, because the causes are timing races across core round trips, gate scripts and load
(hard causes); each red train costs about 55 min of the one serial merge gate
Lane: cloud, branch claude/flakes-g from c227d433 (r1: rw-4 red 8d555fc5 fix f0c9e640; rw-1 c227d433 done). One round, stop at 250 k.
Cost about 3.5 %. Run `git fetch --unshallow` first; merge origin/claude/autonomous-dev-session-3w-i33pg7 at start
and again before the gate (resolve to the session side's types).
Step 0: setup (below), then `cargo check -p chora` and `command -v foot fuzzel waybar`; if either still fails, send_message the PM "flakes-g r3 BLOCKED: <setup's MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh` (up to 25 min).
Open from r1: items 1, 2, 3, 5 and rw-2; rw-1 and rw-4 are done (append to reports/flakes-g.md).
Seeds (read these, not the whole repo): reports/flakes-f.md ("Found, not fixed" and "Review r1"), backlog.md
F26 (line 164), F28 (173), FF1 (last line) on claude/session-log. Load: tests/under-load.sh (4 busy loops).
1. Do item 1 last. Lane cubesplit moves `open_cube` (impl Construct) into plugins/chora-construct/src/cube/construct.rs;
   before item 1 merge origin/claude/autonomous-dev-session-3w-i33pg7 and, if cubesplit is not on it yet,
   origin/claude/cubesplit (a pure move), and edit the moved code. cube.open over 300 ms under load (tests/gate-construct-cube.sh:656; plugins/chora-construct/src/cube.rs:1142
   `open_cube`, about 8 sequential round trips): red cell under load, then fewer round trips (pipeline the
   four output.add or one call with a count); a kit Conn API change stays small and documented in docs/api.md.
2. F26 doors "missing" (lead: train 22 dump, a click lands on a hit node over mc's surface s1) and
3. F28 chat-keys "text typed before the pick did not reach the picked helper": for each a cause with file:line
   and a reproducer that fails reliably under load, red first, then the fix of the cause.
4. FF1: rw-1 gate-tiling-audit.sh:86 wait in seconds, not iterations; rw-2 a test of the write_all short-write
   path (crates/chora/src/words.rs:101); rw-4 a per-run timeout in under-load.sh.
5. Pull flake (backlog, wm5 r3 13:32): tests/gate-construct-pull.sh:163 pane 169 px for 200 under load, passes alone:
   cause with file:line, red under load, fix.
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
