# review handover-mo r1 (rc 2: ca-4, sd-h2 fixed; sh-23, sd-h1 shown not red): seeded review before the merge

Model: sonnet, effort high (xp plugin keys and windows plus a new gate script and a merge-gate.sh step; the builder never reviews itself)
Lane: cloud, read-only on code: claude/handover-mo head f7dded7a, red 8a3c2bc9, fix 20e4ad6e, base 598a34ff (6 files, +231 -6). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "review-handover-mo-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/handover-mo-r1.md, reports/handover-mo.md "## r1", the rows ca-4, sd-h1, sd-h2, sh-23 in findings/. `git diff 598a34ff..f7dded7a`.
Checks, each with its command and files:
1) Red first: 8a3c2bc9 holds only the gate and its merge-gate.sh wiring; cells cube and give red there, green at f7dded7a (`tests/gate-start-tap.sh` once per sha if it runs under 10 min).
2) ca-4 fix (plugins/chora-xp/src/wm/keys.rs button()): a lone Super tap still opens and closes Start (cell tap); a Super with any other key or button between press and release never toggles Start; `cargo test -p chora-xp`.
3) sd-h2 fix (windows.rs adopt): menus shut when a window arrives by give or drag, not when a window of xp's own maps under an open menu the person is using (e.g. a balloon or a dialog opened from the menu).
4) Gate script: not vacuous (a cell fails when its window or Start state is missing, no silent skip); runs inside the matrix step; adds no flake (timing 0.1 s steps: look for sleeps without a poll).
5) sh-23 and sd-h1 "not red 5/5": the report blames demo/tour/chapters/handover.py:35 starting tiling without --with-companions. Judge: can a person reach tiling without companions (default launcher, docs, session file)? If yes, the black output with no bar or start door is still a product MAJOR (keep the rows open, name the path); if no, the rows close and the tour line is a chapters-a fix.
6) KISS: no change outside xp, the new gate and its wiring; state.rs addition minimal.
Known, not findings: co-keys owns construct focus; chapters-a owns the tour harness.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rhm-N, severity, file:line, text, fix in one sentence). Append to reports/handover-mo.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list, the check 5 answer. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
