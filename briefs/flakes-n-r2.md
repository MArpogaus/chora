# flakes-n r2 (F5): MX-CAP cap from full-train load (PM review r1 pr1-1 MAJOR)

Model: sonnet, effort medium (one number from given data plus one confirming run)
Lane: cloud, branch claude/flakes-n from fe398489 (your r1 head). One round, stop at 250 k. Cost about 0.3 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-n BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/flakes-n.md "## r1" and "## PM review r1". Not yours: everything outside tests/gate-handover-matrix.sh, tests/matrix-cap-selftest.sh and their comment.
Item: pr1-1. The matrix own run time in real trains (PM host, 86 logs) reached 283, 290, 300, 301, 309 s. Set the five runs of the comment to these and the cap to 309 s + 25 % rounded up (387 s); comment, usage line and default together; the selftest stays and passes. Then one run of `tests/merge-gate.sh --jobs 2 --only build,matrix,layout,construct-extra,doors` and name its matrix run time (it must be below 387 s). Red-first: a commit where the selftest fails against the new comment, then the value.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix`; poll it in the same turn; never end a turn to wait.
Done when: red sha, fix sha, the confirming run time; reports/flakes-n.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-n and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
