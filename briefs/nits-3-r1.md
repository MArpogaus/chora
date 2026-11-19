# nits-3 r1 (F5): three small open NITs

Model: sonnet, effort medium (small fixes with red tests)
Lane: cloud, branch claude/nits-3 from the session branch a9e7084d. One round, stop at 250 k. Cost about 0.4 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "nits-3 r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/kit5.md (r2-3), reports/kit-minors.md (NIT test name), reports/f24-rest.md and backlog row F24R-1.
Each fix: its red test commit first, then the fix.
1. kit5 r2-3: plugins/chora-notify/src/card.rs:274, a card over a chat wider than the room beside it falls back to a vertical step (never over the chat); extend the unit test `wide` (red first).
2. kit-minors NIT: rename the_critical_ids_are_capped_at_one_more to say the cap (64) it checks.
3. F24R-1 (after merging claude/f24-rest, MERGE-OK 7231b51a): tests/grab-gap-check.py also flags a callee that skips its grab on a branch, or the selftest documents why it cannot; selftest case first.
4. Before the final gate: `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,xp` green. Poll it in the same turn; never end a turn to wait.
Done when: items 1-3 with red first; gate green; reports/nits-3.md "## r1" on claude/session-log, each item, its test, head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/nits-3 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
