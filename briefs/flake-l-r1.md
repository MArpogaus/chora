# flake-l r1 (F5): xp 16c flake "said Notify Starting slow-manager" (rm-2, third hit)

Model: sonnet, effort high (routine flake fix, reproduce then fix)
Lane: cloud, branch claude/flake-l from the train 74 merge a9e7084d. One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-l r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md rm-2, reports/merge-65.md, reports/mc-i5.md. Do not touch the cells of the running lanes flake-h, flake-k (hud), temp-rest (test temp folders), nits-3.
The failure (mc-i5 gate, train 65, train 76; each `--jobs 3`, full gate): tests/xp.sh step 16c, after the Run dialog, `open_my_documents_with "$out/slow-manager"` (a script: sleep 3, exec foot), then `wait_for said Notify "Starting slow-manager"` timed out. Alone `--only xp` passed each time.
1. Reproduce: the xp step 30 times under load (`--jobs 3` beside a `cargo build --workspace` of a second target dir, or stress); name the count.
2. Cause: is the "Starting" notice tied to a timer that races the 3 s child under load, does the click on My Documents miss (the Run dialog or the menu still open), or is the notice shown and closed before wait_for polls? A product fault (no notice for a slow start) is a finding: red test first, then the fix in the plugin.
3. Fix: bounded waits on state (until_ok of the gate lib), no retry of the whole cell, no longer plain sleep; the check must still prove a slow start gets a message that names it.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,xp` green, then xp twice more under load. Before the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: 0 of 30 under load after (counts before and after), or a named product finding with its red test and fix; gate green; reports/flake-l.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-l and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
