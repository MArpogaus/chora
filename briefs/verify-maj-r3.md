# verify-maj r3 (rc 1, 9): close or confirm the open MAJOR verify lines co-36, ti-20, mc-47 and A13 (window loss)

Model: sonnet, effort medium (verification runs with existing cells; no product code)
Lane: cloud, claude/verify-maj3 at 1c599c74. Cost at most 1 %; stop at 250 k context. One area: verification of findings (tests only, no product code).
Why: plans/week-3.md RC row 1 and 9 (claude/session-log): three MAJOR verify lines are open in the findings ledgers (co-36 construct, ti-20 tiling, mc-47 mc) and A13 (mc drag onto construct loses panes) is not verified. reports/verify-maj.md r1, r2 and the PM review r2 hold the earlier state (ti-20 verified on 11-20 at 923 ms; check that the ledger says so).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "verify-maj BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Per line: find it in the ledgers (backlog.md, reports/*, design/findings*) and the cell that asserts it; run the cell 5 times on 1c599c74 (`tests/merge-gate.sh --jobs 1 --only build,<cell>`). Green 5 of 5 with the assert present: closed. Red: confirmed, with the first failing line. No assert: write the assert as a test commit (red first if the bug is present), no product change.
2) A13: write or find the act that drags mc panes onto construct and counts windows before and after; run it 5 times.
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,<the cells you touched>` green (no product code). A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: each of the four lines closed or confirmed with a cell and counts.
Report: reports/verify-maj.md "## r3" on claude/session-log (line, cell, count of 5, verdict, sha); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/verify-maj3 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
