# xp polish r3 (rc 1, 3): question-card hint flake

Model: sonnet, effort medium (one bounded cause in one cell)
Lane: cloud, branch claude/xp-polish (r2 head 5a17aab6, MERGE-OK); merge the session branch
first (c949cc84; conflict in the tests/merge-gate.sh "# Steps:" header: keep both lists). One round.
Stop at 250 k context. Read reports/review-xp-polish-r2.md, dev-rules.md.

1. Train 31 (5a17aab6 on c949cc84) failed tests/gate-xp-question.sh at 1x: nodes-1.json
   has hud-card, hud-card-yes/no but no hud-chat-hint, 3 s after voice.confirm. Rerun on the
   merge: 1 of 2 failed. Find why the hint lags or is missing (chat.rs hint(), waiting(),
   glass.rs hint_node, the redraw after the card). If the hint can come later than the card,
   that is the bug: the card and the hint show in the same frame. Red first: make the cell
   poll the nodes for up to 10 s and assert the hint is present whenever the card is.
2. Run the cell 10 times at 1x and 2x; all pass. Name the runs in the report.
3. Optional only if trivial: the r2 MINOR (host.rs chat.restyle red test).

Handback: reports/xp-polish-r3.md on claude/session-log (cause, cell, 10 runs, head sha).
