# handover-mo r2 (rc 2, shared RC bar "a visible way to start"): review FIX rhm-1 MAJOR, bare chora-tiling paints nothing

Model: sonnet, effort medium (one look's start path plus a gate cell; the cause is known from review r1)
Lane: cloud, claude/handover-mo at f7dded7a. Cost at most 0.6 %; stop at 250 k context. One area: the tiling start path. First merge claude/ti-29 at 1f9a5180 (MERGE-OK, also edits plugins/chora-tiling/src/main.rs; resolve conflicts keeping both).
Findings (reports/handover-mo.md "## Review r1" on claude/session-log):
- rhm-1 MAJOR README.md:31, plugins/chora-tiling/src/main.rs:390: the documented start `chora -- chora-tiling` (and bare `chora-tiling --output` beside xp) gives a black screen, no bar, no start door. Fix (POLS): chora-tiling starts its companions by default (the pid-file guard `companion()` stops doubles); keep `--with-companions` accepted as a no-op for one release, add `--no-companions` only if a test needs it; README and docs/known-issues.md:119 updated; findings sh-23 render half and sd-h1 rows updated with the sha.
- rhm-2 nit tests/gate-start-tap.sh:58,111,127: `menu_after closed` must poll the whole window (for example 1 s) instead of returning at the first 0.2 s poll.
- rhm-3 nit tests/gate-start-tap.sh:136: replace `sleep 0.5` with a poll of the chosen manager.
Known, not in scope: rhm-4, rhm-5 (backlog).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "handover-mo BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first: a gate cell (tests/gate-handover-matrix.sh or gate-doors.sh) that runs bare `chora-tiling` under headless chora with no pre-started companions and asserts the bar and start door surfaces exist within 5 s; commit it alone, red on f7dded7a.
2) Smallest fix, KISS, no new dependency; then the cell green. No new lint allow (tools/lint-ratchet.list must not grow).
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,xp,matrix,virtual,doors,acts,tiling-audit,tiling-look,launcher-click` green. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: rhm-1 fixed (red cell commit, then green), rhm-2 and rhm-3 fixed, the gate green, finding rows updated on claude/session-log.
Report: reports/handover-mo.md "## r2" on claude/session-log (shas, cause with file:line, cell per row, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/handover-mo and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
