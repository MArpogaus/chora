# review tiling-focus r3 (rc 1, 2; ti-57; rtf2-1, rtf2-2): seeded review of claude/tiling-focus-r3 before its merge

Model: opus, effort high, because the diff completes the D119 keyboard-holder rule in mc across outputs, where a wrong rule sends keys into a client the person does not see; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-focus-r3 head d4fd349f, own work 668a9c67..d4fd349f (7 files, +333 -49; 668a9c67 merges 6daaaab8 and 1ab2e6a7, both already reviewed). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green: 67 PASS).
Read briefs/tiling-focus-r3.md (done criteria), reports/tiling-focus.md "## Review r2" and "## r3" on claude/session-log; `git diff 668a9c67..d4fd349f`.
Checks, each with its command and files:
1) rtf2-1 giver (plugins/chora-mc/src/manage.rs, windows.rs, main.rs): Mc::hand_kept keeps the keyboard on the window of the scene shown after the leave, else the root node mc, before the give; a refused give puts focus back; uses kit Focus::keep, no per-look copy of the rule (compare plugins/chora-tiling and chora-xp hand_kept, crates/chora-kit/src/give.rs). `cargo test -p chora-mc`.
2) rtf2-1 receiver: settle and adopt take the keyboard only with `origin != Given || focus.takes_given(..)`; Book::confirm_keys(id, keys); no path shows a scene with keys while another look holds them.
3) Red first: 8b22151a before 41ddd4ba (`git log --oneline 668a9c67..d4fd349f`); the four cells tiling mc key, xp mc key, mc tiling key, mc xp key expect kept in tests/gate-handover-matrix.sh and gate-handover-virtual.sh.
4) Gate setup commits 7b46b322, 0dd8cb1d, d4fd349f: `place` through the keyboard holder is a test-only change that does not hide a real fault (would a person hit the "window has no focus with mc" case? if yes, that is a finding); no new sleep line (tools/sleep-ratchet).
5) rtf2-2: D119 (docs/decisions.md) and docs/api.md "Window managers" match the code for xp Control Panel rows and mc.
Known, not findings: construct give path open (CO-DROP r3 runs); rmm5-1 MINOR filed.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtf3-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-focus.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
