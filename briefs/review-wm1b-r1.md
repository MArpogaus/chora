# review wm1b r1 (F2, F3): independent review of claude/wm1b before its merge

Model: sonnet, effort high (port review; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: review claude/wm1b at 0ffc53a8 against its base 27bd60c7. One round.
Stop at 250 k context and hand over. Cost about 0.7 %.
Read briefs/wm1b-r1.md (the done criteria), reports/wm1b.md and reports/wm1.md on claude/session-log.
Check: 1) mc lifecycle goes only through kit::wm::book; no claim filter, known check, reattach or state
adapter left in mc (grep, name the lines); 2) cells were red before the port (git log order, run the red
commit's tests if cheap); 3) behaviour unchanged: detach puts the window back, give out is a proposal,
restore from 29e2cd77 meta; 4) copies.toml counts true (`tools/cut-check`); 5) kit::testing::Script is
generic and used by the wm_book tests; 6) simplicity, dead code, refactor ideas. Run `cargo test -p chora-kit
--features testing` and `cargo test -p chora-mc` (poll in the same turn; never end a turn to wait).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm1b.md on claude/session-log under
"## Review r1". Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6
lines): verdict, counts, BLOCK and MAJOR list.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
