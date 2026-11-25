# tiling-doors r2 (rc 2: tiling persona r9 not met; ti-55, ti-56 MAJOR open 4th round; L2 false green)

Model: sonnet, effort high (three small decided changes, red tests first)
Lane: cloud, branch claude/tiling-doors-r2 at 721be592 (session head t134). Areas: flavors/tiling/waybar (custom/looks), plugins/chora-tiling (main.rs HINT_TIME, looks word), tests/doors/tiling.sh, tests/persona/check.py (verify_reloaded, T7 goal text if it lives there), tests/persona tasks/goals of tiling, docs/configuration.md. Stop at 250 k. Cost at most 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "tiling-doors-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-r9-tiling.md "## r9" (Fix check, Stop rule, open items 1-3); feedback/2026-11-25-p2r9ti-{ricer,kemal}.md; findings/tiling.md ti-55, ti-56; reports/tiling-doors.md "## r1".
PM decisions (11-25, final; do not reopen):
- ti-55: the bar word names where it leads. custom/looks shows "Looks: <name>" when one other look runs (e.g. "Looks: xp"), "Looks: xp, mc" for several, plain "Looks" when none; refresh on look join and leave (signal or interval <= 2 s, no sleep in tests). The persona T7 goal stays a goal in the user's words, with the target named as the bar names it: "Send the terminal over to the xp look (the other desktop that is running)". Change only that text wherever the tiling T7 goal is stored (repo or tests/persona); if it is not in the repo, put it in tests/persona/README.md under tiling goals.
- ti-56: HINT_TIME 3 s -> 8 s (a hint must be readable by slow eyes and by a persona look 4.3 s after a key); rest of the r1 rule unchanged (once per 10 s, not while the key list shows).
- L2 false green: verify_reloaded passes only when the value of gap_inner on disk differs from its value at begin AND the running config reports the new value after reload; a digest change alone fails.
1) Red commit first (tests only, message cites ti-55, ti-56, L2). Paste the failing lines:
   (a) doors cell looks_word: tiling+xp bar text of custom/looks is "Looks: xp"; tiling only "Looks";
   (b) chora-tiling unit test: the hint chip still shows 7 s after the key and is gone at 9 s (clock injected, no sleep);
   (c) check.py self-test (tests/persona-selftest.sh or a pytest beside check.py): open-editor-without-change -> L2 FAIL; value 12 -> 20 and reload -> L2 pass.
2) Fix commits, one per decision.
3) Tour: `demo/tour/run.py tiling` and `run.py handover` from a fresh compositor stay 28/28 and 11/11; update only expected bar words. Captures of the bar (tiling only, tiling+xp) and the hint at 7 s in media/2026-11-25/tiling-doors-r2/ on claude/session-log.
4) Full gate: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. `tools/sleep-ratchet`, `tools/size-ratchet`, `tools/lint-ratchet` ok. Before ending: `pgrep -fa 'chora --headless'` empty.
Never end a turn to wait; poll any long command in the same turn.
Done when: red then fix shas with pasted lines; tour 28/28 and handover 11/11; full gate green or its foreign reds named; three ratchets ok; captures pushed.
Out of scope: rth-*, rtd-*, rv9j-*, ti-31, ti-49, Super+q binding, the xp tray clock (I6), the L1 contract.
Report: reports/tiling-doors.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tiling-doors-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
