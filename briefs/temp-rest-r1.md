# temp-rest r1 (F2, F5): the eight temp folders outside the counted helper

Model: sonnet, effort medium (mechanical moves with a red copies row)
Lane: cloud, branch claude/temp-rest from claude/temp-dirs 86f42d1f (temp-dirs r2, MERGE-OK, train 77). One round, stop at 250 k. Cost about 0.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "temp-rest r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/temp-dirs.md (r2 and PM review r2). tools/copies.toml row "temp folder by pid outside the counted helper" (max 8).
The eight sites: chora backend/headless/guard.rs:48, testing.rs:36, transaction.rs:604, render/sampling.rs:17 and :111, ipc.rs:1692, testing/wayland.rs:1325; chora-xp sheets.rs:46.
1. Red first: set the row to max 0 and commit (ci-check fails).
2. Each test-only site takes chora_scene::testing::TempDir (or chora_kit::testing::TempDir in plugins) and writes its file inside it; core testing.rs:36 and wayland.rs:1325 become uses of the one helper. A site in non-test code that hands a path on (a debug PNG): keep it only with a one-line reason and the row's allow naming that file; say why in the report.
3. Before the final gate: `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,xp` green. Poll it in the same turn; never end a turn to wait.
Done when: row max 0 (or each allow named with a reason); gate green; reports/temp-rest.md "## r1" on claude/session-log: each site, what it uses now, head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/temp-rest and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
