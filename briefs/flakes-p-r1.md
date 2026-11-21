# flakes-p r1 (F5: no flaky tests): FLK-PULL

Model: sonnet, effort high (timing faults under load; each needs a cause with evidence, not a retry)
Lane: cloud, branch claude/flakes-p from 618ae513 (session head, train 94b). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-p BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md rows FLK-PULL; reports/flake-series.md "## r3"; reports/flakes-o.md and reports/flakes-l.md for the method. Not yours: the cells of the other flake lane (flakes-p: pull; flakes-q: construct-hud, xp-faces).
Items, each: reproduce under load (`--jobs 3` plus a second `cargo build` in a loop), cause with log evidence, red first, fix (a wait on the real state, never a sleep or a retry), then 20 loaded runs of the cell green; a row not reproduced in 20 loaded runs says so with the count and keeps the added failure log (commit it, it is the evidence for the next red):
1. FLK-PULL: tests/gate-construct-pull.sh (cell `pull`) "the second window is no pane": 1 of 10 alone on dcb77733. First log the pane tree and the second window's node (id, owner, role) when the check fails, so a red names its cause.
2. If done below 150 k: rerun the `placed` and `cube` cells 10 times loaded and report reds (evidence only).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,<your cells>`, then the touched cells twice more at --jobs 3; poll it in the same turn; never end a turn to wait.
Done when: per item red sha, fix sha, loaded-run count; backlog rows closed or named; reports/flakes-p.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-p and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
