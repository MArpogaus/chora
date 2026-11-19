# audit-f r2 (F1-F8): evening foundation audit after the r1 slices, gaps ranked into slices

Model: opus, effort high, because this is planning: it decides what the last 32 h of lanes build and which F count as met (rules.md 9; owner 11-17 10:59 perfectionist)
Lane: cloud, read-only on code, branch claude/audit-f at the session head. One round, stop at 150 k. Cost about 2 %.
Step 0: setup (below), then `git fetch --unshallow` and `cargo check -p chora`; a failure: send_message the PM "audit-f r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Tree to measure (P): origin/claude/autonomous-dev-session-3w-i33pg7 (b5bf800c or newer) merged locally with c00dcf3e
(claude/moved-tests-mk3), f7a69761 (claude/mc-i5), da68737d (claude/construct-red), all MERGE-OK; if the session head already
contains them (train 69, 70), measure the head. claude/debt-down is in a fix round: measure it separately as an add-on.
Read on claude/session-log: design/foundation.md, reports/audit-f.md (r1: section 1, the 7 slices, section 4 exact measures),
state.md, reports/debt-rc.md, reports/debt-down.md, reports/moved-tests.md, reports/copies-0.md if present, backlog.md (last 40 lines).
For each of F1-F8 re-measure with the r1 commands and the section 4 exact measures: value on P, command, met / open, gap in one line.
Be strict: "partly" is open; a met needs value, command and this independent check. F1, F3, F6, F8 were met earlier: re-check, do not assume.
F5 flakes: count train reruns since 11-19 14:40 (git log of the session branch, /tmp logs are not yours: use reports/flake-series.md if present).
Then at most 6 slices that close the remaining gaps by Sat 03:59 UTC, ranked by F moved per % budget: name, F, files, done criterion,
model (haiku tests, sonnet routine, opus complex), cost %, conflicts with debt-down and flake-series (in flight). No new feature;
visible behaviour as tour v8. If no slice can move an F, say so and name the best use of the budget for stability and looks (vision card).
Output: append "## r2" to reports/audit-f.md on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN
(at most 6 lines: met count, top 3 slices). Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
