# logging r1 (rc 9a): logging audit and chora-report

Lane: opus, cloud, new branch claude/logging-r1 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (98291088 or newer). One round. Stop at 250 k context and
hand over. Read dev-rules.md and conventions.md (with the skills it names) first, then backlog L1
and design/rc.md row 9a. Another lane (flakes r5) works in tests/ and the xp theme cache; harness r2
in tests/merge-gate.sh and hooks: keep away from those files.
Items, red first where a test fits:
1. Audit: a table per component (core crates, each plugin, services/voice, choractl) of what it
   logs, at which level and target, and where it goes. Findings ranked: secrets, typed text, chat
   text or voice transcripts in a log = BLOCK; a crash or refused request without a log line =
   MAJOR; noise at info = MINOR. Report as reports/logging-audit-r1.md on claude/session-log.
2. Fix every BLOCK and MAJOR: structured tracing, one target per area, no typed or spoken text,
   no tokens; a test per rule (a unit test that a typed string never reaches the log, or a clippy
   disallowed-macros/grep check where the rule is mechanical: compile time first).
3. Files: rotating logs under $XDG_STATE_HOME/chora/logs (size cap, keep N), honest default level;
   documented in docs/ (how to read them).
4. `chora-report` in dist/: a shell script that bundles logs, versions and config into an
   anonymised .tar.gz the person sends by hand (no upload); scrubbing tested on a fixture (user
   name, home path, host name, tokens). docs: how to report a problem.
Done when: tests/merge-gate.sh passes as one run (a known flake F16-F20: rerun once, name it).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines) and as reports/logging-r1.md.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/logging-r1 and the
reports; never commit Microsoft files; never end a turn to wait for a long background job.
