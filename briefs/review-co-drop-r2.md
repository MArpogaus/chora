# review co-drop r2 (rc 1, 2): seeded review of claude/co-drop-r2 before its merge

Model: opus, effort high, because it is a look review (construct focus rule across faces: a wrong rule sends keys into a window on a face the person does not see; critic 11-24: opus for look reviews); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-drop-r2 head 636287c0, base 75d4623e (7 files, +241 -52). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane's: all steps passed on 822d5be4; 636287c0 touches only tests/gate-construct-pull.sh, the pull step alone passed).
Read briefs/co-drop-r2.md (done criteria) and reports/co-drop.md ("## Review r1", "## r2") on claude/session-log; `git diff 75d4623e..636287c0`.
Step 0: `git fetch --unshallow`, `cargo check --workspace`.
Checks, each with its command and files:
1) Red first: 1d820350 (and bffb0f2d) before the fixes; at 1d820350 `cargo test -p chora-construct` fails the 3-face drop and the sheet test, nothing else; at 636287c0 all pass.
2) rcdr1-1 pull.rs: the noted window is forgotten on any face (raw focus); with 3+ faces Return after turning back activates no hidden window; no path where no look holds the keyboard or keys reach an unseen face. rcdr1-2 list.rs comment true.
3) rv9kb-3 command.rs (+131): the key sheet at 1280x720 inside [32, 688] with the foot row and Close on screen; at 1920x1080 and 1024x768 unchanged or better (read the layout math; captures media/2026-11-26/co-drop-r2/); 3 columns only when 2 do not fit; KISS (no new option, no copy of a kit layout helper); no new lint allow; size, sleep, lint ratchets (`tools/size-ratchet`, `tools/sleep-ratchet`, `tools/lint-ratchet`) ok.
4) Gate scripts: tests/gate-construct-pull.sh case `turned` asserts the holder, not a sleep; gate-construct-cube.sh change justified.
5) `git merge-tree --write-tree 6daaaab8 636287c0` clean; `cargo check --workspace` on the merged tree.
Known, not findings: FLAKE-CUBE r1, FLAKE-SOFA, FLAKE-PERSONA-NOTEPAD r1, construct give menu list.rs (filed with ti-57), rtf2-1 (TILING-FOCUS r3), rsl1-1.
Verdict computed: an open BLOCK or MAJOR is FIX; a failed Done-when line is INCOMPLETE; a fix with no red test commit before it is MAJOR; else MERGE-OK. Findings one line each (rcdr2-N, severity, file:line, text, fix in one sentence). Append to reports/co-drop.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
