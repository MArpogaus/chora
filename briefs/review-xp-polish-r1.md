# review xp-polish r1 (rc 8, xp): seeded review of claude/xp-polish r1 before its merge

Model: sonnet, effort high (port-level UI fixes in two plugins; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/xp-polish head 64478c66, base 2235551f (11 files, +277 -43). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (lane gate: all PASS, tiling-windows once foreign "slide is not between the places", PASS on rerun).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `cargo test -p chora-xp -p chora-assistant`. A failure: send_message the PM "review-xp-polish-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/xp-polish-r1.md (done criteria), reports/xp-polish.md "## r1", findings/xp.md xp-14, xp-15, xp-17; `git diff 2235551f..64478c66`; captures/xp-polish-r1/*.png on claude/xp-polish or the log.
Checks, each with its command and files:
1) Red first: 11007d3a (tests only) before 86343e3f, 63fff561, a0c8ce36; each red cell fails at 11007d3a (cargo test or xp.sh line pasted) and passes at head.
2) xp-14: the typing line stands inside the question card, no hint under it (card.rs, chat/view.rs); the gate-xp-question.sh change reads the new place and asserts no weaker than before (diff the asserts; a dropped assert is MAJOR).
3) xp-15: a cut Start menu name shows in full in a tooltip; uncut rows get none; the tooltip clears on leave (startmenu.rs, pointer.rs).
4) xp-17: Display Properties has a task button as Help has; it closes with the window, no lost window, no second button on reopen (display.rs, wm/windows.rs, tests_moved.rs).
5) KISS, POLS, the XP look: no new mode or setting; tooltip and button look and timing as XP (design/xp*.md); no other look touched (chora-assistant change keeps the other costumes: name the cells or tests that show it).
6) The diff touches no file the tiling-windows cell runs (so the flake is foreign).
Known, not findings: xp-18, xp-34, xp-38, co-*, ti-*.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rxp1-N, severity, file:line, text, fix in one sentence). Append to reports/xp-polish.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
