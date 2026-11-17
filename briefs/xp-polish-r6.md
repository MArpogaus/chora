# xp-polish r6 (rc 1): Notepad notice only after the window shows; no second editor over a slow one

Model: sonnet, effort medium (two fixes in one launcher, review-found)
Lane: cloud, branch claude/xp-polish (head f40b2ddc); merge the session branch first. One round. Stop at 250 k.
Read reports/review-xp-polish-r5.md (MAJOR 1, 2), reports/xp-polish-r5.md, dev-rules.md.

1. MAJOR 1 (launch.rs:96-106, 219-300): "Notepad opened with X" only once X's window shows (Done path).
   Before that, either no notice or "Starting Notepad with X"; a failing fallback never leaves a success notice.
   Red cell first: a fake fallback editor that starts and exits without a window; assert no "opened" notice, and the next step or "Cannot open Notepad".
2. MAJOR 2 (launch.rs:382): a dead launcher pid within 2 s must not start the next editor while a hand-off editor
   is still coming up. KISS: wait for a window up to the old limit when the launcher exited with status 0; fall
   through early only on a non-zero exit or a missing binary. Red cell first: a fake editor that forks, exits 0,
   and opens its window after 4 s; assert exactly one editor window and no fallback notice.
3. Docs only if the behaviour text changes (docs/configuration.md, xp.yaml).
No MINOR work (rules.md 6; MINOR 1, 2 of the review go to known issues). Fakes on PATH only, no packages.
Done when: the cells pass, cargo test and clippy of chora-xp pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Note: e2e gate-tiling-apps-click fails in your container with fuzzel 1.9.2 also on base; report it, do not chase it.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/xp-polish-r6.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/xp-polish and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
