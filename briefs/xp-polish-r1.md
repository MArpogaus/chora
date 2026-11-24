# xp-polish r1 (rc 8; looks): three proven xp MINORs from verify-minor r2

Model: sonnet, effort high (layout of the chat card and start menu plus a task button; routine with care)
Lane: cloud, claude/xp-polish at 2235551f (session head, t125). One round. Stop at 200 k context; cost at most 1 %.
Why: reports/verify-minor.md "## r2" on claude/session-log reproduced these with a dump line; xp has no open BLOCK or MAJOR (rules.md 6).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`; a failure: send_message the PM "xp-polish BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read: findings/xp.md xp-14, xp-15, xp-17; reports/verify-minor.md "## r2" and "## Seen"; design/looks/xp.md and design/xp-reference.md.
1) xp-14: "Typing does not answer." stands inside the question card (chat-question.png of tests/xp.sh step 16b: y 576 under card bottom 560). The unit test chat::view::tests::the_typing_line_stands_in_the_card_and_not_above_the_field passes while the real layout fails: first make the test measure what is drawn (red on 2235551f), then fix the layout.
2) xp-15 label: a long Start menu name (ImageMagick "(color depth=q16)") reads cut; XP behaviour: one ellipsis at the row end, full name on hover (tooltip) if the shell has one; no new widget. Red first: unit or xp.sh assertion.
3) xp-17 Display: the Display dialog gets a taskbar button like Help (wm.rs:107 HELP_TASK pattern); xp.sh 12b asserts it. Red first. Sounds and key focus stay out (verify).
Done when: 1-3 in with red commits before fixes; full gate (tests/merge-gate.sh, no --only, --jobs 2) on the head, foreign reds named by cell; one capture each (card, menu row, taskbar) named in the report.
Report: reports/xp-polish.md on claude/session-log "## r1" (commits, outputs, gate cells, captures). Then send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, done items, reds.
Poll every command in the same turn; never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Rules: push claude/xp-polish and (report only) claude/session-log; fetch and merge on reject, no force; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages; never a bare git stash; no new `sleep` line (sleep ratchet).
