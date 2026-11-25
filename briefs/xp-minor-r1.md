# xp-minor r1 (rc 1 known issues, F7; xp has no open BLOCK or MAJOR, rules.md 6): four xp MINORs that personas and the judge hit

Model: sonnet, effort high (four small product fixes in chora-xp with red tests; routine but in the showcase look)
Lane: cloud, branch claude/xp-minor-r1 at 721be592 (session head t134). Areas: plugins/chora-xp (taskbar.rs pager, Turn Off dialog, helper chat header, paperclip click), its tests and gate cells. Stop at 250 k. Cost at most 1.0 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "xp-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read findings/xp.md rows xp-9, xp-19, xp-28, xp-29, rv9j-1 on claude/session-log, the vision card (XP SP2/SP3 Luna, POLS for XP users) and design/looks (xp).
Each item: red test commit first (tests only, message names the id), then the fix commit; a pixel or node assert in the xp gate cells, not a look at a screenshot.
1) rv9j-1 + xp-19 (taskbar.rs:94 pager_colors): the current desktop button must differ from the others by at least 3:1 (now #164EB8 vs #1E54BE, about 1.07:1), and the four buttons sit in the tray tone (Luna tray blue, not own flat squares). Same colors on the cube xp faces if they share the code.
2) xp-28: after the helper answers, the chat header leaves "Working, step 1 of 1" and Stop goes inactive; an answer with no steps shows the idle state. Test drives a finished answer.
3) xp-9: Turn Off dialog Cancel at least 24 px high (XP: 75x23 at 96 dpi, take 24), icon captions at least 14 px; no other layout change.
4) xp-29 (owner 11-17 10:35): a click on the paperclip itself opens its balloon and the chat, like the tray icon does; the tray path stays.
Out of scope: any other xp row, other looks, chora-notify, ratchet counts; no new sleep (sleep-ratchet).
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: four red-then-fix sha pairs with pasted red lines; full gate EXIT 0 (or foreign reds named); one capture per item in media/<date>/xp-minor-r1/ named in the report.
Report: reports/xp-minor.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/xp-minor-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
