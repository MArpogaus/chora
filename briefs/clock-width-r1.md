# clock-width r1 (F5, rc 3: train 104b red only in a date-dependent check)

Model: sonnet, effort medium (one gate check made independent of the date, behind a red test; no product code)
Lane: cloud, branch claude/clock-width at 1c9e64f7 (session head). Cost at most 0.4 %; stop at 150 k context. One area:
tests/gate-tiling-bar-spec.sh (and a test of it); no product code.
Why: train 104b (00:36 UTC 11-22, a Sunday) failed tiling-look only in bar-font: "the clock is 120.0 wide, not Inter 14 px
weight 500 (100-119)" (also 120.5 at scale 2, both themes). The same branches passed it in train 104 at 23:5x on Saturday.
gate-tiling-bar-spec.sh:170 holds a fixed width range for a clock whose text changes with the date and time.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure:
send_message the PM "clock-width BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first: show the cell red on the current date (log line), or with the clock text of 11-22 00:36 if the date moved on.
   Commit a red test or the red evidence in the commit message.
2) Fix (KISS): the cell's width bound follows the clock text actually shown (from the bar's state or the capture time, read
   the code that formats it), so any date and time passes with Inter 14 px weight 500 and the mono face still fails. If the
   bar can take a fixed time for tests, pinning it is equally fine. Prove: the cell green now, and red with the mono face
   (or a width measurement of the mono face in a test). Commit.
Gate: `tests/merge-gate.sh --jobs 1 --only ci-check,tiling-look` green (no product code), then full `tests/merge-gate.sh
--jobs 1` green on claude/clock-width merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not
cause: name it with its log line; do not rerun to green. Run it in the background and poll in the same turn; never end a
turn to wait.
Done when: red and fix commits, the cell green at the current date, the full gate green.
Report: reports/clock-width.md "## r1" on claude/session-log (shas, the red line, the fix line, gate summary); send_message
the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/clock-width and claude/session-log only (fetch and merge on reject, no force; on claude/session-log
change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>
(git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
