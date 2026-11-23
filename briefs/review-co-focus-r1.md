# review co-focus r1 (rc 1, 2): seeded review of claude/co-focus before its merge

Model: opus, effort high, because the diff moves key focus between the construct host and xp faces (stray keys, rc bar) and loosens three persona contract checks; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-focus head 866382dd, base 86e619f3 (10 files, +358 -10). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green: 60 PASS).
Read briefs/co-focus-r1.md (done criteria), reports/co-focus.md "## r1" and findings/construct.md co-48..co-51 on claude/session-log; `git diff 86e619f3..866382dd`.
Checks, each with its command and files:
1) Red first: 9b0466ae holds the unit tests, gate-construct-focus.sh and test_check.py cases and comes before the fix; `cargo test -p chora-xp hosted` at 9b0466ae red, at 866382dd green (run both if cheap).
2) co-50 (plugins/chora-xp/src/wm/dispatch.rs, hosted.rs, windows.rs): after a host turn the keys leave a window of a face out of sight and go to the front face, through kit::wm::focus (D116); keys held by another client are not taken; no key during the turn itself goes to a hidden window. Navigate mode (step out) unchanged.
3) co-51 (keys.rs, windows.rs): a `focus.changed null` before `surface.destroyed` passes the keys on; no path leaves the core focus empty while xp or construct claims to hold; the escape hatch and lock stay above (lock window gets the keys while it is up).
4) Contract change (tests/persona/check.py, test_check.py): C3, T1, T3 accept the front face's manager only in use mode, and only for a shown window; cite docs/cube-modes.md States. A hidden window or navigate mode must still fail; check that no other check (I2, I3, other looks) got looser. This is the point most likely to hide a bug: be strict.
5) hosted.rs 147 lines new: size ratchet and copies (`tools/size-ratchet`, `tools/cut-check`); no copy of kit focus logic.
6) `cargo test -p chora-xp -p chora-construct`, `python3 -I tests/persona/test_check.py`.
Known, not findings: co-51 demo rows I2/I3 at lock-window map (0/5, open verify); Super+Return on a face (UX triage).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcf-N, severity, file:line, text, fix in one sentence). Append to reports/co-focus.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
