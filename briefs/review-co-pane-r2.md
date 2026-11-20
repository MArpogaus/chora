# review co-pane r2 (RC 2, construct I3, key sheet): seeded review of claude/co-pane before its merge

Model: sonnet, effort high (a look plugin and the persona checker, no core change; seeded review)
Lane: cloud, read-only on code: claude/co-pane head 628bb930, base f10842d8 (13 files, +434 -38); r1 reviewed bbaad12b..f10842d8. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran 15 cells green); `cargo test -p` and `bash tests/persona-selftest.sh` only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-co-pane r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-pane-r2.md (done criteria) and reports/co-pane.md "## Review r1" and "## r2" on claude/session-log; `git diff f10842d8..628bb930`.
Checks, each with its command and files:
1) rv-1 MAJOR (camera.rs regrab, swallow_taps; docs/cube-modes.md): in cube use mode a construct Super chord leaves no bare Super press and release for the client; the lone Super tap price is written down and no other key is swallowed (typing, Super chords of the program outside construct's grabs); there is no gate step (act.py sends a chord as one batch): say whether the unit test pins the client-side key stream or only construct's state; `cargo test -p chora-construct`.
2) rv-2, rv-3 (overlay.rs help_takes_keyboard, help_gives_keyboard): the sheet takes the keys from a pressed pane window; close gives the keys back only to a window still in sight (through take_focus or an equal check), else to the walk.
3) rv-4, rv-5 (command.rs footer hit only for the sheet or handled for the list; camera.rs unseen_focus_to_walk regrabs after a glide, with a test).
4) rv-6 (camera.rs turn, plans/construct-requirements.md F5, docs/cube-modes.md): a drag grabs the world, dx and dy both inverted, docs and F5 say so in the same words; the "first drag does not look" is by design (press on a pane window focuses it), pinned by d762d10f; POLS against design/looks construct.
5) rv-7 (tests/persona/check.py with_children, tests/test_acts.py, tests/persona-selftest.sh): a child look counts only as the manager of a face named by cube.list, shown only at the front face; a negative case breaches I1; `bash tests/persona-selftest.sh`.
6) rv-8 (toggle_help regrab kept with a red test, 628bb930) and red first: each item's red commit precedes its fix and fails on its parent for the named reason (shas in reports/co-pane.md "## r2"); build at least rv-1, rv-3, rv-6.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity, file:line, text, fix in one sentence). Append to reports/co-pane.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
