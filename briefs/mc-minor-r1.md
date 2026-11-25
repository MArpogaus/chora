# mc-minor r1 (rc 1 known issues, F7; mc has no open BLOCK or MAJOR, rules.md 6): five mc MINORs from rc1-pass and the tour

Model: sonnet, effort high (five small layout fixes in chora-mc with red tests; routine but in a look)
Lane: cloud, branch claude/mc-minor-r1 at 721be592 (session head t134). Areas: plugins/chora-mc (column, chat panel, key list, timer card, footer, pill, Helpers page), its tests and gate cells. Stop at 250 k. Cost at most 1.0 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "mc-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read findings/mc.md rows mc-56, mc-61 and reports/rc1-pass.md rows rpx1-9, rpx1-11..rpx1-14 on claude/session-log, the vision card (mc: content shapes the view, traditional typesetting) and design/looks (mc, mc-style.md).
Each item: red test commit first (tests only, message names the id), then the fix commit; a pixel or node assert in the mc gate cells, not a look at a screenshot.
1) mc-56: with the chat open the column doors (Programs, Lock, Power, Volume) are not left under the chat panel with live hit areas: hide them (and their hit areas) while the chat is open, or place the chat above them; pick the simpler; a click where a hidden door was does nothing.
2) rpx1-9: the Super+F1 key list: no rule line through "Mute"; at 1280x720 every row and door stays on screen.
3) rpx1-11 + rpx1-12: the timer card and the footer band never cover the last page line: the page reserves their height (the last line, e.g. "parsley.", fully visible).
4) rpx1-13: with the column hidden the state pill does not cover the header clock.
5) rpx1-14: the Helpers page with no helper says so in words (e.g. "No helper set up"), in the page type.
Out of scope: rpx1-10 (fuzzel theme), rpx1-15 (volume level), rpx1-16, rpx1-17, rv9j-2, other looks, chora-notify, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: five red-then-fix sha pairs with pasted red lines; full gate EXIT 0 (or foreign reds named); one capture per item in media/<date>/mc-minor-r1/ named in the report.
Report: reports/mc-minor.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/mc-minor-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
