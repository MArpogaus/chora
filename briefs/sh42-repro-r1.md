# sh42-repro r1 (rc 1: reproduce sh-42 on the session head before routing)

Model: haiku, effort high (scripted hand runs and state dumps; tests)
Lane: cloud, branch claude/sh42-repro-r1 at c3c37c2e (session head after train132). Run only: no product, test or design changes. Stop at 150 k. Cost at most 0.4 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`. A failure: send_message the PM "sh42-repro-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md sh-42; reports/stray-super.md "## r2" (Found on the way: the steps and dumps); reports/co-repro.md "## r1" (the form); tests/persona/README.md and docs on handover (D119) on the branch.
1) sh-42, 5x each from a fresh compositor: (a) xp with a focused Foot, then tiling joins beside it (the look switch or join the r2 report used); (b) the same from mc. Per run record: core focus holder, each look's holder (dump line), whether typed text `echo r$N` reaches the focused window (capture or foot output), and in mc whether a click or Return gives the keyboard back. Then 3x each as control: the same switch with no window open, and xp to mc without tiling.
2) A rate table per path; each capture under 2 MB, at most 20, under media/2026-11-24/sh42-repro/ on claude/session-log. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait; poll in the same turn.
Done when: the table with rates and captures; one line: "sh-42: keys lost N/5 (xp), M/5 (mc); recovered by click K/M" and the steps that reproduce it shortest.
Report: reports/sh42-repro.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines.
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
