# xp-tidy r3 (RC 2, xp; F4 lint ratchet): make the reviewed xp-tidy branch pass lint-ratchet after lint-down

Model: sonnet, effort medium (a mechanical lint fix on a reviewed branch)
Lane: cloud, claude/xp-tidy at 74b423bb (review r2 MERGE-OK). Cost at most 0.4 %; stop at 150 k context.
Why: train 92 (20:02 UTC) failed only on ci-check lint-ratchet: lint-down (78ec62e1) set tools/lint-ratchet.list; xp-tidy adds one `#[expect(` (plugins/chora-xp, slider percent, has a reason) and one `eprintln!("chora-xp: clients.list: {err}")`. The list says eprintln only falls; allow may rise only with a reason, in the same commit, the message saying why.
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "xp-tidy r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
1) `git fetch origin claude/lint-down && git merge --no-ff 78ec62e1` (message "Merge branch 'claude/lint-down' into claude/xp-tidy"); `tools/lint-ratchet` shows the breach (that is the red).
2) Fix: the clients.list error must stay visible once (POLS: honest failure); keep eprintln out of the count the cleanest way the code allows (an existing log path of chora-xp, or remove an equal number of redundant eprintln in chora-xp with no lost message); for the `#[expect(` raise `allow` in tools/lint-ratchet.list by one in the same commit, message says why. No other change.
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,xp-faces,xp-notepad,launcher-click`. Poll in the same turn; never end a turn to wait.
Report: append "## r3" to reports/xp-tidy.md on claude/session-log (shas, gate cells, what replaced the eprintln); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/xp-tidy and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
