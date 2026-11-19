# review kit2b r1 (F2): independent review of claude/kit2b before its merge

Model: sonnet, effort high (port review; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: review claude/kit2b at ce0a564d against its base 27bd60c7. One round.
Stop at 250 k context and hand over. Cost about 0.7 %.
Read briefs/kit2b-r1.md (the done criteria), reports/kit2b.md and reports/kit2.md on claude/session-log.
Check: 1) construct's key actions go only through kit::actions (Common<Own>, System); no own action enum
or match left for the shared actions (grep, name the lines); `kit::actions::perform` is the one place
System acts are performed, mc and construct only implement Host; 2) red cells 4c5eeb7f failed before the
port (run them if cheap); 3) behaviour: the 29e2cd77 construct table loads unchanged; judge the two
visible changes the builder reports (construct now acts on run/give; old names load in mc too) against
the brief's "visible behaviour unchanged": a bug fix with a cell is fine, a new feature is MAJOR;
4) `Settings::renamed` (keys read as bindings): no silent override when both keys exist, a warning;
5) the gate was never one full run (cube flaked once under --jobs 3): rerun `cube` twice and say flake or
cause; 6) simplicity, dead code, refactor ideas. Run `cargo test -p chora-kit --features testing`,
`cargo test -p chora-construct` and `cargo test -p chora-mc` (poll in the same turn; never end a turn to wait).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rk-N, severity,
file:line, text, fix in one sentence). Append to reports/kit2b.md on claude/session-log under
"## Review r1". Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6
lines): verdict, counts, BLOCK and MAJOR list.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
