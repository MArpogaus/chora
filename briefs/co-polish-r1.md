# co-polish r1 (rc 1 known issues, F7; construct has no open BLOCK or MAJOR, rules.md 6): co-44 COSMETIC, co-45 MINOR

Model: sonnet, effort medium (two small fixes in chora-construct hud with red gate rows; routine)
Lane: cloud, branch claude/co-polish-r1 at d86be5d0 (session head). Areas: plugins/chora-construct (hud.rs, the key overlay), tests/acts/known-red.toml (construct rows only), docs. Stop at 250 k. Cost at most 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "co-polish-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read findings/construct.md co-44, co-45 on claude/session-log, the vision card (construct) and tour-v9.md section 7 (gaps in whole units, construct 8; text 14 px).
Each item: red test commit first (tests only, message names the id), then the fix commit; a unit test or a gate assert, not a look at a screenshot.
1) co-44: hud.rs place_chip: chip x from whole units so neighbour gaps are 8, not 9 (rects [289,652,187,32], [485,652,129,32], [623,652,197,32]). Red: a unit test on place_chip gaps for those widths. Then remove the construct-mode-step-in row of tests/acts/known-red.toml.
2) co-45: the key overlay (Super+F1) headings construct-help-body-scope, -head-0, -head-14-every, -head-6 at 14 px, not 11 px (construct is not xp). Red: a unit test on the overlay font size; then remove construct-list-body-scope and matching rows of known-red.toml if the acts pass.
Keep the hints contrast row (mid-fade read) unless the acts show it met 5/5.
Out of scope: other looks, camera, adopt, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: red-then-fix sha pairs for items 1-2 with pasted red lines; known-red rows removed only where acts pass; full gate EXIT 0 (or foreign reds named).
Report: reports/co-polish.md "## r1 (lane co-polish-r1)" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-polish-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
