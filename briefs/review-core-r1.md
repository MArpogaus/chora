# review core r1 (rc 6 core and security, rc 9a logging)

Lane: Fable, cloud, branch claude/review-core from the session branch head. One round.
Scope: crates/chora (core), chora-kit, the IPC/API surface, the escape hatch, process
spawning, file and socket permissions, the voice service's network and tool paths.
1. Review: simplicity and readability first, then correctness, security, dead code,
   docs. Write reports/review-core-r1.md on claude/session-log: one line per finding
   (id, HIGH/MEDIUM/LOW, file:line, text). Push it early, update at the end.
2. Fix every HIGH and every security MEDIUM on claude/review-core, red test first.
   Other MEDIUM: one line in the report for the known issues. Do not touch
   crates/chora/src/words.rs or hang.rs (lane flakes).
3. Logging audit (rc 9a) for core and kit: every error path logs once with context;
   no secrets logged. Findings in the same report.
Done when: the full merge gate passes. Report: counts per severity, fixed ids, commits.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify; stage by name; no force push; no pull request;
never commit Microsoft files or captures with them. Stop at 250 k context and hand over. Push claude/review-core and the report to claude/session-log.
