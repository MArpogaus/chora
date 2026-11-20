# gate-silent r1 (F5, F6 evidence; harness): a cell that does not run shows SKIP; fuzzel version guard; MC-47-CELL back

Model: sonnet, effort high (harness change in tests/merge-gate.sh; it must never block a valid run nor pass a silent one)
Lane: cloud, branch claude/gate-silent from f5ac3500 (probe-leak, MERGE-OK, on top of session head d81c8293; it touches merge-gate.sh). One round, stop at 250 k. Cost about 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "gate-silent BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md GATE-SILENT (~318) and MC-47-CELL (~324); reports/mc-fix/mc-47-cell.patch; reports/audit-f.md "## r4" F5 (launcher-click red-by-host, fuzzel 1.9.2 vs 1.11.1). Not yours: flake fixes of flakes-l/m/n, matrix cap (flakes-n), lint lists.
Items (a red test commit first, then the fix):
1. GATE-SILENT: a cell whose subtests do not run (tool missing or too old, e.g. fuzzel < 1.11, xp-fields Run box) prints SKIP with the reason in the gate summary, never PASS. Red: a tests/gate-lib selftest that feeds a cell with no subtests run and expects SKIP. One shared fuzzel version check (lib) used by launcher-click and the Run box cells: below 1.11 SKIP with "fuzzel <ver> < 1.11", not red.
2. A train must still fail when a needed cell is SKIP: the summary lists SKIP cells and `--push` refuses when any cell is SKIP unless `--allow-skip <cell>` names it (state the rule in the script header). Red: selftest.
3. MC-47-CELL: apply the patch (mc-sofa launcher-click row) behind the same guard.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,xp-fields,launcher-click,mc-sofa`, once with fuzzel 1.11+ on PATH (PASS) and once with a PATH that hides it or a stub of 1.9 (SKIP, and --push refused in a dry run); poll in the same turn; never end a turn to wait.
Done when: red shas before fixes; both runs as above with their summaries quoted; backlog GATE-SILENT and MC-47-CELL closed; reports/gate-silent.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/gate-silent and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
