# audit-f r5 (F1-F8 strict re-measure after t115): which goals are met, with evidence, and the last slices to Sat 03:59 UTC

Model: opus, effort high, because this is planning: it decides which F count as met and what the remaining days build (rules.md 9; owner 11-17 10:59 perfectionist)
Lane: cloud, read-only on code, branch claude/audit-f-r5 at the session head bc153929. One round, stop at 120 k. Cost about 0.8 %.
Step 0: no setup.sh (the environment setup ran it); `git fetch --unshallow || true` and `cargo check -p chora`; a failure: send_message the PM "audit-f r5 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Tree to measure (P): origin/claude/autonomous-dev-session-3w-i33pg7 (bc153929 or newer). Running lanes (do not merge, read their briefs): flake-series r6, sim-long r2, tiling-disc r1, xp-readable r1, mc-tidy r1, leak r1, p2-r7-co.
Read on claude/session-log: design/foundation.md, design/rc.md, reports/audit-f.md "## r4", state.md, findings/shared.md (sh-31..34), reports/flake-series*.md, reports/sim-long.md, reports/record-v9.md "## r4" and "## Review r4", backlog.md lines with FLK-.
1) F1-F4, F6, F8 (met by r4): re-measure each with the r4 commands on P; any count above its RC value or ratchet reopens it. No assumption carried from r4.
2) F6: every change on P since 5b34d4b1's r4 point came through tests/merge-gate.sh (merge parents against the train logs named in state.md); any direct commit is open.
3) F5: open flakes (FLK- ids, sh-32..34) against the bar; is the flake-series r6 red (keys, sh-34) a flake or a fault? Coverage still over 80 % (command and value).
4) F7: count of acts that differ from v8 on the latest tour report and which are filed per look; name what must hold for F7 met.
5) RC (design/rc.md): one line per criterion, state and the shortest path.
6) At most 4 slices to Sat 03:59 UTC ranked by F or rc moved per % budget: name, F/rc, files, done criterion (with its red cell), model (haiku tests, sonnet routine, opus complex), cost %, conflicts with the running lanes. No new feature.
Output: append "## r5" to reports/audit-f.md on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines: met count, open F with gap, top slices). Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force); author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
