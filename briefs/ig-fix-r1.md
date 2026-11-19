# ig-fix r1 (F6): id-gate review findings ig-1 and ig-2

Model: sonnet, effort medium (routine harness fix)
Lane: cloud, branch claude/ig-fix from the id-gate head a9fd91f3. One round, stop at 250 k. Cost about 0.3 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "ig-fix r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/id-gate.md "## r1 PM review"; backlog.md row HARNESS ig-1.
1. ig-1: tests/merge-gate-selftest.sh runs in no gate step or CI job. Call it from the `gate-lib` step of tests/merge-gate.sh (beside gate-lib-selftest.sh), so every gate runs it. Red first: a selftest case (or a check in gate-lib-selftest.sh) that fails while the gate-lib command lacks merge-gate-selftest.sh.
2. ig-2: when MERGE_GATE_IDENTITY is set, the identity step prints one line "identity: MERGE_GATE_IDENTITY overrides the expected identity: <value>"; a selftest case checks the line.
No other change. Keep the gate-lib step under 2 minutes (name its time before and after).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib` green. Poll it in the same turn; never end a turn to wait.
Done when: both closed; gate green; reports/ig-fix.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/ig-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
