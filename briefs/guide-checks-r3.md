# guide-checks r3 (F7, RC 11): close review r2 (rg-9 MAJOR, rg-10, rg-11)

Model: sonnet, effort medium (three contained harness fixes with red tests)
Lane: cloud, claude/guide-checks at 4efbf430 (review r2 FIX). Cost at most 0.6 %; stop at 200 k context.
Why: reports/guide-checks.md "## Review r2" on claude/session-log:
- rg-9 MAJOR known-red.toml:221,536,554,572,590,743,1454,1472,1643: `flaky = true` on 9 rows (report said 2), act dropped, so each covers that tray contrast in any act; no run evidence; likely the tray clock redraws at the minute change during the capture.
- rg-10 MINOR invariants.py:503: "no font size, the guide wants 14 px" contains the match of the 135 size rows, so those rows hide a harness gap.
- rg-11 MINOR invariants.py:240: `mode` in KEY_LINE gives 7:1 to any `-mode-` label (xp-mode-label fails at 4.54:1).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "guide-checks r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
1) Red first (own commit): unit tests in tests/test_acts.py: a new act's 1.0:1 tray breach is not covered by a flaky row; a "no font size" breach is not matched by a size row; a `-mode-` label that is not a key line gets 4.5:1.
2) Fix rg-9: drop `flaky`; make the capture wait for a settled clock (e.g. capture after a minute tick or read the clock node twice); every row keeps its act. If a fault stays truly intermittent, keep `flaky` only with an `evidence` field naming two runs (one with, one without) and the act; judge() refuses flaky rows without evidence.
3) Fix rg-10: give the "no font size" breach its own wording; known-red rows for it only where the node really has no size, each with its act.
4) Fix rg-11: key lines only by node meta or an anchored label list; no substring `mode`.
5) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,gate-lib,hud,layout,tiling-audit,acts`, plus `python3 -I -m pytest tests/test_acts.py -q`. A red cell you did not cause: name it with its log line (FLK-* in backlog.md), do not rerun to green. Poll in the same turn; never end a turn to wait.
Report: append "## r3" to reports/guide-checks.md on claude/session-log (shas, per rg red and fix, row counts before and after); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/guide-checks and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
