# ledger-audit r1 (rc 1: no open BLOCK/MAJOR; ledger truth, no code)

Model: sonnet, effort medium (reading the ledger against gate cells and tests; no product change)
Lane: cloud, claude/ledger-audit at ce866b0d (session head, t124). Cost at most 0.5 %; stop at 150 k context. One area: findings/*.md on claude/session-log, read only; no product code, no commit on claude/ledger-audit.
Why: findings/ lists BLOCK/MAJOR rows whose state is not fixed or closed although their text names a fix or a merge: co-48, co-49, co-50, co-51, co-52, co-54 (construct), mc-45 (mc), ti-20, ti-32, ti-38, ti-47, ti-48 (tiling), sh-23, sh-38 (shared). RC 1 needs a true count. co-67 (co-reach r4 runs), ti-47, ti-48 (tiling-reach runs), sh-38 (flake-series runs): only state the row, do not test.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "ledger-audit BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Per row: the fix sha it names; is that sha an ancestor of ce866b0d (`git merge-base --is-ancestor`)? The test or gate cell that guards it (file:line or tests/gate-*.sh step); run that test or step once on ce866b0d (cargo test -p <crate> <name>, or the gate script alone) and paste the pass/fail line.
2) Verdict per row, computed: fix in the head and its guard passes -> "fixed <sha> (guard <cell>, ledger-audit r1)"; no guard -> "open: no guard" (a MAJOR stays open); guard fails -> "open: red <line>"; "not a bug" rows: the deciding line.
3) No severity changes and no new findings beyond one line each under "## Seen" in the report.
Gate: none (no code); run each test in the foreground, never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: one line per row above with sha, ancestor yes/no, guard, pasted pass/fail line and the computed state.
Report: reports/ledger-audit.md "## r1" on claude/session-log (a table: id, fix sha, in head, guard, line, proposed state); the PM edits findings/. send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines: count open BLOCK/MAJOR before and after.
Rules: push claude/session-log only, your report only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
