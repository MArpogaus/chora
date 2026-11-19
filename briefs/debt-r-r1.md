# debt-r r1 (F4): debt recount after the cube split

Model: sonnet, effort high (routine count with proof)
Lane: cloud, branch claude/debt-r from train 65 85764974 (integ, cubesplit, kit3, flakes-g, focus-port, moved-tests xp and ct). Read-only on code. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "debt-r BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/audit-f.md (section 1 row F4, section 2 rank 1, line 77) and reports/debt-rc.md, reports/debt-2026-11-19.md on claude/session-log.
1. Sizes: `tools/size-ratchet` and the size script of audit-f section 5: production files over 1,500 lines (target 0).
2. The five debt-rc categories (workarounds 9, refactors 7, design questions 8, records 9, hygiene 14): each RC item open or closed, with proof (file:line still there, or the merge or commit that closed it); new items since the RC (grep TODO, FIXME, HACK, workaround, known-red.toml rows, backlog refactor rows) counted per category.
3. Lint lines by the exact commands of audit-f row F4: `#[allow]`/`#[expect]` total, without `reason`, and `eprintln!` in core, plugins, services; RC values 378, 238, 135.
Done when: reports/debt-2026-11-20.md on claude/session-log has a table per category (RC count, now, items with proof), the size line and the lint lines, and a computed line "F4: met" only if every category is at or below RC, the total is lower, and the lint lines are at or below RC; else "F4: open" with the rows above RC (they feed LINT-DOWN). No gate run (read-only).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push only the branches named here and claude/session-log (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
