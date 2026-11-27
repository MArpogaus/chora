# flake-gate r1 (rc 3, F5: the non-launcher flakes sh-41, sh-43, sh-45, sh-46)

Model: sonnet, effort high (routine flake fixes in gate cells and session start)
Red on head: no deterministic red yet; sightings: sh-41 tiling-bar "2 waybar processes on one config", sh-43 gate-tiling-notice.sh:31 "choractl: socket: No such file or directory", sh-45 xp-notepad "the client typed ':end' and not start::end", sh-46 acts tiling I14 "chora-tiling.launch disconnected" (findings/shared.md). All green on rerun. Step 1 makes them red.
Lane: cloud, branch claude/flake-gate-r1 at 0e18f14c34d95e7575859c6c731cd6e53e19ac9f (session head, t157). Area: the gate cells tiling-bar, tiling-notice (tiling-system), xp-notepad, acts and the session start path they share (socket readiness, look launch). Not the launcher start path (flake-launcher-r1 owns it). Stop at 250 k. Cost at most 0.7 %.
Why: RC 3 needs 3 greens with no flake; reports/flake-series.md "## r10".
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "flake-gate-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Reproduce: run each cell's own tests/gate-*.sh script (as merge-gate.sh calls it) 10 times beside a `cargo build --workspace` load or bash busy loops; count misses; keep the first failing log; read the session log at the miss.
2) Root cause per sighting; fix it, not the timeout. A longer wait only with a readiness signal, when the log proves the step happens and is merely slow.
3) Red test commit first per fix (a cell or unit test that fails on 0e18f14c without it), then the fix. Never skip or quarantine a cell.
4) After: each cell 10 times under the same load: 0 misses; then the full gate once (`tests/merge-gate.sh --jobs 2`, no step filter).
Never end a turn to wait; poll any long command in the same turn. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: miss counts before and after per cell (with logs), root cause per sighting in one paragraph with the log line, red commit before each fix, full gate green. A sighting with 0 misses in 10 before: say so; no fix without a red.
Report: reports/flake-gate.md "## r1" on claude/session-log (logs under media/2026-11-26/flake-gate-r1/, each under 500 KB); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/flake-gate-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
