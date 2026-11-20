# audit-f r4 (F4, F5, F6 recheck): strict re-measure before Sat 03:59 UTC, last slices ranked

Model: opus, effort high, because this is planning: it decides which F count as met and what the last ~13 h of lanes build (rules.md 9; owner 11-17 10:59 perfectionist)
Lane: cloud, read-only on code, branch claude/audit-f-r4 at the session head d81c8293. One round, stop at 120 k. Cost about 0.6 %.
Step 0: no setup.sh (the environment setup ran it); `git fetch --unshallow || true` and `cargo check -p chora`; a failure: send_message the PM "audit-f r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Tree to measure (P): origin/claude/autonomous-dev-session-3w-i33pg7 (d81c8293 or newer). Pending MERGE-OK shas (not yet in P; read their reports, do not merge): co-r5 c2b0310f (train 88b), probe-leak f5ac3500 (sh-30), co-majors-r4 e0c3a266, lint-down 78ec62e1 (tools/lint-ratchet). Running lanes: mc-fix r4, flakes-l r2, flakes-m r1 (briefs in briefs/).
Read on claude/session-log: design/foundation.md, reports/audit-f.md "## r3", state.md, findings/shared.md (sh-30), reports/lint-down.md, reports/probe-leak.md, reports/flakes-l.md and flakes-m.md if present, backlog.md lines with FLK-.
1) F4: re-measure every F4 bar on P with the r3 commands plus tools/lint-ratchet and tools/size-ratchet; name each count against its RC value (workarounds, hygiene were open in r3). met / open, gap in one line.
2) F6: every change on P since 79c8d2b5 came through tests/merge-gate.sh: check the merge commits' parents against the train logs named in state.md and the session branch reflog of origin; any direct commit is open.
3) F5: count open flakes (FLK- ids, sh-30) against the bar; say whether greens at --jobs 1 hide flakes (they do not count, state.md) and what --jobs the bar needs.
4) F1-F3, F8: one-line re-check each (value, command); do not assume.
5) At most 3 slices that close F4 or F5 gaps by Sat 03:59 UTC, ranked by F moved per % budget: name, F, files, done criterion, model (haiku tests, sonnet routine, opus complex), cost %, conflicts with the running lanes. No new feature.
Output: append "## r4" to reports/audit-f.md on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines: met count, open F with gap, top slices). Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force); author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
