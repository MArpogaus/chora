# review co-majors r2 (RC, construct): seeded review of claude/co-majors r2 before its merge

Model: sonnet, effort high (one plugin, a fix round of review r1; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/co-majors head 8e24dcb6 (8e24dcb6 merges the session branch bb16f2ff); r2 range db4d703b..c5b143b1 (4 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 12 cells green).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`; `git fetch origin claude/session-log claude/co-majors`; `cargo check --workspace`; a failure: send_message the PM "review co-majors r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read reports/co-majors.md "## Review r1" (rv-1..rv-4) and "## r2" on claude/session-log; then `git diff db4d703b..c5b143b1` and `git diff bb16f2ff..8e24dcb6 --stat` (only construct files and tests/gate-construct-pull.sh may differ).
Checks, each with its command and files:
1) Red first: d155d807 and 1ed59e8e fail on db4d703b's code (gate case 2c: pane-s1/pane-s2 overlap; 4 unit tests) and pass at 3afe9285 (`cargo test -p chora-construct aside` on each in a worktree).
2) rv-1: plugins/chora-construct/src/cube/aside.rs `make_way`: running offset per side, nearest pane first; with 3 panes on one side, or 2 wide panes, no pane overlaps another or a face; the "other side" fallback cannot put a pane over a pane already placed there (read the loop; add a scratch unit test, do not commit it).
3) rv-1 shrink path: a pane that fits on neither side shrinks its offset for at most 20 steps; can it end over a face or another pane? Is the result deterministic (no order from a HashMap)?
4) rv-4: the place is clamped to `radius()` as in `move_pane_facing`, and stays on screen at 640x720 and 1280x800 (unit tests the_place_beside_the_cube_is_inside_the_leash, a_wide_pane_beside_the_cube_stays_on_the_screen: do they test the screen bound or only the leash?).
5) rv-2: the six named tests assert exact home rects (not approximate) and the person-moved pane stays put (POLS); gate 2a step-in compare in c5b143b1 compares the home rect exactly.
6) rv-3: closed_cube_breaches in tests/gate-construct-pull.sh now checks mirror y and height, the 1 px edge length, pane against pane; would it catch a 2 px gap between two faces?
7) docs/cube-modes.md describes the per-side order and the other-side fallback.
8) `cargo test -p chora-construct` and `cargo clippy -p chora-construct --all-targets -- -D warnings`.
Known, not findings: the other open construct MINORs in findings/construct.md; the 640x720 gate case not run (say if you see a reason it fails); tour v8 baseline (next week).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv2-N, severity, file:line, text, fix in one sentence). Append to reports/co-majors.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
