# kit3 r2 (F2): fix round after review r1 (verdict FIX, rk-1 MAJOR)

Model: sonnet, effort high (a port on fixed types: two more looks onto `keysheet::rows`)
Lane: cloud, branch claude/kit3 from 5b788c72. One round, stop at 250 k. Cost about 2 %.
Step 0: setup (below), then `git fetch --unshallow` and `cargo check -p chora`; a failure: send_message the PM "kit3 r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/kit3.md "## Review r1" (rk-1 to rk-8) and briefs/kit3-r1.md on claude/session-log.
1. rk-1 MAJOR first: construct (config.rs:498-620 hand rows) and xp (help.rs `LINES`, `rows`) take their rows from
   `keysheet::rows` with `named` for look-only actions; both tables go. Red first: a kit or look cell that fails while a
   look keeps its own table (e.g. the drawn rows equal `keysheet::rows` of its bindings). Visible rows and keys stay;
   a lost row or renamed key is a fault. Each look keeps its own costume.
2. Then rk-2 (give row when any give key is held), rk-3 (cut-known xp-chat max 4), rk-4 (one German give wording, in
   words.table and the cell), rk-5 (keep the thin space in tiling), rk-6 (keep the MOD filter, or name the change in
   docs/configuration.md); rk-7, rk-8 if cheap.
3. Merge: kit5b and kit7 conflict with this branch; lane integ merges them on claude/integ. Before the gate merge
   origin/claude/autonomous-dev-session-3w-i33pg7, and origin/claude/integ if reports/integ.md says MERGE-OK or done
   and the session branch lacks it. Resolve so both sides keep their effect; name each resolution in the report.
Done when: rk-1 to rk-6 each have a sha or a written reason; red then green; `tests/merge-gate.sh --jobs 3` once on the
head, green (poll in the same turn); `tools/cut-check` REPORT lines not grown.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and append
"## r2" to reports/kit3.md on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/kit3 and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
