# kit-connect r2 (F2): the Runtime::with left by cube-tests

Model: sonnet, effort low (one test helper site, a merge)
Lane: cloud, branch claude/kit-connect at f666a025 (kc-fix on your r1). One round, stop at 250 k. Cost about 0.3 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-connect r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Why: train 75 (22:18) failed in ci-check: "cut-check: copies: own test connect (Runtime::with in a plugin): 1 left, the row allows 0". The site came from cube-tests (train 72), which your r1 did not see: plugins/chora-construct/src/cube/tests_construct.rs:64 (`Runtime::with(conn, NAME, Vec::new())` after conn.apply and set_output_owner on a real core).
1. `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it.
2. Move that site to a kit helper (FakeCore::runtime if it fits; else a small kit helper that takes the prepared conn, used by this site and any other that fits). No allow and no raised max: the copies row stays at 0. Do not change what the cube tests check.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube` green. Poll it in the same turn; never end a turn to wait.
Done when: `grep -rn 'Runtime::with(' plugins` is empty; gate green; reports/kit-connect.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/kit-connect and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
