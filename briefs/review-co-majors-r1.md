# review co-majors r1 (RC, construct): seeded review of claude/co-majors before its merge

Model: sonnet, effort high (one plugin, 244 lines; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/co-majors head db4d703b, base 30727c82 (9 files, +244 -9). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 12 cells green).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`; `git fetch origin claude/session-log claude/co-majors`; `cargo check --workspace`; a failure: send_message the PM "review co-majors BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read findings/construct.md rows co-19..co-21 and reports/co-majors.md "## r1" on claude/session-log (captures in reports/co-majors/); then `git diff 30727c82..db4d703b`.
Checks, each with its command and files:
1) Red first: 85691f86 alone fails pull 2a and db4d703b passes (`git log --oneline 30727c82..db4d703b`; run `tests/gate-construct-pull.sh` on each in a worktree, BIN=$PWD/target/debug after `cargo build --workspace`).
2) The red check is a real closed-cube check, not tuned to this one scene: closed_cube_breaches in tests/gate-construct-pull.sh (shared edge 1 px, length 2 %, mirror under face, no pane over a face); would it catch a gap or overlap between faces?
3) plugins/chora-construct/src/cube/aside.rs: the aside place (depth of the cube, side, half diagonal + 0.15 w + half pane) stays on screen at 640x720 and with two or more placed panes; panes do not overlap each other or a face.
4) Home: step in and close restore the exact rect; a pane moved by the person during stand-back stays put (POLS); no window lost (D-handover rule); unit tests cover these or say which are missing.
5) Only construct changed: frame.rs, modes.rs, panes.rs, main.rs hunks; nothing in core or other looks; no new public API without a doc line.
6) docs/cube-modes.md matches the behaviour; captures before/after show the fix (look at the two co-21 PNGs).
7) co-19/co-20: 929b64e7 is in 30727c82 (`git merge-base --is-ancestor 929b64e7 30727c82`) and cell `out` passes on db4d703b.
8) `cargo test -p chora-construct` and `cargo clippy -p chora-construct --all-targets -- -D warnings`.
Known, not findings: the other open construct MINORs in findings/construct.md; tour v8 baseline (next week).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/co-majors.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
