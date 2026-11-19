# flake-h r2 (F5): prove the load fixes under load

Model: sonnet, effort medium (routine test runs and a comment)
Lane: cloud, branch claude/flake-h at bc3f1662. One round, stop at 250 k. Cost about 0.5 %.
Step 0: run setup once: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-h r2 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore.
Read on claude/session-log: reports/flake-h.md "## r1" and "## PM review r1".
1. rw-1: `tests/under-load.sh --runs 5 --cpus 2 -- tests/merge-gate.sh --jobs 3 --only matrix,layout,doors,construct-pull` on bc3f1662 (the load of the r1 reds). Each red cell: find the race, fix the test wait (a red is the red test; commit the fix after), then 5 more loaded runs.
2. rw-2: a comment at `until_ok` naming the unit (second argument: rounds of 0.5 s, waited in wall-clock seconds).
3. rw-3: no change (backlog).
Poll each run in the same turn; never end a turn to wait.
Done when: 5 of 5 green under `under-load.sh --cpus 2` on the final head; reports/flake-h.md "## r2" on claude/session-log with each run line, any red cell, cause and fix commit; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-h and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
