# review tiling-disc r1 (rc 2): seeded review of claude/tiling-disc before its merge

Model: sonnet, effort medium (tiling key list, two small product changes and persona harness; routine)
Lane: cloud, read-only on code: claude/tiling-disc head 8df47cc8, base 5b34d4b1 (21 files, +507 -54). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the PM train runs it; the lane's gate had foreign reds: keys sh-34, xp-run, mc-host, e2e with fuzzel 1.9.2 in its container).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-tiling-disc BLOCKED: <line>" and stop.
Read briefs/tiling-disc-r1.md (done criteria 1-5) and reports/tiling-disc.md "## r1" on claude/session-log; `git diff 5b34d4b1...8df47cc8`.
Checks, each with its command and files:
1) Red first: each fix (efe0aa9d, d5151e38, 4c239af4, 9f84d3e4, a4badd8e) has its test commit before it (5abd20f8, 8158e4d8, 45742402, 7f49752e, bab7a89d) and the test fails on that commit (`git checkout <test sha>`; `cargo test -p chora-tiling` or `python3 -m pytest tests/persona/test_check.py -q`). A fix without a failing test before it is MAJOR.
2) L1: Alt+Tab moves focus only when no client holds the keys (no stray key into a client, RC bar); `bash tests/gate-tiling-focus-key.sh` 3x green; Super+Left/Right unchanged.
3) L2: "Edit settings" runs $EDITOR or foot with vi on the user's chora-tiling.yaml (no shell injection through the path; quoting in tiling.yaml); MOD+Ctrl+r named; `bash tests/gate-tiling-settings.sh` 3x green.
4) T7: the contract fails on a tag move and passes only when the window's owner changed look (tests/persona/check.py, test_check.py); no other contract loosened (diff of check.py).
5) Harness: round.sh stop and session.sh changes keep folders and kill the session on failure; gate-persona-keep.sh joins merge-gate.sh without a new sleep or known-red row.
6) cargo test -p chora-tiling, `cargo clippy -p chora-tiling --all-targets` clean at the head.
Known, not findings: sh-34 (keys cell red in lane containers), fuzzel 1.9.2 reds, ti-31 (T6 fuzzel).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtd-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-disc.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
