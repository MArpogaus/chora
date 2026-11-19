# kit-minors r1 (F2, F5): the open MINOR findings of kit5 and kit7

Model: sonnet, effort high (routine fixes with red tests)
Lane: cloud, branch claude/kit-minors from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-minors r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: backlog.md rows "kit5 r2 MINOR" and "kit7 r2 MINOR/nit", reports/kit5.md, reports/kit7.md. Do not touch fn report or lint attributes (lanes kit-report and lint-reason own them).
Each fix: its red test commit first, then the fix.
1. kit5 r2-1: a second chora-notify while the bus name is taken exits without a restart loop (notify main.rs:246); a test or gate cell shows one exit, no loop.
2. kit5 r2-2: mc agent.rs:50 caps the critical ids it keeps (a unit test at cap + 1).
3. kit7: helper.rs answered -> working has a test (helper.rs:195); a new utterance clears answered (helper.rs:205, red test first); seed() keeps report order (helper.rs:142).
4. Nits only if small: kit5 r2-3 card width over a very wide chat (card.rs:274); docs/api.md:1573 rewrap.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test` green. Poll it in the same turn; never end a turn to wait.
Done when: items 1-3 in with red first; gate green; reports/kit-minors.md "## r1" on claude/session-log naming each finding closed and its test; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-minors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
