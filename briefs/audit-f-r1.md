# audit-f r1 (F1-F8): mid-week foundation audit, gaps ranked into slices

Model: opus, effort high, because this is planning: it decides what the last 37 h of lanes build (rules.md 9)
Lane: cloud, read-only on code, branch claude/audit-f at the session head. One round, stop at 150 k. Cost about 2 %.
Step 0: setup (below), then `git fetch --unshallow` and `cargo check -p chora`; a failure: send_message the PM "audit-f r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: design/foundation.md (F1-F8, "met" needs value, command, independent check), state.md,
reports/arch-plan-fable.md (slice table), reports/debt-rc.md, reports/debt-2026-11-19.md, backlog.md (last 40 lines).
Also fetch claude/integ, claude/kit3, claude/wm3, claude/wm7b, claude/kitcov, claude/cubesplit, claude/flakes-g (in flight).
For each of F1-F8: the measured value on the session head plus in-flight branches, the command, met / open, and the gap
in one line. Use: `tools/cut-check` (REPORT lines), `tools/size-ratchet`, `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit
--features chora-kit/testing --summary-only`, tests/acts/known-red.toml, the debt counts, docs/decisions.md.
Then at most 8 slices that close gaps by Sat 03:59 UTC, ranked by F moved per % budget: name, F, files, done criterion,
model (haiku tests, sonnet routine, opus complex), cost %, conflicts with in-flight branches. No new feature; visible
behaviour as tour v8; F7 needs the new tour (next week, owner 11-17): say what part, if any, can be measured now.
Be strict (owner 11-17: perfectionist about met targets): "partly" is open.
Output: reports/audit-f.md on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN
(at most 6 lines: met count, top 3 slices). Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
