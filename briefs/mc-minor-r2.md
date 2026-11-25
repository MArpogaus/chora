# mc-minor r2 (rc 1 known issues, F7; mc has no open BLOCK or MAJOR, rules.md 6): the r1 review gaps and three more mc MINORs

Model: sonnet, effort high (small layout and theme fixes in chora-mc with red tests; routine but in a look)
Lane: cloud, branch claude/mc-minor-r2 at 5cb288e2 (mc-minor r1, MERGE-OK, waits for train138). Areas: plugins/chora-mc (page, layout, region, key list, volume door, notice card place), the fuzzel config it writes, its tests and gate cells. Stop at 250 k. Cost at most 1.0 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "mc-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/mc-minor.md "## PM review r1c" (rmm1-1..4) and reports/rc1-pass.md rows rpx1-10, rpx1-15 on claude/session-log, the vision card (mc: content shapes the view, traditional typesetting; remote and mouse a full second path) and design/looks (mc, mc-style.md).
Each item: red test commit first (tests only, message names the id), then the fix commit; a pixel or node assert in the mc gate cells, not a look at a screenshot.
1) rmm1-1: no line of any built-in scene is cut at the page edge at 1280x800 and 1280x720 (ink check over every scene, not only morning); the unit test `every_panel_of_a_scene_is_placed_at_1280_by_800` goes back to 0.5 px slack.
2) rmm1-2: the page bottom and the header end come from the rects the HUD host and chora-notify report (state or query), not from the constants PILL_ROOM and CARD_ROOM; a cell moves the card or pill and the page follows.
3) rmm1-3: a cell clicks a row of the full key list (Super+F1) and its action runs.
4) rmm1-4: a stacked page that does not fit (tour "sauce") shows its last line at rest, or says by a mark at the edge that more follows (pick the simpler; KISS).
5) rpx1-10: launcher and power menu (fuzzel) take the mc theme of the session (dark page: dark box), text at least 4.5:1 against its box.
6) rpx1-15: a mouse way to change the volume level (e.g. click on the level steps up or down, or a wheel on the volume door); the remote path stays.
Out of scope: rpx1-16, rpx1-17, rv9j-2, other looks, chora-notify code, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: six red-then-fix sha pairs with pasted red lines (item 3 may be test only if it passes at once); full gate EXIT 0 (or foreign reds named); one capture per item in media/<date>/mc-minor-r2/ named in the report.
Report: reports/mc-minor.md "## r2 (lane mc-minor-r2)" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/mc-minor-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
