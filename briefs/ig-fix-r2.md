# ig-fix r2 (F6): ig-fix on top of train 74 (gate-pick)

Model: sonnet, effort low (a merge and one usage line)
Lane: cloud, branch claude/ig-fix (head 3a39659c). One round, stop at 250 k. Cost about 0.2 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "ig-fix r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/ig-fix.md "PM review r1" (igr-1, igr-2).
1. igr-1: `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it (it holds gate-pick: `&& tests/cells-for-selftest.sh` on the gate-lib line). Resolve so the gate-lib step runs gate-lib-selftest, under-load-selftest and cells-for-selftest beside merge-gate-selftest, every exit code checked, the nested guard kept. Extend your selftest check so it also greps cells-for-selftest.sh on the gate-lib line (red first: commit the check before the resolution if it fails on the merge).
2. igr-2: name MERGE_GATE_NESTED in the env list of the usage text of merge-gate.sh.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib` green. Poll it in the same turn; never end a turn to wait.
Done when: igr-1, igr-2 closed; gate green; reports/ig-fix.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/ig-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
