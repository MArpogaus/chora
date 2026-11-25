# mc-minor r3 (rc 1 known issues, F7; mc has no open BLOCK or MAJOR, rules.md 6): the r2 review MINORs

Model: sonnet, effort medium (small fixes in chora-mc with red tests; routine)
Lane: cloud, branch claude/mc-minor-r3 at 0f68810c (mc-minor r2, MERGE-OK, waits for train140). Areas: plugins/chora-mc (rooms.rs, acts.rs), tests/mc-edges.sh, docs/mc-style.md. Stop at 250 k. Cost at most 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "mc-minor-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/mc-minor.md "## Review r2" (rmm2-1..6) on claude/session-log, the vision card (mc) and docs/mc-style.md.
Each item: red test commit first (tests only, message names the id), then the fix commit; a unit test or a gate-cell assert, not a look at a screenshot.
1) rmm2-1: the card room counts a card only when it is on mc's output and under the page top (output coordinates); a top or middle card leaves the page whole.
2) rmm2-2: a card of another height never reflows the pages (keep the larger room seen, KISS); a tall card does not leave the page short after it closes.
3) rmm2-3: the wheel steps one notch per discrete step; Finger and Continuous scroll accumulate dy to one step per notch.
4) rmm2-4: the ink cell takes x and w from the page rect, fills agents and overview; it is red on the base overview.yaml (paste the red line from a temporary revert, then restore).
5) rmm2-5: a FakeCore unit test: moving hud-word sends no page rebuild (test only if green at once).
6) rmm2-6: docs/mc-style.md names the more mark, the volume wheel and the themed menus (docs commit only).
Out of scope: other looks, chora-notify code, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: red-then-fix sha pairs for items 1-4 with pasted red lines (5 may be test only, 6 docs only); full gate EXIT 0 (or foreign reds named).
Report: reports/mc-minor.md "## r3 (lane mc-minor-r3)" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/mc-minor-r3 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
