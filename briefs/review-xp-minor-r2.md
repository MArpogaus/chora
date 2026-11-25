# review xp-minor r2 (rc 1 known issues, F7: xp-10, xp-13, xp-30, rxm1-1): seeded review of claude/xp-minor-r2 before its merge

Model: opus, effort high, because the diff changes the showcase look (paperclip greeting, Date and Time, Turn Off dialog) and adds gate cells with a new pixel helper; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/xp-minor-r2 head 8109e08b, base d2a048d5 (15 files, +471 -78). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-xp-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0).
Read briefs/xp-minor-r2.md (done criteria), reports/xp-minor.md "## r2" and "## PM decisions r2", findings/xp.md xp-10, xp-13, xp-30 on claude/session-log; `git diff d2a048d5..8109e08b`; captures media/2026-11-25/xp-minor-r2/.
Checks, each with its command and files:
1) Red first: 3fc6045b before 6d5c1f31, acee8d9f before cdc9446a, 28d29cc2 before 7ee01d44 (`git log --oneline --stat d2a048d5..8109e08b`). The lane says the window-case red unit was replaced in the fix commit: is the gate cell xp30-window red at 28d29cc2 by its own code (read tests/gate-xp-helper.sh at that commit)? A behaviour fix with no red test before it is MAJOR.
2) xp-30 (plugins/chora-assistant/src/{voice.rs,main.rs}, crates/chora-kit/src/words.rs): greets once (flag written only when shown); any press takes it away; waits while a window meets the balloon area; 500 ms retry: no busy loop, no log spam, no leaked scene.query per retry (leak cell). POLS for Erika 74: do the words tell how to close? de words correct.
3) xp-10 (plugins/chora-xp/src/wm/{panels.rs,windows.rs,pointer.rs}, datetime.rs): one click opens, a double click opens one dialog only, the task button appears, raises and goes with the dialog; no other panel lost its task button (sync_tasks split). `cargo test -p chora-xp`.
4) rxm1-1 (session.rs, tests/png-extent.py, tests/gate-xp-readable.sh): 4 px between captions on both measures; buttons still inside the dialog at 1024 and 1280; png-extent.py reads only the caption rows (no false green from the icon).
5) Gate cells: tests/gate-xp-helper.sh, gate-xp-readable.sh: no weakened assert, no new sleep (`tools/sleep-ratchet`), `ctl wait` on a missing node `pause` is a bounded wait, not a hang.
6) Beyond the brief: docs or words that still say "closes by itself"; the removed `last_clock` and startup.rs line leave no dead path.
Known, not findings (PM decisions r2): Esc does not close the greeting; 11 px type in Date and Time and desktop labels (docs/xp-style.md row 1); xp-16, xp-20..22, xp-34, xp-39.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rxm2-N, severity,
file:line, text, fix in one sentence). Append to reports/xp-minor.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
