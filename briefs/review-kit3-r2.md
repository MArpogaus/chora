# review kit3 r2 (F2): seeded review of the r2 fixes on claude/kit3 before its merge

Model: sonnet, effort high (port review: key rows from kit::keysheet in xp and construct; no core change)
Lane: cloud, read-only on code: claude/kit3 head 04ea5851. Own work of r2: commits 7c44ceb5 b43e555b 19f5e497 838c3e35
8e7ded04 a642f7ae eb549825 (`git show <sha>` each; ~520 lines); the rest was reviewed in r1 or is already on the session head. One round.
Step 0: `command -v cargo` and `cargo check -p chora-kit`; a missing tool: send BLOCKED (no setup) to the PM and stop.
Stop at 120 k context; cost at most 0.3 %; at most 8 findings. No merge gate run (the lane ran it green on 04ea5851).
Read reports/kit3.md on claude/session-log: "## Review r1" (rk-1 to rk-8) and "## r2".
Checks, each with its command and files:
1) Red first: 7c44ceb5 before b43e555b and 19f5e497; 838c3e35 before 8e7ded04 (`git log --oneline 5b788c72..04ea5851`).
2) rk-1 (MAJOR): plugins/chora-construct/src/config.rs and plugins/chora-xp/src/help.rs keep no own row table (`LINES`,
   `rows` gone); both draw `kit::keysheet` rows. The rows shown on the shipped config are unchanged (a lost or renamed row is MAJOR):
   `cargo test -p chora-xp -p chora-construct -p chora-kit --features chora-kit/testing`.
3) rk-2 to rk-7 closed as the r2 report says; rk-8 open by the report's reason (not a finding if the reason holds).
4) `tools/cut-check` REPORT lines not grown; tools/cut-known.toml xp-chat max 4 matches the count.
5) No new feature, key, wording or System variant beyond the named give wording (one in DE).
Known, not findings: mc I5 layout (net3-m1); conflicts with kit5b and kit7 (lane integ).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rk-N from rk-9, severity,
file:line, text, fix in one sentence). Append to reports/kit3.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list.
Authorized by this brief (the PM, for the owner): the push below and the send_message report; do not ask again. A refused
tool call is a blocker: report BLOCKED and stop. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
