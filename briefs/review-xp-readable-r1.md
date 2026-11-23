# review xp-readable r1 (F7, rc 1): seeded review of claude/xp-readable before its merge

Model: sonnet, effort medium (xp plugin colour and size fixes plus one gate; routine)
Lane: cloud, read-only on code: claude/xp-readable head 2d2413f6, base 5b34d4b1 (13 files, +209 -331). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the PM train runs it; the lane's gate had foreign reds: keys sh-34 and e2e gate-tiling-apps-click, both fuzzel 1.9.2, both red on 5b34d4b1).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-xp-readable BLOCKED: <line>" and stop.
Read briefs/xp-readable-r1.md (done criteria) and reports/xp-readable.md "## r1" on claude/session-log; vision-card.md (chora-xp: pixel-perfect XP SP2/SP3 Luna replica); `git diff 5b34d4b1...2d2413f6`.
Checks, each with its command and files:
1) Red first: test commit 13d31263 fails on its own commit (`git checkout 13d31263; bash tests/gate-xp-readable.sh`) and each fix (d78fc4bb, 7f5e3f39, f1693fb3, 54f810dc, 0e9e0bd3) comes after it. A fix without a failing test before it is MAJOR.
2) Exemptions: the three new 3.0 exemptions in tests/acts/known-red.toml (xp-task-, xp-menu-frame, xp-session-button-) are true XP Luna colours (cite the colour in docs/xp-style.md or the XP reference the repo uses) and the prefix match does not exempt other rows; the 31 removed rows each pass now (`grep` the acts run or rerun `tests/acts/run.py` for xp if present). An exemption that hides a fixable contrast row is MAJOR.
3) xp-13 Recycle Bin moved to the top right: XP SP2 puts it bottom right; judge against the vision card (replica) and POLS: is the notice collision better solved by moving notices? State MAJOR if the replica is broken without need, with the fix in one sentence; docs/xp-style.md row 11 matches the code either way.
4) xp-10 dialog buttons 75x23, xp-40 shadow captions, xp-41 inactive title, xp-39/19 pager digits: values match XP (75x23 px push button at 96 dpi) and `bash tests/gate-xp-readable.sh` 3x green at the head.
5) `cargo test -p chora-xp`, `cargo clippy -p chora-xp --all-targets` clean at the head; tests/merge-gate.sh adds the cell without a new sleep or --only gap.
Known, not findings: sh-34, fuzzel 1.9.2 reds, xp-10 single-click on clock/taskbar (open, next round).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rxr-N, severity, file:line, text, fix in one sentence). Append to reports/xp-readable.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
