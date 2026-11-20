# flakes-n r1 (F5: no flaky tests; the two unowned rows): rw-3 tiling panel unit test, MX-CAP matrix cap

Model: sonnet, effort high (timing faults; each needs a cause with evidence, not a retry)
Lane: cloud, branch claude/flakes-n from f5ac3500 (probe-leak, MERGE-OK, on top of session head d81c8293; MX-CAP must be measured without leaked probes). One round, stop at 250 k. Cost about 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-n BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/audit-f.md "## r4" (F5 section and slice 2); reports/co-p2-fix.md rw-3 (line ~74); backlog.md MX-CAP (~316). Not yours: tests/gate-tiling-look.sh and the flakes-l rows (lane flakes-l), cube, doors (lane flakes-m), lint lists (lint-down).
Items:
1. rw-3: `cargo test -p chora-tiling` 20 times under load (a second `cargo build` loop beside it); count reds of `panel::the_layout_word_is_a_word_of_a_person`. Red: name the order or timing dependency with evidence, red-first commit, fix (no sleep, no retry, no ignore), 0 of 20 after. Not reproduced in 20: say so with the count, read the test for shared state and name any you find.
2. MX-CAP: 5 timed runs of the handover-matrix cell at --jobs 2 (`tests/merge-gate.sh --jobs 2 --only build,matrix,construct-extra` or the cell alone, say which); cap = slowest + 25 % per the comment in tests/gate-handover-matrix.sh; red-first selftest that the cap follows that rule (or a check that the comment and the value agree), then the value.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix`; poll it in the same turn; never end a turn to wait.
Done when: per item red sha, fix sha, run counts; backlog MX-CAP closed or named; reports/flakes-n.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-n and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
