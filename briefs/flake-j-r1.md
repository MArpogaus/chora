# flake-j r1 (F5): open flake F29, tiling-audit fullscreen bar

Model: sonnet, effort high (routine flake fix, reproduce then fix)
Lane: cloud, branch claude/flake-j from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-j r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md row F29.
Do not touch the cells of the running lanes flake-h (layout HUD audit, doors helper, construct pull pane) and flake-i (log_file.rs, tests/*.sh cleanup helper).
1. F29: the tiling-audit fullscreen act fails about 2 of 30 trains ("bar shows over the window at x 66", "left island ends at 481"): the act checks before the bar is hidden and the window has its fullscreen size. Reproduce first: the fullscreen cells of tiling-audit 30 times under load (--jobs 3 or a parallel cargo build beside it); name the count. Fix: the act waits (bounded, until_ok of the gate lib) for the bar hidden and the window at the output size before the capture; no retry of the whole cell, no longer sleep. If the core shows the bar over a fullscreen window for real (not timing), stop and report it as a finding with the capture.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,tiling-audit` green, then tiling-audit twice more. Poll it in the same turn; never end a turn to wait.
Done when: F29 closed with reproduce counts before and after (0 of 30 after), or named a real core finding; gate green; reports/flake-j.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-j and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
