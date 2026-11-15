# logging r1 (rc 9a, backlog L1): report

Branch claude/logging-r1 at 150db8f0 (base 98291088), pushed. Audit: reports/logging-audit-r1.md.

Commits: 0f5e1f08 test(voice) all-level capture; cdf98806 test(chora) refused request; d4666f3e
fix(voice) no spoken/typed text at any level, deps below info off, panic via tracing; ab747722
fix(chora) refused request logs client+code; 2ded0e4c feat(scene) rotating files; 27c5b16b fix(chora)
smithay keysyms off at trace; 5c1210a7 feat(chora) chora.log + panic words; 7cbd7456 feat(voice)
voice.log; 47f2ce7a feat(dist) chora-report + RPM/Arch; 150db8f0 docs/logs.md, nav, glossary.

Findings: 3 BLOCK (voice debug transcripts L1, rmcp tool args L2, smithay keysyms at trace L3),
2 MAJOR (refused IPC request without a line L4, no log file / panic only on stderr L5): all fixed.
4 MINOR open (L6 plugins use unstructured eprintln, L7 one voice target, L8 argv at info, L9 notify summary).

Red: voice 8 FAILED of 236 (text in debug lines); chora refused-request test "no line of the
refusal"; log::tests key press FAILED with "TRACE smithay::input::keyboard ... sym=\"h\"";
logs::tests 2 FAILED with rotation off; log_file.rs FAILED (no chora.log); chora_report.rs
FAILED without the script. Green: all of them.
Gate: tests/merge-gate.sh one run, all 28 steps PASS, no flake (/tmp/merge-gate.oSQOTX).
Also: ci-local docs, deny, machete, packaging, pre-commit PASS; packaging-files.py 0 gaps;
shellcheck clean on dist/chora-report (the hook skips files without .sh: it did not run there).

Deviations: plugins keep stderr only (journal); chora-report bundles their journal. No
compile-time check: the rules are about values, a disallowed-macros ban cannot see them.
Rotation is hand-written (std, 100 lines): tracing-appender rotates by time only, file-rotate
pulls chrono and flate2.
Open: tests/battle/w16.sh check 5 greps "say: Shall I start the program foot" in the debug log,
now "say: chars=N" (w16 is not in the gate; tests/ is the flakes lane's: needs another signal,
for example the confirm event). For the backlog: L6 plugins to tracing with a target each.
