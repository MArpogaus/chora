# kit-minors r2 (F2, F5): the xp cell after the notify message change

Model: sonnet, effort medium (one test string, one gate cell)
Lane: cloud, branch claude/kit-minors at 88f441e4 (r1 head). One round, stop at 250 k. Cost about 0.3 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-minors r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Finding (train 73, 21:17 UTC): the xp cell fails at step 17, `FAIL: no word of the owner`. r1 commit 8348e8c1 changed the second chora-notify's line to "chora-notify: the name notify is taken, another chora-notify serves the session"; tests/xp.sh:593 still greps "another owner".
1. Run `tests/merge-gate.sh --jobs 3 --only build,ci-check,xp` first and keep the red log.
2. Fix tests/xp.sh step 17 so it checks the new line ("another chora-notify serves the session") and that the second notify exits 0 within its timeout; rename the step text if "leaves the bus name to the first" is no longer true. Do not change the r1 Rust behaviour unless the cell shows it is wrong.
3. grep tests/ for any other check of the old wording ("another owner", StartError::Taken text) and align it.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,xp,virtual` green. Poll it in the same turn; never end a turn to wait.
Done when: xp green; reports/kit-minors.md "## r2" on claude/session-log with the red and green cell logs' last lines; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-minors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
