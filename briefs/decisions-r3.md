# decisions r3 (rc 8): D87-D94 are proposals, not fixed decisions

Model: sonnet, effort low (wording change in one file)
Lane: cloud, branch claude/decisions, continue from its head (80a5931). One round, at most 1 hour.
Read answers.md line "2026-11-17 04:41 UTC" on claude/session-log.
Change: entries D87-D94 (the owner's answers of 04:38) get status "proposed (owner, 2026-11-17)" instead
of decided, and one sentence each: "May be revised or reverted when a later review finds a better design."
Keep their option and consequences text. Add nothing else. Docs only. Done when pre-commit and
mkdocs --strict pass. Report: two lines appended to reports/decisions-r1.md, one send_message to the PM.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify; stage by name; no force push; no pull request. Push only claude/decisions.
