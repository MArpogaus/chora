# review flake-series r8 (F5, rc 3, 9): seeded review of claude/flake-series-r7 before its merge

Model: opus, effort medium, because the leak verdict is a statistic (slope bound, rounds) where a wrong bound hides a real leak or keeps a flake; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/flake-series-r7 head d2c2174c, base 2adac16a (6 files, +139 -54). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it 3/3 green). Step 0: `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-flake-series-r8 BLOCKED: <line>" and stop.
Read briefs/flake-series-r8.md (done criteria) and reports/flake-series.md "## r8" on claude/session-log; `git diff 2adac16a..d2c2174c`.
Checks, each with its command and files:
1) sh-38 leak verdict (tests/leak-verdict.awk, tests/gate-leak.sh, red 252b3d3e, fix d2c2174c): the bound scaled by rounds is not a raised bar (the 1024 at 10 rounds); a planted +1.5 MB/round still fails; `sh tests/leak-verdict-selftest.sh` at 252b3d3e (red) and d2c2174c (green). Say whether 12 rounds fits the gate time.
2) xp-42 (tests/gate-xp-faces.sh; c0900955 revert, red 5b280d4c, fix e7192a72): the red is shown on its own commit before the fix; the wait is a poll on a condition, not a sleep.
3) rfs7-1 (380fa09a, merge-gate.sh poll(), merge-gate-selftest.sh poll_for()): `tools/sleep-ratchet` on d2c2174c merged with d40142e4 (the session head; run in a scratch worktree) passes at <= 779 / 930; no fixed sleep moved into a helper.
4) Red-first per fix commit: each fix(...) has a test commit before it that is red on its parent (procedures Seeded review: a fix without one is MAJOR; 380fa09a has no red: judge the lane's reason).
5) `bash tests/merge-gate-selftest.sh` at d2c2174c: the 2 pre-commit-stub FAILs the lane names are also on 2adac16a (foreign) or caused here.
Known, not findings: cause of the heap level switches open; rfs7-3 untouched.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rfs8-N, severity, file:line, text, fix in one sentence). Append to reports/flake-series.md on claude/session-log under "## Review r8".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages; never a bare git stash.
