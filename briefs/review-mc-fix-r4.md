# review mc-fix r4 (RC 2 persona round 4, stability: mc-42 rv-1, rv-2, rv-3): seeded review of claude/mc-fix before its merge

Model: sonnet, effort high (named fixes against the r3 review's own findings and fixes; small diff, plugin and checker only)
Lane: cloud, read-only on code: claude/mc-fix head a8cd0de0, base b0997b71 (7 files, +168 -19). One round.
Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No full merge gate (the lane ran 11 gate steps green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/mc-fix`; `cargo check --workspace`; a failure: send_message the PM "review mc-fix r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/mc-fix-r4.md (done criteria), reports/mc-fix.md "## Review r3" and "## r4"; then `git diff b0997b71..a8cd0de0`.
Checks, each with its command and files:
1) Red first: ad1f2ded fails on its parent and passes at 0b72ff84; ee8b0ada before 75272909; 5c422e10 before 6508d95d (cargo test -p chora-mc, python3 -m pytest tests/test_acts.py per sha in a scratch worktree). The cell terminal-turn (a8cd0de0, one inject Right, Left, Right): run tests/gate-mc-paths.sh with it at 0b72ff84^ (expect red) and at a8cd0de0 (green) if headless, else say so.
2) rv-1, show.rs: plain_start set while waiting and taken only on a name match. Any path where a window that should get the keys now never does (the launcher path, a second program of the same name, a program that never maps)? Is plain_start cleared when the page is left for good?
3) rv-2, main.rs refresh: id:<own node> with the focused filter; the keep case (own node focused, holder mc) and the drop case both tested; a failed query logged once, not per frame.
4) rv-3, tests/acts/invariants.py: the lane deviated from the review's rule ("a whole-output picture of the look above, not of the look below"). Can a real I5 breach (mc drawn over xp while xp is shown) still pass? Does the r3 guard hold? Each new case in test_acts.py fails without the change.
5) `cargo test -p chora-kit -p chora-mc`, `python3 -m pytest tests/test_acts.py -q`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
Known, not findings: launcher-click red in lane containers (fuzzel 1.9.2); mc-50 yellow square, T1 end-time gap.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/mc-fix.md on claude/session-log under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
