# ledger audit r1 (rc 1-2): every open BLOCK and MAJOR row checked against the session head

Model: sonnet, effort medium (read and run existing gate cells; no product code; ledger rows are stale: sh-24 open though stray-grab merged a5184639)
Lane: cloud, claude/autonomous-dev-session-3w-i33pg7 at 598a34ff; writes only findings/*.md and reports/ledger-audit.md on claude/session-log. One round.
Stop at 150 k context; cost at most 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "ledger-audit BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Rows: every row of findings/*.md on claude/session-log with severity BLOCK or MAJOR and a state that is not fixed-and-merged (open, in flight, fix pending, fixed on a lane branch). Skip rows a running lane owns: sh-23, sd-h1, sd-h2, ca-4 (handover-mo), co-38, co-40, co-41 (co-keys), FLK-* (flakes-r).
Per row:
1) Find the fixing merge: `git log --oneline --merges 598a34ff | grep <lane>` and the row's named branch or sha (`git merge-base --is-ancestor <sha> 598a34ff`).
2) Run the row's cell (named in the row or its report; e.g. `tests/merge-gate.sh --jobs 1 --only <cell>` or the act or test it names) once on 598a34ff. Green and merged: state "fixed <sha> merged <merge sha>, cell <name> green on 598a34ff (11-21)". Red: "open, cell <name> red on 598a34ff (11-21): <log line>". No cell exists: "open, no cell (11-21)"; write none.
3) Never change a severity or close a row without a green cell on 598a34ff; never edit product code or tests.
Gate: none (read-only on code). Poll in the same turn; never end a turn to wait.
Done when: every row in scope has a dated state; reports/ledger-audit.md "## r1" has one line per row (id, old state, new state, cell, evidence) and the counts open BLOCK and open MAJOR per look.
Report: send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines: counts per look, rows still open with no cell.
Rules: push only claude/session-log (fetch and merge on reject, no force); author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
