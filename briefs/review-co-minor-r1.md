# review co-minor r1 (rc 1b construct; rpc2-1, rpc2-2, rcrr6-1): seeded review of claude/co-minor-r1 before its merge

Model: sonnet, effort high (routine: two MINOR hint/sheet rules and a docs line, 268 lines)
Red on head: none (a review; the lane's red 3d312128 precedes its fix dc532d60)
Lane: cloud, read-only on code: claude/co-minor-r1 at dc532d60, review 57ce1480..dc532d60 (9 files, +268 -27: chora-construct command.rs, config.rs, keys.rs, main.rs, overlay.rs, reach_tests.rs; tests/gate-construct-cube.sh; docs). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/co-minor-r1.md (done criteria), reports/co-minor.md "## r2" (this lane's report), reports/rc1-pass.md "## r2 construct" (rpc2-1, rpc2-2), reports/co-reach.md "## Review r6" (rcrr6-1).
Checks, each with its command and files (no merge gate; `cargo test -p chora-construct`, `cargo clippy -p chora-construct`, `tests/gate-construct-cube.sh` once on head):
1) Red first: gate-construct-cube.sh step 9 and the unit tests of 3d312128 fail on 57ce1480 (run there) and pass on dc532d60; else MAJOR.
2) rpc2-1: the key sheet in a pane lists only keys that reach the window; the sheet and the hint bar read one "keys that work now" source (KISS), not a special case per path.
3) rpc2-2: after the sheet in a pane, Escape, step back: the hints match those without the sheet (Super Alt 2 Into face).
4) POLS: no other path's sheet or hints lose a key a person relies on (face path, co-79 r6 steps 7, 8 green; the walk holds at most six hints).
5) rcrr6-1: docs/configuration.md and docs/cube-modes.md say what the code does (six hints, Super C Cube with its condition).
6) The lane's note: docs and ci-check failed only for missing mkdocs and Pillow; confirm by reading the step scripts, no code path of this diff.
Known, not findings: rpc2-3..6, rcrr6-2..6, co-44, co-79 (next lane), sh-37..sh-47.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcmr1-N, severity, file:line, text, fix in one sentence). Append to reports/co-minor.md under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
