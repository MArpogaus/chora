# review co-focus r2 (rc 1, 2): seeded review of claude/co-focus before its merge

Model: opus, effort high, because the diff moves keys between the construct host and xp's faces (Start menu shut on a turn, kit::wm::focus) and tightens the persona contract C3, T1, T3; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-focus head 8a599917; own diff 8216d26e..8a599917 (3 files, +275 -26; 8216d26e merges the session branch 2c1748a4). One round.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review co-focus BLOCKED: <line>" and stop.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green: 62 PASS, EXIT 0).
Read briefs/co-focus-r2.md (done criteria), reports/co-focus.md "## r2" and "## Review r1" on claude/session-log; `git diff 8216d26e..8a599917`.
Checks, each with its command and files:
1) Red first: c0f0f1d3 holds the tests and comes before the fix; `cargo test -p chora-xp hosted` at c0f0f1d3 red (4 failed), at 8a599917 green (11).
2) rcf-1, rcf-2 (tests/persona/check.py, test_check.py): T1, C3, T3 accept a face manager only for a shown window, or a surfaceless manager only when its focused node draws on the front face's output in scene.boxes. A hidden window, navigate mode, or a node on another face must still fail; no other check (I2, I3, other looks) got looser. Be strict: this is where a loose contract hides a bug.
3) rcf-3 (plugins/chora-xp/src/wm/hosted.rs host_turned, shut_menus_off_front): a turn at rest shuts menus off the front face and gives the keys to the front face's top window, else the icons, through Wm::focus (D116); an open menu on the front face keeps its keys; keys held by another client are not taken; lock and escape hatch stay above; no key goes to a hidden window during a turn.
4) rcf-4 (find_host, host_back): front comes from Listing.front each time a host is taken or comes back; no stale front after a host restart.
5) Size and copies: `tools/size-ratchet`, `tools/cut-check`; no copy of kit focus logic.
6) `cargo test -p chora-xp -p chora-construct`, `python3 -I tests/persona/test_check.py`.
Known, not findings (open, next round): rcf-7 MAJOR toggle_start opens Start on face 0 (keys.rs:216); rcf-8 MINOR icons only on face 0 (panels.rs:622). Check only that this diff does not make them worse.
Verdict computed: an open BLOCK or MAJOR in this diff is FIX, else MERGE-OK. Findings one line each (rcf-N from rcf-9, severity, file:line, text, fix in one sentence). Append to reports/co-focus.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
