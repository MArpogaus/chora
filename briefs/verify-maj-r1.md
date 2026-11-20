# verify-maj-r1 (RC 1 and rule 6: three open MAJORs marked verify or stale block MINOR work on tiling, construct, xp): measure them on the current tree, fix nothing

Model: haiku, effort high (runs existing gates and probes and writes down what it saw; no code change)
Lane: cloud, branch claude/verify-maj from 88258c87 (session branch 79c8d2b5 plus launcher-click 741d6bca, train 82c). One round, stop at 250 k. Cost about 0.3 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace`; a failure: send_message the PM "verify-maj BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/tiling.md (ti-20), findings/construct.md (co-26), findings/xp.md (xp-24), reports/xp-polish-r1.md and reports/review-xp-polish-r1.md (what was done on xp-24). On the branch: tests/merge-gate.sh (cell names), tests/gate-tiling-apps-click.sh, tests/gate-construct-keys*.sh (whatever exists), tests/gate-xp-fields.sh.
Scope: no change under crates/, plugins/, services/, flavors/ or tests/. You write only reports/verify-maj.md and the three ledger rows on claude/session-log.
Work, per finding:
1. ti-20 (Apps click in the tiling bar shows no launcher within 1.5 s): run tests/gate-tiling-apps-click.sh 5 times headless; per run pass or fail with the last 10 lines.
2. co-26 (Super+F1 in a construct face shows no key list): run the construct key-list gate that exists (name it); if none covers a face, say so and run the closest cell 3 times.
3. xp-24 (text fields off the XP grid): run gate-xp-fields.sh 3 times; list the text fields of xp that the gate does not measure (Run box, search, dialogs) by reading the xp plugin and the flavor config, with file:line.
Verdict per finding, one of: reproduced (with run output), not reproduced N/N (with run output), partly (what is measured, what is not). Do not change a ledger state to fixed: write "verify-maj r1: <verdict>, <date>" into the row's evidence column only; the PM decides the state.
Done when: the three verdicts with output are in reports/verify-maj.md "## r1" on claude/session-log and the ledger rows carry them.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
