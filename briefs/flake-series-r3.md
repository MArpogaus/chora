# flake-series r3 (F5): five full gates in a row at --jobs 2 on train 92c, and the chora-kit coverage figure

Model: haiku, effort high (runs and counts tests; no production code)
Lane: cloud, branch claude/flake-series-r3 at dcb77733 (train 92c head: flakes-l, flakes-n, flakes-o merged). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3 ffmpeg`, then `git fetch --unshallow` and `cargo check --workspace`; no setup.sh (auto mode refuses it in lanes); a missing tool or failure: send_message the PM "flake-series BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/foundation.md row F5 (the bar: 5 full gate runs green in a row at --jobs 2), reports/flake-series.md r2, backlog.md FLK-LAUNCHER and FLK-HUD-TAB.
Runs take long: start each gate in the background and poll it in the same turn; never end a turn to wait.
1. Run `tests/merge-gate.sh --jobs 2` (no --push) five times in a row on dcb77733. Keep every log. For each run: exit code, the failing steps and the first failing line.
2. Any red step: rerun that step alone (`--only <step>`) ten times; report count red of ten and the cause line. Also run `--only launcher-click` and `--only construct-hud` ten times each (FLK-LAUNCHER, FLK-HUD-TAB). Do not fix, do not change a timeout, do not mark a test ignored or known-red.
3. Coverage: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features testing --summary-only`; the lines figure (F5 needs 80 %) and the five files with the most uncovered lines.
Done when: reports/flake-series.md "## r3" on claude/session-log with a table run 1-5 (exit, red steps), the reruns of step 2, the coverage figure and file list; no commit on the code branch.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "flake-series DONE: <greens>/5, kit <n> %" plus the red steps (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
