# decisions r1 (rc 8): bring docs/decisions.md up to date

Model: sonnet, effort low (docs only, sources are named below)
Lane: cloud, branch claude/decisions from the session branch head. One round, at most 3 hours.
Read dev-rules.md, conventions.md and reports/debt-rc.md (claude/session-log, section 3c) once.

Goal: docs/decisions.md records every design decision that landed since 11-14 (last change 1a3f4b57).
Add one D-entry, in the file's existing format and numbering, for each of the nine items of
debt-rc.md 3c (injected input flag; API keys out of the agent environment; icon dir, FIFO and small
PNG checks; log location, rotation, content and chora-report scrub; watchdog wait as host load;
mc root order below other plugins; construct placed pane stays in view; xp Notepad fallback chain;
tiling wobbly on by default). Each entry: context, decision, consequences, the commits that carry it.
Read the commits and code before writing; write only what they show. Where code and docs disagree,
say so in the report, do not decide. Add the open questions D1-D8 of debt-rc.md 3b as entries with
status "open" (no decision text). Also run `git log --first-parent --since=2026-11-14` on the session
branch and list in the report any other merge with design weight and no entry.
Docs only: do not touch code or tests. Done when pre-commit and the docs build pass.
Report (reports/decisions-r1.md, at most 25 lines): entries added, disagreements, merges left out.
Also a section "Refactor proposals" (dev-rules.md): what you saw while reading.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify; stage by name; no force push; no pull request. Push only claude/decisions.
Stop at 250 k context and hand over.
