# review kit7 r2 (F2): seeded check of the r2 fixes on claude/kit7 before its merge

Model: sonnet, effort high (fix verification of 7 known findings plus one new state rule; the design had its r1 review)
Lane: cloud, read-only on code: claude/kit7 head 8321c16, r1 head c1dbc47b, base daf83391. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran its steps green).
Step 0, first 3 minutes: `cargo check -p chora` and `command -v foot fuzzel waybar`; if either fails, send_message
the PM at once "review kit7 r2 BLOCKED: no setup" and stop.
Read reports/kit7.md "## Review r1" and "## r2" on claude/session-log; `git log --oneline c1dbc47b..8321c16`;
`git diff c1dbc47b..8321c16`.
Checks, each with its command and files:
1) rw-1..rw-7 each closed: the red commit fails before its fix and passes after (check out the red sha, run the named
   test: `python3 tools/cut-check --base daf83391` for rw-1; `cargo test -p chora-kit --features testing helper` for
   rw-2/3/6; the per-look state.rs cells for rw-4). A fix that weakens a check is a finding.
2) The new rule of r2 (rw-6): "answered holds over voice listening" in crates/chora-kit/src/helper.rs state(). Vision
   card: honest states (working, no helper). Does answered ever stick: after a new question, a new working report, the
   helper's exit, a look handover? Name the clearing event in code and a test for each, or a finding.
3) rw-3: voice alone is no_helper in every look (xp, tiling, mc, construct) and docs/api.md says so.
4) kit7-a1 and kit7-a2 stay listed known red under WM2 (tests/acts/known-red.toml), each with a real failing act.
5) `python3 tools/cut-check` exit 0; no file over 1,500 lines; `cargo clippy -p chora-kit -p chora-assistant
   -p chora-xp -p chora-construct --all-targets` clean.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit7.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
