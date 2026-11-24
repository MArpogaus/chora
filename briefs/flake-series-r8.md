# flake-series r8 (F5, F4, rc 3): fix PM review r7 (rfs7-1, rfs7-2 MAJOR) and sh-38, red test first

Model: sonnet, effort high (harness and test-race fixes; statistics of a bound)
Lane: cloud, branch claude/flake-series-r7 at 6f7bcb26; first merge origin/claude/autonomous-dev-session-3w-i33pg7 (ce866b0d). One round, stop at 250 k. Cost at most 1.2 %.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; no setup.sh; failure: send_message the PM "flake-series-r8 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/flake-series.md "## r7" and "## PM review r7"; findings/shared.md sh-38; findings/xp.md xp-42.
1. rfs7-1: after the merge, `grep -h -E '(^|[^a-z_])sleep ' tests/*.sh | wc -l` must be <= 779 (now 787). Replace the new waits with poll loops on a condition through existing gate-lib helpers (until_ok or similar) or remove an equal number of fixed sleeps in the files you touch; no behaviour lost; report the count.
2. rfs7-2: xp-42 red: a forced delay (e.g. an env knob in the cell that injects the motion late, or a stub that drops the first pointer.entered) that makes the old cell fail and the new one pass; red commit first, then keep or adjust 05a23dca. If no red can be built, revert 05a23dca and say so.
3. sh-38: the leak step's RSS slope. Measure its spread (at least 10 runs of the cell, idle and beside `cargo build --workspace`); red: a selftest with the r7 numbers (-1199..+1613 kB/round, real leak absent) that the current bound fails; fix: a bound or statistic outside the noise (e.g. more rounds, median slope, or a fit over warm rounds) that still fails a planted leak (keep or add a planted-leak selftest that must fail). No bound raised without the planted leak failing.
4. Gate: `tests/merge-gate.sh --jobs 2` (full, no --only) three times on the head; each exit and red step; no rerun to green; `pgrep -a chora` after each.
Done when: red commits before each fix, pushed; W9 count <= 779; leak cell 10/10 idle and the planted leak red; reports/flake-series.md "## r8" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "flake-series-r8 DONE: <head sha>, <greens>/3" plus causes (at most 6 lines).
Poll every command in the same turn; never end a turn to wait.
Rules: push claude/flake-series-r7 and claude/session-log only (fetch and merge on reject, no force; on session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
