# co-minor r1 (rc 2 known issues; construct has no open BLOCK or MAJOR after t136, rules.md 6): three construct MINORs

Model: sonnet, effort high (small fixes in chora-construct with red tests; routine, one look)
Lane: cloud, branch claude/co-minor-r1 at 6f37120b (session head t136: co-75, co-76, co-79 fixed). Areas: plugins/chora-construct (key sheet, face windows, program start in a face), its tests and gate cells. Stop at 250 k. Cost at most 1.0 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "co-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read findings/construct.md rows co-77, co-78, reports/co-reach.md (rcr1-1 MINOR) and reports/co-repro.md r1 on claude/session-log, the vision card (construct: glass, calm, minimal HUD; WM7b hosted: construct holds the face turn, flat fallback) and design/looks (construct).
Each item: red test commit first (tests only, message names the id), then the fix commit; a node or pixel assert in a construct gate cell, not a look at a screenshot.
1) co-78: Escape closes the key sheet also after Super+t opened a face window behind it (Close click already does); focus returns to that window.
2) rcr1-1 (with co-78): Super+t with the key sheet open starts a face window in the face, not a pane, so the demo face is not empty and its hint is told.
3) co-77: Foot started from Programs in a face that holds a window is held by construct (hosted face, WM7b decision 11-19), visible in that face, keys reach it; 5 of 5 runs in the cell; if the contract C3 is wrong instead, fix the contract and say why in the report (one line).
Out of scope: co-44, co-45, other construct rows, other looks, HUD layout, ratchet counts; no new sleep (sleep-ratchet; the gate's `ratchets` step refuses one).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: three red-then-fix sha pairs with pasted red lines; full gate EXIT 0 (or foreign reds named); one capture per item in media/<date>/co-minor-r1/ named in the report.
Report: reports/co-minor.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-minor-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
