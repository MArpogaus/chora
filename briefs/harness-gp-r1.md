# harness-gp r1 (F6): gate-pick MINORs gp-1..3

Model: sonnet, effort medium (small harness fixes with selftests)
Lane: cloud, branch claude/harness-gp from the session branch a9e7084d (train 74). One round, stop at 250 k. Cost about 0.4 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "harness-gp r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/gate-pick.md (gp-1..3), backlog.md row "HARNESS gp-1..3". Tool: tests/cells-for.sh and tests/cells-for-selftest.sh.
Each fix: its red selftest case commit first, then the fix.
1. gp-1: without `--head`, cells-for also sees untracked files (`git ls-files --others --exclude-standard`); a selftest case: a new untracked tests/gate-x.sh picks its cell.
2. gp-2: a change to tests/merge-gate.sh (or tests/gate-lib*) picks gate-lib too; a selftest case.
3. gp-3: CONTRIBUTING.md says in one or two lines that the pick is advisory (a plugin started only through the session config is not picked) and the train runs the full gate.
4. Before the final gate: `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it (TRAIN75: catch conflicts with lanes merged meanwhile).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib` green. Poll it in the same turn; never end a turn to wait.
Done when: gp-1..3 closed with red first; gate green; reports/harness-gp.md "## r1" on claude/session-log naming each finding, its test and the head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/harness-gp and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
