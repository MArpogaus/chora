# kit-minor-r1 (F2 kit quality, known issues): Book origin labels and the kit7 r2 helper MINORs

Model: sonnet, effort medium (small kit fixes with cells)
Lane: cloud, branch claude/kit-minor at e7f2410a (session fee9c695 plus mc-fix 929de144). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "kit-minor r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/mc-fix.md "## PM review r6" (pr6-1), reports/kit.md or reports/kit7.md (r2 review), backlog.md line "kit7 r2 MINOR/nit". Not yours: the looks' plugins beyond a call-site rename, the core, the gate script.
Items (each: red cell first, then the fix):
1. pr6-1: crates/chora-kit/src/wm/book.rs Origin: the labels "given" and "new" must mean what docs/api.md:1619 and :1655 say (given = another window manager gave it). Find which side is wrong (code, Book's log lines or docs) with a cell that adopts a handed window and reads its origin; fix that side only.
2. helper.rs:195 answered -> working has no cell: add one.
3. helper.rs:205 a new utterance does not clear answered: cell, then fix.
4. helper.rs:142 seed() keeps report order, not alphabetical: cell, then fix.
5. docs/api.md:1573 rewrap (no word change).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,mc-paths,mc-handover,wm-pane,acts,doors`; poll it in the same turn; never end a turn to wait.
Done when: commits pushed to claude/kit-minor; the gate list passes; reports/kit-minor.md "## r1" on claude/session-log names per item the red and fix shas and the cell.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
