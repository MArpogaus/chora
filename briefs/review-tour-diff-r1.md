# review tour-diff r1 (F7, RC 11): seeded review of claude/tour-diff before its merge

Model: sonnet, effort medium (a Python diff tool and checker; seeded review)
Lane: cloud, read-only on code: claude/tour-diff head fa796bb9, base 8b996cee (5 files, +569). One round.
Stop at 100 k context; cost at most 0.3 %; at most 8 findings. No merge gate run (the lane ran pre-commit, gate-lib, ci-check, build, test, acts green); `python3 -m pytest` on the new tests only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-tour-diff r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tour-diff-r1.md (done criteria), design/tour-v9.md sections 6 and 8 row 2, reports/tour-diff.md "## r1" on claude/session-log; `git diff 8b996cee..fa796bb9`.
Checks, each with its command and files:
1) tests/tour-diff.py: per act and capture both a scene diff and a pixel diff; kinds as tour-v9 section 6; a change with a row in demo/tour/intended.toml reads as intended, one without as regression; same vs same is same. Reuses tests/capture-diff.py (no copy: `tools/cut-check --base 8b996cee`).
2) tests/tour-video-check.py: black frames, flash, freeze, screen-fixed node moving in a camera move; thresholds named and justified; ffmpeg missing gives a clear error, not a pass.
3) Planted cases (moved node, changed blob, flicker, freeze, black stretch, fixed node): each test fails without the code (red a58a39f6 before fa796bb9, check on the parent) and would catch a real v8 regression, not only the plant.
4) The gate-acts hook: what it runs, that it cannot turn a missing capture into a pass.
5) Report truth: the v8 runs (02-1-tile_left vs tile_bottom; 01-xp, 04-construct with 21 and 7 freezes) reproduce from the report's commands.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rd-N, severity, file:line, text, fix in one sentence). Append to reports/tour-diff.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
