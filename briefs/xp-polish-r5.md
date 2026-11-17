# xp-polish r5 (rc 1): Notepad that opens for Erika, honest when it cannot

Model: sonnet, effort medium (one launcher fallback and its failure state)
Lane: cloud, branch claude/xp-polish; merge the session branch 4105f8d1 first. One round. Stop at 250 k.
Read reports/review-tour-v5.md, reports/media-r3.md (N1, N8), reports/journey-1.md (j1-2), design/looks/xp.md, dev-rules.md.

1. j1-2/N1 MAJOR: Notepad (xp editor entry) tries the configured editor, then a fallback list of
   graphical editors (for example gnome-text-editor, gedit, mousepad, kate), then a terminal editor in
   a window; when it falls back or nothing starts, one plain notice says which and why. Red cell
   first: with the configured editor missing, a window opens and the notice names the fallback.
2. Keep the config plain text and the default list documented (docs and the plugin README).
Do not add packages to the container; test with fake editors on PATH.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/xp-polish-r5.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/xp-polish and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
