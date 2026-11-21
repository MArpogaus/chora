# gate-waits r1 (F5, rc 3): fixed sleeps in the flaky gate scripts become waits on a state

Model: sonnet, effort medium (mechanical test-script changes with one helper; no product code)
Lane: cloud, claude/gate-waits at 1c599c74. Cost at most 1.6 %; stop at 250 k context. One area: the gate scripts of the cells that flaked this week: doors, xp-faces, construct-hud, layout, pull, tiling-look (tests/*.sh).
Why: plans/week-3.md slice 6 (claude/session-log): F5 (no flaky cell) is open; t99c and t99d went red at doors, and reports/flake-series.md names the cells above. A fixed `sleep N` before an assert races with a loaded machine.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "gate-waits BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Count first: `grep -c sleep tests/*.sh` total (673 at plan time); name it in the report.
2) In those six scripts, each fixed sleep that waits for a state (a window mapped, a layer shown, a log line, a capture ready) becomes an until_ok wait on that state with a timeout (use the existing helper in tests/gate-lib.sh; add one only if none exists, with a selftest). Keep sleeps that pace input on purpose and say why in one comment. KISS: no new framework.
3) Load test: each of the six cells 10 times at `--jobs 2` with a CPU load beside it (`stress-ng` if present, else a `yes > /dev/null` per core); count greens. Before (on 1c599c74) and after.
4) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the branch merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: the sleep count falls, each of the six cells 10 of 10 at --jobs 2 loaded after the change (before counts named), full gate green.
Report: reports/gate-waits.md "## r1" on claude/session-log (shas, sleep counts before and after, per cell green of 10 before and after, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/gate-waits and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
