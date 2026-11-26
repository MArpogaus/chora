# window-loss r1 (rc 9, shared RC bar "no lost window in any handover"; A13, A17): reproduce or close the two old window-loss items

Model: haiku, effort high (testing: repeat existing gate cells and a scripted handover, count, no product code)
Red on head: unknown; A13 (mc drag onto construct: construct loses its panes, `pane: none`) and A17 (matrix gate ends a window in xp without keyboard focus under load) are open in backlog.md since 11-09, never re-checked on the session head
Lane: cloud, branch claude/window-loss-r1 at 7c6955c8 (session head). Area: tests/ only (a new script tests/repeat-handover.sh if needed); no product code. Stop at 250 k. Cost at most 0.4 %.
Why: backlog.md "## UX and looks" A13 and "## Flakes and gates" A17; RC 9 (sl-1, sl-2) in state.md.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "window-loss-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. A13: find the matrix/handover cell that drags an mc window onto the construct output (grep tests/ for `window.show` repair and `pane`); run that path 10 times with the setup repair removed in a local copy only (do not commit the removal); count `pane: none`.
2. A17: run the matrix gate cells cgive and voice 10 times each, 5 alone and 5 beside a `cargo build --workspace` load; count windows in xp without keyboard focus.
3. Each count with its log under media/2026-11-26/window-loss-r1/ (logs only, each under 500 KB).
Done when: both counts in reports/window-loss.md "## r1" on claude/session-log with commands, logs and a verdict per item: CLOSED (0 of 10 and 0 of 10) or OPEN (with the first failing log line and the cell that shows it, as a candidate red cell for a fix lane). No fix in this lane.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): the two counts, verdicts, log paths.
Rules: push claude/window-loss-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead). Before ending: `pgrep -fa 'chora --headless'` empty. Poll it in the same turn; never end a turn to wait.
