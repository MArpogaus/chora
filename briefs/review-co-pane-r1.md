# review co-pane r1 (RC 2, construct I3, key sheet): seeded review of claude/co-pane before its merge

Model: sonnet, effort high (a look plugin and the persona checker, no core change; seeded review)
Lane: cloud, read-only on code: claude/co-pane head f10842d8, base bbaad12b (10 files, +288 -18). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green); `cargo test -p` only.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-co-pane r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-pane-r1.md (done criteria) and reports/co-pane.md "## r1" on claude/session-log; `git diff bbaad12b..f10842d8`.
Checks, each with its command and files:
1) Look away (plugins/chora-construct/src/camera.rs unseen_focus_to_walk, state.rs in_sight): the keyboard never stays on an unseen pane, comes back on a look back or a click, no loop with the glide; `cargo test -p chora-construct`.
2) Key sheet (overlay.rs, command.rs, input.rs, modes.rs): takes and gives back the keyboard only to a shown window; Close is a hit node at the drawn footer; the revert f10842d8 leaves no dead code of 1e92ccf5.
3) Residual of item 2 (Super+F1 over an xp program in a cube also opens the XP Start menu; Escape closes the menu first): say whether it breaks POLS badly enough to be MAJOR for the RC shared bar ("the key list ... no stray keys into clients") and name the smallest fix (construct swallows the lone Super tap in use mode, or xp opens Start on a tap only with the keyboard).
4) Look direction (drag down looks down, FPS-style; Max dragged down 3x): MAJOR, MINOR or as designed against design/looks construct and POLS; one sentence why. Also the report's "first drag after a fresh start does not look".
5) Checker (tests/persona/check.py with_children, tests/persona-selftest.sh): a child look's windows count as shown only when that child is inside a construct face; no hidden I1/I3 masked; `bash tests/persona-selftest.sh`.
6) Red first: a6e3a1b5, 918d4a3c, a3327873 precede their fixes and fail on bbaad12b for the named reason.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/co-pane.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
