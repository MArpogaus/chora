# flake-series r7 (F5, rc 3): fix sh-34, the stray-process stall and xp-42, red test first

Model: sonnet, effort high (harness and test-race fixes; no new feature)
Lane: cloud, branch claude/flake-series-r7 at 1ec04694 (session head, t117 green). One round, stop at 250 k. Cost at most 1 %.
Why: series r6 was 0/5 green on 230ea0e5 (reports/flake-series.md r6): keys red x4 (sh-34, lane containers only), acts red x1 with reruns stalled by leftover chora-voice and chora-assistant processes, xp-42 2 hits in ~8 gates. F5 cannot close while these are open.
Step 0: `command -v cargo sccache foot fuzzel python3 ffmpeg; fuzzel --version`, `git fetch --unshallow`, `cargo check --workspace`; no setup.sh; a missing tool or failure: send_message the PM "flake-series-r7 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md sh-34, sh-29; findings/xp.md xp-42; reports/flake-series.md r6.
Runs take long: start in the background and poll in the same turn; never end a turn to wait.
1. sh-34: reproduce the keys cell "the Foot row of Programs started no foot" here (10 runs of the cell alone). Find the cause (fuzzel version, foot start time, harness wait). Red test or reproduction commit first, then the fix. If the cause is a missing or different tool in the lane environment, the fix is a need_fuzzel / need_foot guard that reports SKIP with the reason, plus the line for the environment in the report; do not install packages.
2. Stall: the gate's cleanup must leave no chora-voice or chora-assistant process (check `pgrep -a chora` after each gate). Red check first (a gate step that fails when one is left), then the cleanup fix in tests/ (never kill by name outside the gate's own session).
3. xp-42: the first-placement race in tests/gate-xp-faces (pointer read before the injected motion lands). Red first (a loop that hits it, or a forced delay), then the fix; no timeout raised as the fix.
4. Gate: `tests/merge-gate.sh --jobs 2` (full, no --only) three times; report each exit and red steps. No rerun to green; no test ignored or marked known-red.
Done when: three commits red-then-fix per item on claude/flake-series-r7, pushed; sh-34 cell 10/10 green (or SKIP with a named missing tool), xp-faces cell 20/20, zero chora processes left after each of the 3 gates; reports/flake-series.md "## r7" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "flake-series-r7 DONE: <head sha>, <greens>/3" plus causes (at most 6 lines).
Rules: push claude/flake-series-r7 and claude/session-log only (fetch and merge on reject, no force; on session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
